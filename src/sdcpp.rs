use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, Result, anyhow, bail};
use gif::{Encoder, Frame, Repeat};
use image::{AnimationDecoder, codecs::gif::GifDecoder};
use tokio::{fs, process::Command};
use walkdir::WalkDir;

use crate::{
    gguf::{GgufSummary, inspect_gguf},
    types::{
        BackendRuntimeStatus, GenerateRequest, InputAsset, MediaKind, ModelBackend, ModelInfo,
        ReferenceIntent, RuntimeAcceleration,
    },
};

const BUILD_DIR_NAME: &str = "build-chatty";
const DEFAULT_VIDEO_FPS: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SdcppBuildMode {
    Vulkan,
    CpuOnly,
}
#[derive(Debug, Clone)]
pub struct SdcppSupport {
    pub family: String,
    pub runtime_supported: bool,
    pub compatibility_note: String,
    pub supported_kinds: Vec<MediaKind>,
    pub requires_reference: bool,
    pub supports_image_reference: bool,
    pub requires_end_image_reference: bool,
    pub supports_end_image_reference: bool,
    pub supports_video_reference: bool,
    pub supports_audio_reference: bool,
}

#[derive(Debug)]
pub struct SdcppGeneration {
    pub mime: String,
    pub note: String,
}

#[derive(Debug, Clone)]
enum RuntimeFamily {
    StableDiffusion,
    StableDiffusion3,
    Flux,
    FluxKontext,
    QwenImage,
    QwenImageEdit,
    Wan,
    ZImage,
    OvisImage,
    Anima,
    Ltx,
    StandaloneDiffusion,
}

#[derive(Debug, Clone, Copy)]
enum ReferenceMode {
    InitImage,
    RefImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WanVariant {
    T2v,
    I2v,
    Flf2v,
    Ti2v,
    Vace,
}

#[derive(Debug, Clone)]
struct RuntimeRecipe {
    family: RuntimeFamily,
    runtime_tree_ready: bool,
    model_path: PathBuf,
    vae_path: Option<PathBuf>,
    llm_path: Option<PathBuf>,
    llm_vision_path: Option<PathBuf>,
    t5_path: Option<PathBuf>,
    clip_l_path: Option<PathBuf>,
    clip_g_path: Option<PathBuf>,
    clip_vision_path: Option<PathBuf>,
    high_noise_model_path: Option<PathBuf>,
    missing: Vec<String>,
    supported_kinds: Vec<MediaKind>,
    requires_reference: bool,
    supports_image_reference: bool,
    requires_end_image_reference: bool,
    supports_end_image_reference: bool,
    supports_video_reference: bool,
    compatibility_note: String,
    reference_mode: Option<ReferenceMode>,
    extra_args: Vec<String>,
}

pub fn detect_sdcpp_support(
    diffuse_runtime_dir: &Path,
    models_dir: &Path,
    model_relative_path: &str,
    file_name: &str,
) -> Option<SdcppSupport> {
    let model_path = models_dir.join(native_relative_path(model_relative_path));
    let recipe = detect_runtime_recipe(diffuse_runtime_dir, models_dir, &model_path, file_name)?;

    Some(SdcppSupport {
        family: recipe.family_label().to_string(),
        runtime_supported: recipe.runtime_supported(),
        compatibility_note: recipe.compatibility_note,
        supported_kinds: recipe.supported_kinds,
        requires_reference: recipe.requires_reference,
        supports_image_reference: recipe.supports_image_reference,
        requires_end_image_reference: recipe.requires_end_image_reference,
        supports_end_image_reference: recipe.supports_end_image_reference,
        supports_video_reference: recipe.supports_video_reference,
        supports_audio_reference: false,
    })
}

pub fn realism_runtime_status(diffuse_runtime_dir: &Path) -> BackendRuntimeStatus {
    let build_mode = detect_sdcpp_build_mode(diffuse_runtime_dir);
    let build_exists = find_sd_cli(diffuse_runtime_dir).is_some();

    if !diffuse_runtime_dir.join("CMakeLists.txt").exists() {
        return BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "Runtime missing".to_string(),
            acceleration: RuntimeAcceleration::IncompleteTree,
            note: "Add a full stable-diffusion.cpp source tree to diffuse_runtime/ to enable realism mode.".to_string(),
        };
    }

    if !diffuse_runtime_dir
        .join("ggml")
        .join("CMakeLists.txt")
        .exists()
    {
        return BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "Runtime incomplete".to_string(),
            acceleration: RuntimeAcceleration::IncompleteTree,
            note: "diffuse_runtime/ggml is missing. Use a stable-diffusion.cpp checkout with submodules, not a partial source zip.".to_string(),
        };
    }

    match build_mode {
        Some(SdcppBuildMode::Vulkan) => BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "Vulkan".to_string(),
            acceleration: RuntimeAcceleration::Vulkan,
            note: "stable-diffusion.cpp is built with Vulkan acceleration for realism jobs.".to_string(),
        },
        Some(SdcppBuildMode::CpuOnly) => BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "CPU-only".to_string(),
            acceleration: RuntimeAcceleration::CpuOnly,
            note: "stable-diffusion.cpp is currently built without Vulkan. Large Wan video jobs can take a very long time.".to_string(),
        },
        None if build_exists && has_vulkan_sdk() => BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "Build pending".to_string(),
            acceleration: RuntimeAcceleration::BuildPending,
            note: "A stable-diffusion.cpp binary exists, but Chatty-art could not confirm whether it was built with Vulkan. The next realism run may rebuild it.".to_string(),
        },
        None if has_vulkan_sdk() => BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "Build pending".to_string(),
            acceleration: RuntimeAcceleration::BuildPending,
            note: "sd-cli is not built yet. The first realism run should build stable-diffusion.cpp with Vulkan.".to_string(),
        },
        None => BackendRuntimeStatus {
            backend: ModelBackend::StableDiffusionCpp,
            label: "Build pending".to_string(),
            acceleration: RuntimeAcceleration::BuildPending,
            note: "sd-cli is not built yet. No Vulkan SDK was detected, so the first realism build will be CPU-only unless you install Vulkan first.".to_string(),
        },
    }
}

pub async fn generate_with_sdcpp(
    diffuse_runtime_dir: &Path,
    models_dir: &Path,
    input_dir: &Path,
    model: &ModelInfo,
    request: &GenerateRequest,
    reference_asset: Option<&InputAsset>,
    end_reference_asset: Option<&InputAsset>,
    control_reference_asset: Option<&InputAsset>,
    seed: u32,
    output_path: &Path,
) -> Result<SdcppGeneration> {
    let model_path = models_dir.join(native_relative_path(&model.relative_path));
    let recipe = detect_runtime_recipe(
        diffuse_runtime_dir,
        models_dir,
        &model_path,
        &model.file_name,
    )
    .ok_or_else(|| {
        anyhow!("This model is not wired to the local stable-diffusion.cpp backend yet.")
    })?;

    if !recipe.runtime_supported() {
        bail!("{}", recipe.compatibility_note);
    }

    let cli_path = ensure_sd_cli(diffuse_runtime_dir).await?;
    let mut args = build_base_sdcpp_args(&recipe, request, seed)?;
    let mut temp_dirs = Vec::new();

    if let Some(reference_asset) = reference_asset {
        let reference_path = input_dir.join(native_relative_path(&reference_asset.relative_path));
        if reference_asset.kind != MediaKind::Image {
            bail!("This stable-diffusion.cpp path only supports image references right now.");
        }

        match recipe.reference_mode {
            Some(ReferenceMode::InitImage) => {
                args.push("--init-img".to_string());
                args.push(reference_path.display().to_string());
                args.push("--strength".to_string());
                args.push(init_image_strength(request.reference_intent).to_string());
            }
            Some(ReferenceMode::RefImage) => {
                args.push("-r".to_string());
                args.push(reference_path.display().to_string());
            }
            None => {
                bail!(
                    "This {} path does not use reference media in Chatty-art yet.",
                    recipe.family_label()
                );
            }
        }
    } else if recipe.requires_reference {
        bail!("This model expects an image reference. Pick one in the Input Tray and try again.");
    }

    if let Some(end_reference_asset) = end_reference_asset {
        let end_reference_path =
            input_dir.join(native_relative_path(&end_reference_asset.relative_path));
        if end_reference_asset.kind != MediaKind::Image {
            bail!("This stable-diffusion.cpp path expects the end frame to be a still image.");
        }
        if !recipe.supports_end_image_reference && !recipe.requires_end_image_reference {
            bail!(
                "This {} path does not use an end image in Chatty-art yet.",
                recipe.family_label()
            );
        }
        args.push("--end-img".to_string());
        args.push(end_reference_path.display().to_string());
    } else if recipe.requires_end_image_reference {
        bail!(
            "This model needs both a start image and an end image. Pick the end frame in the Input Tray and try again."
        );
    }

    if let Some(control_reference_asset) = control_reference_asset {
        if !recipe.supports_video_reference {
            bail!(
                "This {} path does not use control-video input in Chatty-art yet.",
                recipe.family_label()
            );
        }

        let control_reference_path =
            input_dir.join(native_relative_path(&control_reference_asset.relative_path));
        let control_frames_dir =
            prepare_control_video_frames(&control_reference_path, output_path).await?;
        temp_dirs.push(control_frames_dir.clone());
        args.push("--control-video".to_string());
        args.push(control_frames_dir.display().to_string());
    }

    let generation_result = async {
        match request.kind {
            MediaKind::Image => {
                args.push("-o".to_string());
                args.push(output_path.display().to_string());
                run_sd_cli(&cli_path, diffuse_runtime_dir, &args).await?;

                if !output_path.exists() {
                    bail!(
                        "stable-diffusion.cpp finished without creating {}.",
                        output_path.display()
                    );
                }

                Ok(SdcppGeneration {
                    mime: request.kind.output_mime().to_string(),
                    note: recipe.generation_note_for(request.kind),
                })
            }
            MediaKind::Gif => {
                let frame_dir = output_path.with_extension("frames");
                if frame_dir.exists() {
                    let _ = fs::remove_dir_all(&frame_dir).await;
                }
                fs::create_dir_all(&frame_dir).await?;
                let frame_pattern = frame_dir.join("frame_%03d.png");
                args.push("-o".to_string());
                args.push(frame_pattern.display().to_string());

                let gif_result = async {
                    run_sd_cli(&cli_path, diffuse_runtime_dir, &args).await?;
                    encode_png_sequence_to_gif(
                        &frame_dir,
                        output_path,
                        request.settings.video_fps,
                    )?;
                    Ok(SdcppGeneration {
                        mime: "image/gif".to_string(),
                        note: recipe.generation_note_for(MediaKind::Gif),
                    })
                }
                .await;
                let _ = fs::remove_dir_all(&frame_dir).await;
                gif_result
            }
            MediaKind::Video => {
                args.push("-o".to_string());
                args.push(output_path.display().to_string());
                run_sd_cli(&cli_path, diffuse_runtime_dir, &args).await?;

                normalize_sdcpp_video_output(output_path)?;

                Ok(SdcppGeneration {
                    mime: request.kind.output_mime().to_string(),
                    note: recipe.generation_note_for(request.kind),
                })
            }
            MediaKind::Audio => bail!("stable-diffusion.cpp does not generate audio"),
        }
    }
    .await;

    for temp_dir in temp_dirs {
        let _ = fs::remove_dir_all(temp_dir).await;
    }

    generation_result
}

fn build_base_sdcpp_args(
    recipe: &RuntimeRecipe,
    request: &GenerateRequest,
    seed: u32,
) -> Result<Vec<String>> {
    let (width, height) = request.settings.dimensions_for(request.kind);
    let negative_prompt = request
        .negative_prompt
        .as_deref()
        .unwrap_or("low quality, blurry, distorted");

    let mut args = vec![
        primary_model_flag(&recipe.family).to_string(),
        recipe.model_path.display().to_string(),
        "-p".to_string(),
        request.prompt.clone(),
        "-n".to_string(),
        negative_prompt.to_string(),
        "--steps".to_string(),
        request.settings.steps.to_string(),
        "--cfg-scale".to_string(),
        request.settings.cfg_scale.to_string(),
        "-s".to_string(),
        seed.to_string(),
        "-W".to_string(),
        width.to_string(),
        "-H".to_string(),
        height.to_string(),
        "--sampling-method".to_string(),
        "euler".to_string(),
    ];

    if let Some(path) = &recipe.vae_path {
        args.push("--vae".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.llm_path {
        args.push("--llm".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.llm_vision_path {
        args.push("--llm_vision".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.t5_path {
        args.push("--t5xxl".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.clip_l_path {
        args.push("--clip_l".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.clip_g_path {
        args.push("--clip_g".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.clip_vision_path {
        args.push("--clip_vision".to_string());
        args.push(path.display().to_string());
    }
    if let Some(path) = &recipe.high_noise_model_path {
        args.push("--high-noise-diffusion-model".to_string());
        args.push(path.display().to_string());
        args.push("--high-noise-cfg-scale".to_string());
        args.push(request.settings.cfg_scale.to_string());
        args.push("--high-noise-sampling-method".to_string());
        args.push("euler".to_string());
        args.push("--high-noise-steps".to_string());
        args.push(request.settings.steps.max(4).to_string());
    }

    match recipe.family {
        RuntimeFamily::StableDiffusion
        | RuntimeFamily::StableDiffusion3
        | RuntimeFamily::Flux
        | RuntimeFamily::FluxKontext
        | RuntimeFamily::ZImage
        | RuntimeFamily::OvisImage
        | RuntimeFamily::Anima => {}
        RuntimeFamily::QwenImage => {
            args.push("--flow-shift".to_string());
            args.push("3".to_string());
        }
        RuntimeFamily::QwenImageEdit => {
            args.push("--flow-shift".to_string());
            args.push("3".to_string());
        }
        RuntimeFamily::Wan => {
            args.push("-M".to_string());
            args.push("vid_gen".to_string());
            args.push("--video-frames".to_string());
            args.push(match request.kind {
                MediaKind::Image => "1".to_string(),
                MediaKind::Gif | MediaKind::Video => {
                    request.settings.video_frame_count().to_string()
                }
                MediaKind::Audio => bail!("stable-diffusion.cpp does not generate audio"),
            });
            args.push("--fps".to_string());
            args.push(request.settings.video_fps.to_string());
            args.push("--flow-shift".to_string());
            args.push("3.0".to_string());
            args.push("--clip-on-cpu".to_string());
        }
        RuntimeFamily::Ltx => {
            bail!("LTX is not wired to Chatty-art's stable-diffusion.cpp launcher yet.");
        }
        RuntimeFamily::StandaloneDiffusion => {
            bail!(
                "This diffusion GGUF is detected, but Chatty-art does not yet know its stable-diffusion.cpp launch recipe."
            );
        }
    }

    args.extend(recipe.extra_args.iter().cloned());
    apply_low_vram_tuning(recipe, request, width, height, &mut args);

    Ok(args)
}

fn apply_low_vram_tuning(
    recipe: &RuntimeRecipe,
    request: &GenerateRequest,
    width: u32,
    height: u32,
    args: &mut Vec<String>,
) {
    let high_memory_job =
        matches!(request.kind, MediaKind::Gif | MediaKind::Video) || width.max(height) > 512;
    let low_vram_mode = request.settings.low_vram_mode;

    if high_memory_job || low_vram_mode {
        push_flag_once(args, "--offload-to-cpu");
    }

    if recipe.vae_path.is_some() && (high_memory_job || low_vram_mode) {
        push_flag_once(args, "--vae-tiling");
        push_option_once(
            args,
            "--vae-relative-tile-size",
            vae_tile_grid_for_request(request, width, height, low_vram_mode),
        );
    }

    if recipe.vae_path.is_some()
        && (low_vram_mode || matches!(recipe.family, RuntimeFamily::Wan) && high_memory_job)
    {
        push_flag_once(args, "--vae-on-cpu");
    }
}

fn vae_tile_grid_for_request(
    request: &GenerateRequest,
    width: u32,
    height: u32,
    low_vram_mode: bool,
) -> &'static str {
    if matches!(request.kind, MediaKind::Gif | MediaKind::Video) {
        if low_vram_mode && (request.settings.video_frame_count() > 40 || width.max(height) >= 768)
        {
            "4x4"
        } else if low_vram_mode
            || request.settings.video_frame_count() > 40
            || width.max(height) >= 768
        {
            "3x3"
        } else {
            "2x2"
        }
    } else if low_vram_mode && width.max(height) >= 768 {
        "4x4"
    } else if low_vram_mode || width.max(height) >= 1024 {
        "3x3"
    } else {
        "2x2"
    }
}

fn push_flag_once(args: &mut Vec<String>, flag: &str) {
    if !args.iter().any(|value| value == flag) {
        args.push(flag.to_string());
    }
}

fn push_option_once(args: &mut Vec<String>, flag: &str, value: &str) {
    if !args.windows(2).any(|pair| pair[0] == flag) {
        args.push(flag.to_string());
        args.push(value.to_string());
    }
}

fn normalize_sdcpp_video_output(output_path: &Path) -> Result<()> {
    if output_path.exists() {
        return Ok(());
    }

    let appended_avi = PathBuf::from(format!("{}.avi", output_path.display()));
    if appended_avi.exists() {
        std::fs::rename(&appended_avi, output_path).with_context(|| {
            format!(
                "stable-diffusion.cpp wrote {}, but Chatty-art could not rename it to {}",
                appended_avi.display(),
                output_path.display()
            )
        })?;
        return Ok(());
    }

    bail!(
        "stable-diffusion.cpp finished without creating {}.",
        output_path.display()
    );
}

fn init_image_strength(intent: ReferenceIntent) -> &'static str {
    match intent {
        ReferenceIntent::Guide => "0.8",
        ReferenceIntent::Edit => "0.45",
    }
}

async fn prepare_control_video_frames(
    control_reference_path: &Path,
    output_path: &Path,
) -> Result<PathBuf> {
    let control_dir = output_path.with_extension("control_frames");
    if control_dir.exists() {
        let _ = fs::remove_dir_all(&control_dir).await;
    }
    fs::create_dir_all(&control_dir).await?;

    let lower = control_reference_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let extraction = async {
        match lower.as_str() {
            "gif" => {
                decode_gif_to_png_frames(control_reference_path, &control_dir)?;
            }
            "avi" | "mp4" | "webm" | "mov" | "mkv" => {
                extract_video_frames_with_ffmpeg(control_reference_path, &control_dir).await?;
            }
            _ => {
                bail!(
                    "Control video '{}' is not a supported video source yet. Use a GIF, MP4, AVI, WEBM, MOV, or MKV file in input/video/.",
                    control_reference_path.display()
                );
            }
        }

        Ok::<(), anyhow::Error>(())
    }
    .await;

    if let Err(error) = extraction {
        let _ = fs::remove_dir_all(&control_dir).await;
        return Err(error);
    }

    Ok(control_dir)
}

fn decode_gif_to_png_frames(gif_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(gif_path)
        .with_context(|| format!("failed to open GIF control video {}", gif_path.display()))?;
    let decoder = GifDecoder::new(BufReader::new(file))
        .with_context(|| format!("failed to decode GIF control video {}", gif_path.display()))?;
    let frames = decoder
        .into_frames()
        .collect_frames()
        .with_context(|| format!("failed to read GIF frames from {}", gif_path.display()))?;

    if frames.is_empty() {
        bail!(
            "GIF control video '{}' did not contain any frames.",
            gif_path.display()
        );
    }

    for (index, frame) in frames.into_iter().enumerate() {
        let path = output_dir.join(format!("frame_{index:04}.png"));
        frame
            .into_buffer()
            .save(&path)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    Ok(())
}

async fn extract_video_frames_with_ffmpeg(video_path: &Path, output_dir: &Path) -> Result<()> {
    let frame_pattern = output_dir.join("frame_%04d.png");
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(video_path)
        .arg("-qscale:v")
        .arg("1")
        .arg("-vf")
        .arg(format!("fps={DEFAULT_VIDEO_FPS}"))
        .arg(&frame_pattern)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| {
            format!(
                "failed to launch ffmpeg for control video extraction from {}",
                video_path.display()
            )
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Chatty-art could not unpack '{}' into control-video frames. Install ffmpeg or use a GIF in input/video/.\n{}\n{}",
            video_path.display(),
            summarize_output("stdout", &stdout),
            summarize_output("stderr", &stderr)
        );
    }

    let frame_count = std::fs::read_dir(output_dir)
        .with_context(|| {
            format!(
                "failed to read extracted control frames in {}",
                output_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        })
        .count();
    if frame_count == 0 {
        bail!(
            "ffmpeg ran for '{}', but no control-video frames were written into {}.",
            video_path.display(),
            output_dir.display()
        );
    }

    Ok(())
}

async fn ensure_sd_cli(diffuse_runtime_dir: &Path) -> Result<PathBuf> {
    if let Some(existing) = find_sd_cli(diffuse_runtime_dir) {
        let wants_vulkan = has_vulkan_sdk();
        let built_with_vulkan = matches!(
            detect_sdcpp_build_mode(diffuse_runtime_dir),
            Some(SdcppBuildMode::Vulkan)
        );
        if !wants_vulkan || built_with_vulkan {
            return Ok(existing);
        }
    }

    if !diffuse_runtime_dir.join("CMakeLists.txt").exists() {
        bail!(
            "stable-diffusion.cpp source was not found in diffuse_runtime/. Add the extracted source tree there first."
        );
    }

    if !diffuse_runtime_dir
        .join("ggml")
        .join("CMakeLists.txt")
        .exists()
    {
        bail!(
            "diffuse_runtime/ggml is missing. The stable-diffusion.cpp source tree needs the ggml submodule too."
        );
    }

    let build_dir = diffuse_runtime_dir.join(BUILD_DIR_NAME);
    let mut configure_args = vec![
        "-S".to_string(),
        diffuse_runtime_dir.display().to_string(),
        "-B".to_string(),
        build_dir.display().to_string(),
        "-DSD_BUILD_EXAMPLES=ON".to_string(),
    ];

    if has_vulkan_sdk() {
        configure_args.push("-DSD_VULKAN=ON".to_string());
    }

    run_build_command("cmake", &configure_args).await?;
    run_build_command(
        "cmake",
        &[
            "--build".to_string(),
            build_dir.display().to_string(),
            "--config".to_string(),
            "Release".to_string(),
            "--target".to_string(),
            "sd-cli".to_string(),
        ],
    )
    .await?;

    find_sd_cli(diffuse_runtime_dir).ok_or_else(|| {
        anyhow!(
            "stable-diffusion.cpp built without leaving behind sd-cli.exe in the expected build folder."
        )
    })
}

async fn run_sd_cli(cli_path: &Path, diffuse_runtime_dir: &Path, args: &[String]) -> Result<()> {
    let output = Command::new(cli_path)
        .args(args)
        .current_dir(diffuse_runtime_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| format!("failed to launch {}", cli_path.display()))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let friendly = explain_sdcpp_failure(&stdout, &stderr);
    bail!(
        "{}stable-diffusion.cpp failed.\n{}\n{}",
        friendly
            .map(|message| format!("{message}\n\n"))
            .unwrap_or_default(),
        summarize_output("stdout", &stdout),
        summarize_output("stderr", &stderr)
    );
}

async fn run_build_command(program: &str, args: &[String]) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| format!("failed to launch {program}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!(
        "{} failed.\n{}\n{}",
        program,
        summarize_output("stdout", &stdout),
        summarize_output("stderr", &stderr)
    );
}

fn find_sd_cli(diffuse_runtime_dir: &Path) -> Option<PathBuf> {
    let build_dir = diffuse_runtime_dir.join(BUILD_DIR_NAME);
    let candidates = [
        build_dir.join("bin").join("Release").join("sd-cli.exe"),
        build_dir.join("bin").join("sd-cli.exe"),
        build_dir.join("Release").join("sd-cli.exe"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

fn primary_model_flag(family: &RuntimeFamily) -> &'static str {
    match family {
        // Self-contained Stable Diffusion GGUF checkpoints go through the full-model path.
        RuntimeFamily::StableDiffusion => "-m",
        RuntimeFamily::StableDiffusion3 => "-m",
        RuntimeFamily::Flux
        | RuntimeFamily::FluxKontext
        | RuntimeFamily::QwenImage
        | RuntimeFamily::QwenImageEdit
        | RuntimeFamily::Wan
        | RuntimeFamily::ZImage
        | RuntimeFamily::OvisImage
        | RuntimeFamily::Anima
        | RuntimeFamily::Ltx
        | RuntimeFamily::StandaloneDiffusion => "--diffusion-model",
    }
}

fn base_recipe(
    diffuse_runtime_dir: &Path,
    family: RuntimeFamily,
    model_path: &Path,
) -> RuntimeRecipe {
    RuntimeRecipe {
        family,
        runtime_tree_ready: runtime_tree_ready(diffuse_runtime_dir),
        model_path: model_path.to_path_buf(),
        vae_path: None,
        llm_path: None,
        llm_vision_path: None,
        t5_path: None,
        clip_l_path: None,
        clip_g_path: None,
        clip_vision_path: None,
        high_noise_model_path: None,
        missing: Vec::new(),
        supported_kinds: vec![MediaKind::Image, MediaKind::Gif],
        requires_reference: false,
        supports_image_reference: false,
        requires_end_image_reference: false,
        supports_end_image_reference: false,
        supports_video_reference: false,
        compatibility_note: String::new(),
        reference_mode: None,
        extra_args: Vec::new(),
    }
}

fn detect_runtime_recipe(
    diffuse_runtime_dir: &Path,
    models_dir: &Path,
    model_path: &Path,
    file_name: &str,
) -> Option<RuntimeRecipe> {
    let lower = file_name.to_ascii_lowercase();
    let gguf = inspect_gguf(model_path);

    if is_unsupported_motion_diffusion(&lower, gguf.as_ref()) {
        let missing = vec!["This checkpoint looks like an animation or motion-diffusion variant that Chatty-art's current stable-diffusion.cpp path does not support yet.".to_string()];
        let mut recipe = base_recipe(
            diffuse_runtime_dir,
            RuntimeFamily::StandaloneDiffusion,
            model_path,
        );
        recipe.missing = missing.clone();
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Standalone Diffusion GGUF",
            "This diffusion GGUF includes animation or motion tensors. Chatty-art's current stable-diffusion.cpp launcher only supports standard 2D diffusion and the explicitly wired families here.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_sd3_family(&lower, gguf.as_ref()) {
        let clip_l = find_first_file(
            models_dir,
            &["clip_l", "clip-l", "clipl"],
            &["safetensors", "gguf"],
        );
        let clip_g = find_first_file(
            models_dir,
            &["clip_g", "clip-g", "clipg"],
            &["safetensors", "gguf"],
        );
        let t5 = find_first_file(
            models_dir,
            &["t5xxl", "t5_xxl", "t5-xxl"],
            &["gguf", "safetensors"],
        );
        let has_sd3_layout = sd3_has_checkpoint_layout(gguf.as_ref());
        let has_any_encoder = sd3_has_any_text_encoder(
            gguf.as_ref(),
            clip_l.as_deref(),
            clip_g.as_deref(),
            t5.as_deref(),
        );
        let mut missing = Vec::new();
        if !has_sd3_layout {
            missing.push(
                "an SD3-compatible GGUF checkpoint layout (for example joint_blocks tensors)"
                    .to_string(),
            );
        }
        if !has_any_encoder {
            missing.push("at least one SD3 text encoder (clip_l, clip_g, or t5xxl)".to_string());
        }
        let mut recipe = base_recipe(
            diffuse_runtime_dir,
            RuntimeFamily::StableDiffusion3,
            model_path,
        );
        recipe.clip_l_path = clip_l;
        recipe.clip_g_path = clip_g;
        recipe.t5_path = t5;
        recipe.extra_args = vec!["--clip-on-cpu".to_string()];
        recipe.missing = missing.clone();
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "SD3 / SD3.5 GGUF",
            "SD3 / SD3.5 runs locally through stable-diffusion.cpp. Chatty-art auto-passes any clip_l, clip_g, and t5xxl companion encoders it finds in models/.",
            &missing,
        );
        if missing.is_empty() {
            let recommended = sd3_recommended_components(
                gguf.as_ref(),
                recipe.clip_l_path.as_deref(),
                recipe.clip_g_path.as_deref(),
                recipe.t5_path.as_deref(),
            );
            if !recommended.is_empty() {
                recipe
                    .compatibility_note
                    .push_str(" Recommended for stronger prompt adherence: ");
                recipe.compatibility_note.push_str(&recommended.join(", "));
                recipe.compatibility_note.push('.');
            }
        }
        return Some(recipe);
    }

    if matches_self_contained_diffusion(&lower, gguf.as_ref()) {
        let mut recipe = base_recipe(
            diffuse_runtime_dir,
            RuntimeFamily::StableDiffusion,
            model_path,
        );
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::InitImage);
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Self-contained Diffusion GGUF",
            "Self-contained diffusion GGUF generation runs locally through stable-diffusion.cpp. You can also pick an image in the Input Tray to guide or edit the render, and image-first models can be exported as GIF loops.",
            &[],
        );
        return Some(recipe);
    }

    if matches_flux_kontext(&lower, gguf.as_ref()) {
        let vae = find_first_file(models_dir, &["ae"], &["sft", "safetensors"]);
        let clip_l = find_first_file(models_dir, &["clip_l"], &["safetensors", "gguf"]);
        let t5 = find_first_file(models_dir, &["t5xxl"], &["gguf", "safetensors"]);
        let missing = missing_components(&[
            ("ae.sft / ae.safetensors", vae.as_deref()),
            ("clip_l.safetensors", clip_l.as_deref()),
            ("t5xxl text encoder (.gguf or .safetensors)", t5.as_deref()),
        ]);
        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::FluxKontext, model_path);
        recipe.vae_path = vae;
        recipe.clip_l_path = clip_l;
        recipe.t5_path = t5;
        recipe.missing = missing.clone();
        recipe.requires_reference = true;
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::RefImage);
        recipe.extra_args = vec!["--clip-on-cpu".to_string(), "--diffusion-fa".to_string()];
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "FLUX Kontext GGUF",
            "FLUX Kontext runs locally through stable-diffusion.cpp. Add ae, clip_l, and t5xxl companion weights into models/, then pick an image in the Input Tray as the edit reference.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_flux(&lower, gguf.as_ref()) {
        let vae = find_first_file(models_dir, &["ae"], &["sft", "safetensors"]);
        let clip_l = find_first_file(models_dir, &["clip_l"], &["safetensors", "gguf"]);
        let t5 = find_first_file(models_dir, &["t5xxl"], &["gguf", "safetensors"]);
        let missing = missing_components(&[
            ("ae.sft / ae.safetensors", vae.as_deref()),
            ("clip_l.safetensors", clip_l.as_deref()),
            ("t5xxl text encoder (.gguf or .safetensors)", t5.as_deref()),
        ]);
        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::Flux, model_path);
        recipe.vae_path = vae;
        recipe.clip_l_path = clip_l;
        recipe.t5_path = t5;
        recipe.missing = missing.clone();
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::InitImage);
        recipe.extra_args = vec!["--clip-on-cpu".to_string(), "--diffusion-fa".to_string()];
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "FLUX GGUF",
            "FLUX runs locally through stable-diffusion.cpp. Add ae, clip_l, and t5xxl companion weights into models/ to enable it. Chatty-art can also use a selected image as a guide or edit source on the init-image path.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_qwen_image_edit(&lower, gguf.as_ref()) {
        let vae = find_first_file(models_dir, &["qwen_image_vae"], &["safetensors"]);
        let llm = find_first_file(
            models_dir,
            &["qwen2.5-vl", "qwen_2.5_vl", "qwen2vl", "qwenvl"],
            &["gguf", "safetensors"],
        );
        let llm_vision = if lower.contains("2509") {
            find_first_file_with_all_fragments(models_dir, &["qwen2.5-vl", "mmproj"], &["gguf"])
                .or_else(|| find_first_file(models_dir, &["mmproj"], &["gguf"]))
        } else {
            None
        };
        let missing = missing_components(&[
            ("qwen_image_vae.safetensors", vae.as_deref()),
            (
                "Qwen2.5-VL-7B text encoder (.gguf or .safetensors)",
                llm.as_deref(),
            ),
            (
                "Qwen2.5-VL mmproj GGUF",
                if lower.contains("2509") {
                    llm_vision.as_deref()
                } else {
                    Some(model_path)
                },
            ),
        ]);
        let mut recipe = base_recipe(
            diffuse_runtime_dir,
            RuntimeFamily::QwenImageEdit,
            model_path,
        );
        recipe.vae_path = vae;
        recipe.llm_path = llm;
        recipe.llm_vision_path = llm_vision;
        recipe.missing = missing.clone();
        recipe.requires_reference = true;
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::RefImage);
        recipe.extra_args = vec![
            "--offload-to-cpu".to_string(),
            "--diffusion-fa".to_string(),
            if lower.contains("2511") {
                "--qwen-image-zero-cond-t".to_string()
            } else {
                String::new()
            },
        ]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect();
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Qwen Image Edit GGUF",
            "Qwen Image Edit runs locally through stable-diffusion.cpp. Add the Qwen VAE and Qwen2.5-VL text encoder into models/. Qwen Image Edit uses the selected image as a local edit reference.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_qwen_image(&lower, gguf.as_ref()) {
        let vae = find_first_file(models_dir, &["qwen_image_vae"], &["safetensors"]);
        let llm = find_first_file(
            models_dir,
            &["qwen2.5-vl", "qwen_2.5_vl", "qwen2vl", "qwenvl"],
            &["gguf", "safetensors"],
        );
        let missing = missing_components(&[
            ("qwen_image_vae.safetensors", vae.as_deref()),
            (
                "Qwen2.5-VL-7B text encoder (.gguf or .safetensors)",
                llm.as_deref(),
            ),
        ]);

        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::QwenImage, model_path);
        recipe.vae_path = vae;
        recipe.llm_path = llm;
        recipe.missing = missing.clone();
        recipe.extra_args = vec!["--offload-to-cpu".to_string(), "--diffusion-fa".to_string()];
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Qwen Image GGUF",
            "Qwen Image runs locally through stable-diffusion.cpp. Add the Qwen VAE and Qwen2.5-VL text encoder into models/ to enable it. Chatty-art can also pack image-first renders into GIF video clips.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_z_image(&lower, gguf.as_ref()) {
        let vae = find_first_file(models_dir, &["ae"], &["sft", "safetensors"]);
        let llm = find_first_file(
            models_dir,
            &["qwen3-4b", "qwen_3_4b", "qwen3 4b"],
            &["gguf", "safetensors"],
        );
        let missing = missing_components(&[
            ("ae.sft / ae.safetensors", vae.as_deref()),
            ("Qwen3-4B encoder (.gguf or .safetensors)", llm.as_deref()),
        ]);
        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::ZImage, model_path);
        recipe.vae_path = vae;
        recipe.llm_path = llm;
        recipe.missing = missing.clone();
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::InitImage);
        recipe.extra_args = vec!["--offload-to-cpu".to_string(), "--diffusion-fa".to_string()];
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Z-Image GGUF",
            "Z-Image runs locally through stable-diffusion.cpp. Add ae and a Qwen3-4B encoder into models/ to enable it. Chatty-art can also use a selected image as a guide or edit source on the init-image path.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_ovis_image(&lower, gguf.as_ref()) {
        let vae = find_first_file(models_dir, &["ae"], &["sft", "safetensors"]);
        let llm = find_first_file(
            models_dir,
            &["ovis_2.5", "ovis2.5"],
            &["gguf", "safetensors"],
        );
        let missing = missing_components(&[
            ("ae.sft / ae.safetensors", vae.as_deref()),
            ("Ovis 2.5 encoder (.gguf or .safetensors)", llm.as_deref()),
        ]);
        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::OvisImage, model_path);
        recipe.vae_path = vae;
        recipe.llm_path = llm;
        recipe.missing = missing.clone();
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::InitImage);
        recipe.extra_args = vec!["--offload-to-cpu".to_string(), "--diffusion-fa".to_string()];
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Ovis Image GGUF",
            "Ovis Image runs locally through stable-diffusion.cpp. Add ae and an Ovis 2.5 encoder into models/ to enable it. Chatty-art can also use a selected image as a guide or edit source on the init-image path.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_anima(&lower, gguf.as_ref()) {
        let vae = find_first_file(
            models_dir,
            &["anima_vae", "qwen_image_vae", "anima"],
            &["safetensors"],
        );
        let llm = find_first_file(
            models_dir,
            &["qwen3-0.6b-base", "qwen_3_06b_base", "qwen3-06b-base"],
            &["gguf", "safetensors"],
        );
        let missing = missing_components(&[
            ("Anima VAE / qwen_image_vae.safetensors", vae.as_deref()),
            (
                "Qwen3-0.6B-Base encoder (.gguf or .safetensors)",
                llm.as_deref(),
            ),
        ]);
        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::Anima, model_path);
        recipe.vae_path = vae;
        recipe.llm_path = llm;
        recipe.missing = missing.clone();
        recipe.supports_image_reference = true;
        recipe.reference_mode = Some(ReferenceMode::InitImage);
        recipe.extra_args = vec!["--offload-to-cpu".to_string(), "--diffusion-fa".to_string()];
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Anima GGUF",
            "Anima runs locally through stable-diffusion.cpp. Add the Anima VAE and Qwen3-0.6B-Base encoder into models/ to enable it. Chatty-art can also use a selected image as a guide or edit source on the init-image path.",
            &missing,
        );
        return Some(recipe);
    }

    if matches_wan(&lower, gguf.as_ref()) {
        let variant = detect_wan_variant(&lower, gguf.as_ref());
        if is_unsupported_wan_variant(gguf.as_ref()) {
            let missing = vec![
                "This Wan GGUF includes tensor groups that Chatty-art's current stable-diffusion.cpp launcher still cannot route safely.".to_string(),
            ];
            let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::Wan, model_path);
            recipe.missing = missing.clone();
            recipe.compatibility_note = recipe_note(
                diffuse_runtime_dir,
                "Wan GGUF",
                "This Wan file was recognized, but its tensor layout looks more exotic than the currently wired Wan T2V, I2V, TI2V, or VACE paths.",
                &missing,
            );
            return Some(recipe);
        }

        let variant_label = wan_variant_label(variant);
        if matches!(variant, WanVariant::Vace) && !wan_vace_has_expected_layout(gguf.as_ref()) {
            let missing = vec![
                "This VACE GGUF conversion does not include the VACE patch-embedding tensors that the current stable-diffusion.cpp VACE path expects.".to_string(),
            ];
            let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::Wan, model_path);
            recipe.missing = missing.clone();
            recipe.supported_kinds = vec![MediaKind::Image, MediaKind::Gif, MediaKind::Video];
            recipe.supports_image_reference = true;
            recipe.supports_video_reference = true;
            recipe.compatibility_note = recipe_note(
                diffuse_runtime_dir,
                "Wan GGUF",
                "This Wan VACE file was recognized, but its GGUF conversion does not match the VACE tensor layout expected by the current stable-diffusion.cpp runtime. Try a different VACE conversion or the official safetensors release for this family.",
                &missing,
            );
            return Some(recipe);
        }

        let needs_clip_vision = wan_needs_clip_vision(variant);
        let variant_requires_reference = wan_requires_image_reference(variant);
        let supports_image_reference = wan_supports_image_reference(variant);
        let variant_requires_end_image = wan_requires_end_image_reference(variant);
        let supports_end_image_reference = wan_supports_end_image_reference(variant);
        let supports_video_reference = wan_supports_video_reference(variant);
        let (resolved_model_path, high_noise, pair_note) =
            resolve_wan_model_paths(models_dir, model_path, &lower);

        let vae = if matches!(variant, WanVariant::Ti2v) {
            find_first_file(models_dir, &["wan2.2_vae"], &["safetensors"])
                .or_else(|| find_first_file(models_dir, &["wan_2.1_vae"], &["safetensors"]))
        } else {
            find_first_file(models_dir, &["wan_2.1_vae", "wan2.2_vae"], &["safetensors"])
        };
        let t5 = find_first_file(models_dir, &["umt5"], &["gguf", "safetensors"])
            .or_else(|| find_first_file(models_dir, &["t5xxl"], &["gguf", "safetensors"]));
        let clip_vision = if needs_clip_vision {
            find_first_file(
                models_dir,
                &["clip_vision_h", "clip-vision-h"],
                &["safetensors"],
            )
        } else {
            None
        };
        let missing = missing_components(&[
            ("wan vae (.safetensors)", vae.as_deref()),
            (
                "umt5 / t5xxl text encoder (.gguf or .safetensors)",
                t5.as_deref(),
            ),
            (
                "clip_vision_h.safetensors",
                if needs_clip_vision {
                    clip_vision.as_deref()
                } else {
                    Some(resolved_model_path.as_path())
                },
            ),
            (
                "HighNoise / LowNoise Wan2.2 GGUF pair",
                if wan_needs_high_noise_pair(&lower, variant) {
                    high_noise.as_deref()
                } else {
                    Some(resolved_model_path.as_path())
                },
            ),
        ]);

        let mut note = format!(
            "{} runs locally through stable-diffusion.cpp for image and video jobs. Add the Wan VAE and the UMT5/T5 text encoder into models/ to enable it.",
            variant_label
        );
        if matches!(variant, WanVariant::Flf2v) {
            note.push_str(" FLF2V uses both a start image and an end image from the Input Tray.");
        }
        if matches!(variant, WanVariant::Vace) {
            note.push_str(
                " Wan VACE can run text-to-video, image-guided video, and control-video guided passes here.",
            );
        }
        if needs_clip_vision {
            note.push_str(" Wan image-conditioned variants also need clip_vision_h.safetensors.");
        }
        if wan_needs_high_noise_pair(&lower, variant) {
            note.push_str(
                " Wan2.2 A14B-style models also expect matching HighNoise and LowNoise diffusion files.",
            );
        }
        if let Some(pair_note) = pair_note.as_deref() {
            note.push(' ');
            note.push_str(pair_note);
        }

        let mut recipe = base_recipe(
            diffuse_runtime_dir,
            RuntimeFamily::Wan,
            &resolved_model_path,
        );
        recipe.vae_path = vae;
        recipe.t5_path = t5;
        recipe.clip_vision_path = clip_vision;
        recipe.high_noise_model_path = high_noise;
        recipe.missing = missing.clone();
        recipe.supported_kinds = vec![MediaKind::Image, MediaKind::Gif, MediaKind::Video];
        recipe.requires_reference = variant_requires_reference;
        recipe.supports_image_reference = supports_image_reference;
        recipe.requires_end_image_reference = variant_requires_end_image;
        recipe.supports_end_image_reference = supports_end_image_reference;
        recipe.supports_video_reference = supports_video_reference;
        recipe.reference_mode = if recipe.supports_image_reference {
            Some(ReferenceMode::InitImage)
        } else {
            None
        };
        recipe.extra_args = vec!["--diffusion-fa".to_string(), "--offload-to-cpu".to_string()];
        recipe.compatibility_note = recipe_note(diffuse_runtime_dir, "Wan GGUF", &note, &missing);
        return Some(recipe);
    }

    if lower.contains("ltx")
        || gguf
            .as_ref()
            .and_then(|summary| summary.architecture())
            .is_some_and(|architecture| architecture == "ltxv")
    {
        let mut recipe = base_recipe(diffuse_runtime_dir, RuntimeFamily::Ltx, model_path);
        recipe.missing = vec!["Chatty-art does not have an LTX command path yet".to_string()];
        recipe.supports_image_reference = true;
        recipe.compatibility_note =
            "stable-diffusion.cpp includes LTX internals, but Chatty-art has not wired an LTX launcher yet.".to_string();
        return Some(recipe);
    }

    if looks_like_standalone_diffusion(gguf.as_ref()) {
        let missing = vec![
            "Chatty-art detected a diffusion GGUF, but does not yet know the companion weights or launch flags it needs".to_string(),
        ];
        let mut recipe = base_recipe(
            diffuse_runtime_dir,
            RuntimeFamily::StandaloneDiffusion,
            model_path,
        );
        recipe.missing = missing.clone();
        recipe.compatibility_note = recipe_note(
            diffuse_runtime_dir,
            "Standalone Diffusion GGUF",
            "This looks like a diffusion-style GGUF, but Chatty-art does not yet know its exact stable-diffusion.cpp wiring.",
            &missing,
        );
        return Some(recipe);
    }

    None
}

impl RuntimeRecipe {
    fn runtime_supported(&self) -> bool {
        self.runtime_tree_ready && self.missing.is_empty()
    }

    fn family_label(&self) -> &'static str {
        match self.family {
            RuntimeFamily::StableDiffusion => "Self-contained Diffusion GGUF",
            RuntimeFamily::StableDiffusion3 => "SD3 / SD3.5 GGUF",
            RuntimeFamily::Flux => "FLUX GGUF",
            RuntimeFamily::FluxKontext => "FLUX Kontext GGUF",
            RuntimeFamily::QwenImage => "Qwen Image GGUF",
            RuntimeFamily::QwenImageEdit => "Qwen Image Edit GGUF",
            RuntimeFamily::Wan => "Wan GGUF",
            RuntimeFamily::ZImage => "Z-Image GGUF",
            RuntimeFamily::OvisImage => "Ovis Image GGUF",
            RuntimeFamily::Anima => "Anima GGUF",
            RuntimeFamily::Ltx => "LTX GGUF",
            RuntimeFamily::StandaloneDiffusion => "Standalone Diffusion GGUF",
        }
    }

    fn generation_note_for(&self, kind: MediaKind) -> String {
        match (&self.family, kind) {
            (RuntimeFamily::StableDiffusion, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the self-contained diffusion path."
                    .to_string()
            }
            (RuntimeFamily::StableDiffusion, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the self-contained image path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::StableDiffusion, MediaKind::Video) => {
                "This self-contained diffusion path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::StableDiffusion, MediaKind::Audio) => {
                "stable-diffusion.cpp does not generate audio.".to_string()
            }
            (RuntimeFamily::StableDiffusion3, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the SD3 / SD3.5 path."
                    .to_string()
            }
            (RuntimeFamily::StableDiffusion3, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the SD3 image path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::StableDiffusion3, MediaKind::Video) => {
                "This SD3 / SD3.5 path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::StableDiffusion3, MediaKind::Audio) => {
                "stable-diffusion.cpp does not generate audio.".to_string()
            }
            (RuntimeFamily::Flux, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the FLUX path."
                    .to_string()
            }
            (RuntimeFamily::Flux, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the FLUX image path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::Flux, MediaKind::Video) => {
                "This FLUX path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::FluxKontext, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the FLUX Kontext edit path."
                    .to_string()
            }
            (RuntimeFamily::FluxKontext, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the FLUX Kontext edit path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::FluxKontext, MediaKind::Video) => {
                "This FLUX Kontext path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::QwenImage, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the Qwen Image path."
                    .to_string()
            }
            (RuntimeFamily::QwenImage, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the Qwen Image path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::QwenImage, MediaKind::Video) => {
                "This Qwen Image path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::QwenImage, MediaKind::Audio) => {
                "stable-diffusion.cpp does not generate audio.".to_string()
            }
            (RuntimeFamily::QwenImageEdit, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the Qwen Image Edit path."
                    .to_string()
            }
            (RuntimeFamily::QwenImageEdit, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the Qwen Image Edit path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::QwenImageEdit, MediaKind::Video) => {
                "This Qwen Image Edit path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::Wan, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the Wan image path.".to_string()
            }
            (RuntimeFamily::Wan, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the Wan video path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::Wan, MediaKind::Video) => {
                "Generated locally with stable-diffusion.cpp using the Wan video path and saved as an MJPG AVI video."
                    .to_string()
            }
            (RuntimeFamily::Wan, MediaKind::Audio) => {
                "stable-diffusion.cpp does not generate audio.".to_string()
            }
            (RuntimeFamily::ZImage, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the Z-Image path.".to_string()
            }
            (RuntimeFamily::ZImage, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the Z-Image image path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::ZImage, MediaKind::Video) => {
                "This Z-Image path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::OvisImage, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the Ovis Image path."
                    .to_string()
            }
            (RuntimeFamily::OvisImage, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the Ovis Image path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::OvisImage, MediaKind::Video) => {
                "This Ovis Image path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::Anima, MediaKind::Image) => {
                "Generated locally with stable-diffusion.cpp using the Anima path."
                    .to_string()
            }
            (RuntimeFamily::Anima, MediaKind::Gif) => {
                "Generated locally with stable-diffusion.cpp using the Anima path, then packed into a GIF clip."
                    .to_string()
            }
            (RuntimeFamily::Anima, MediaKind::Video) => {
                "This Anima path is wired for image and GIF output in Chatty-art. Pick Generate GIF for animated export."
                    .to_string()
            }
            (RuntimeFamily::Ltx, _) => {
                "stable-diffusion.cpp LTX support has not been wired into Chatty-art yet."
                    .to_string()
            }
            (RuntimeFamily::StandaloneDiffusion, _) => {
                "stable-diffusion.cpp detected a standalone diffusion GGUF, but Chatty-art has not wired that family yet."
                    .to_string()
            }
            (
                RuntimeFamily::Flux
                | RuntimeFamily::FluxKontext
                | RuntimeFamily::QwenImageEdit
                | RuntimeFamily::ZImage
                | RuntimeFamily::OvisImage
                | RuntimeFamily::Anima,
                MediaKind::Audio,
            ) => "stable-diffusion.cpp does not generate audio.".to_string(),
        }
    }
}

fn matches_qwen_image(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("qwen-image")
        || lower_name.contains("qwen_image")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "qwen_image")
                || summary.contains_tensor("transformer_blocks.0.attn.add_k_proj.weight")
                || summary.contains_tensor_fragment("transformer_blocks.0.img_mod.1.weight")
        })
}

fn matches_qwen_image_edit(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("qwen-image-edit")
        || lower_name.contains("qwen_image_edit")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "qwen_image_edit")
        })
}

fn matches_flux_kontext(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("kontext")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture.contains("kontext"))
        })
}

fn matches_flux(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    (lower_name.contains("flux")
        && !lower_name.contains("kontext")
        && !lower_name.contains("flux2"))
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "flux")
        })
}

fn matches_wan(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("wan")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "wan")
                || summary.contains_tensor("blocks.0.cross_attn.norm_k.weight")
                || summary.contains_tensor_fragment(
                    "model.diffusion_model.blocks.0.cross_attn.norm_k.weight",
                )
        })
}

fn matches_z_image(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("z-image")
        || lower_name.contains("z_image")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "z_image")
        })
}

fn matches_ovis_image(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("ovis-image")
        || lower_name.contains("ovis_image")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "ovis_image")
        })
}

fn matches_anima(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.starts_with("anima")
        || lower_name.contains("anima-")
        || lower_name.contains("anima_")
        || lower_name.contains("anima2")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "anima")
        })
}

fn matches_sd3_family(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("sd3.5")
        || lower_name.contains("sd3_5")
        || lower_name.contains("sd3-")
        || lower_name.starts_with("sd3")
        || lower_name.contains("stable-diffusion-3")
        || lower_name.contains("stable_diffusion_3")
        || gguf.is_some_and(|summary| {
            summary
                .architecture()
                .is_some_and(|architecture| architecture == "sd3")
                || summary.contains_tensor_fragment("text_encoders.clip_l.transformer.")
                || summary.contains_tensor_fragment("text_encoders.clip_g.transformer.")
                || summary.contains_tensor_fragment("model.diffusion_model.joint_blocks.")
        })
}

fn matches_self_contained_diffusion(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    if matches_sd3_family(lower_name, gguf) {
        return false;
    }

    if lower_name.contains("stable-diffusion")
        || lower_name.contains("sd-v1-5")
        || lower_name.contains("sd-v1_5")
        || lower_name.contains("sd15")
        || lower_name.contains("sdxl")
        || lower_name.contains("stablecascade")
    {
        return true;
    }

    let Some(gguf) = gguf else {
        return false;
    };

    let has_unet = gguf.contains_any_tensor_fragment(&[
        "model.diffusion_model.input_blocks.",
        "unet.down_blocks.",
        "model.diffusion_model.joint_blocks.",
        "model.diffusion_model.double_blocks.",
    ]);
    let has_text = gguf.contains_any_tensor_fragment(&[
        "cond_stage_model.transformer.text_model.embeddings.token_embedding.weight",
        "cond_stage_model.model.token_embedding.weight",
        "text_model.embeddings.token_embedding.weight",
        "te.text_model.embeddings.token_embedding.weight",
        "conditioner.embedders.0.model.token_embedding.weight",
        "conditioner.embedders.0.transformer.text_model.embeddings.token_embedding.weight",
        "text_encoders.clip_l.transformer.",
        "text_encoders.clip_g.transformer.",
        "text_encoders.t5xxl.transformer.",
        "text_encoders.llm.",
    ]);
    let has_vae = gguf.contains_any_tensor_fragment(&[
        "first_stage_model.decoder",
        "first_stage_model.encoder",
        "vae.decoder.",
        "vae.encoder.",
        "vae.mid.",
        "tae.decoder.",
    ]);
    has_unet && has_text && has_vae
}

fn sd3_has_checkpoint_layout(gguf: Option<&GgufSummary>) -> bool {
    gguf.is_some_and(|summary| {
        summary
            .architecture()
            .is_some_and(|architecture| architecture == "sd3")
            || summary.contains_tensor_fragment("model.diffusion_model.joint_blocks.")
            || summary.contains_tensor_fragment("model.diffusion_model.x_embedder.")
    })
}

fn sd3_has_internal_clip_l(gguf: Option<&GgufSummary>) -> bool {
    gguf.is_some_and(|summary| {
        summary.contains_tensor_fragment("text_encoders.clip_l.transformer.")
    })
}

fn sd3_has_internal_clip_g(gguf: Option<&GgufSummary>) -> bool {
    gguf.is_some_and(|summary| {
        summary.contains_tensor_fragment("text_encoders.clip_g.transformer.")
    })
}

fn sd3_has_internal_t5(gguf: Option<&GgufSummary>) -> bool {
    gguf.is_some_and(|summary| summary.contains_tensor_fragment("text_encoders.t5xxl.transformer."))
}

fn sd3_has_any_text_encoder(
    gguf: Option<&GgufSummary>,
    clip_l: Option<&Path>,
    clip_g: Option<&Path>,
    t5: Option<&Path>,
) -> bool {
    sd3_has_internal_clip_l(gguf)
        || sd3_has_internal_clip_g(gguf)
        || sd3_has_internal_t5(gguf)
        || clip_l.is_some()
        || clip_g.is_some()
        || t5.is_some()
}

fn sd3_recommended_components(
    gguf: Option<&GgufSummary>,
    clip_l: Option<&Path>,
    clip_g: Option<&Path>,
    t5: Option<&Path>,
) -> Vec<String> {
    let mut recommended = Vec::new();
    if !sd3_has_internal_clip_l(gguf) && clip_l.is_none() {
        recommended.push("clip_l.safetensors".to_string());
    }
    if !sd3_has_internal_clip_g(gguf) && clip_g.is_none() {
        recommended.push("clip_g.safetensors".to_string());
    }
    if !sd3_has_internal_t5(gguf) && t5.is_none() {
        recommended.push("t5xxl_fp16.safetensors".to_string());
    }
    recommended
}

fn looks_like_standalone_diffusion(gguf: Option<&GgufSummary>) -> bool {
    let Some(gguf) = gguf else {
        return false;
    };
    gguf.contains_any_tensor_fragment(&[
        "model.diffusion_model.input_blocks.",
        "unet.down_blocks.",
        "model.diffusion_model.joint_blocks.",
        "model.diffusion_model.double_blocks.",
        "model.diffusion_model.transformer_blocks.0.img_mod.1.weight",
        "model.diffusion_model.blocks.0.cross_attn.norm_k.weight",
        "model.diffusion_model.cap_embedder.0.weight",
        "llm_adapter.blocks.0.cross_attn.q_proj.weight",
    ])
}

fn is_unsupported_motion_diffusion(lower_name: &str, gguf: Option<&GgufSummary>) -> bool {
    lower_name.contains("animatediff")
        || lower_name.contains("animationdiffusion")
        || lower_name.contains("3danimation")
        || gguf.is_some_and(|summary| {
            summary.contains_any_tensor_fragment(&[
                "motion_encoder.",
                "motion_modules.",
                "temporal",
            ])
        })
}

fn is_unsupported_wan_variant(gguf: Option<&GgufSummary>) -> bool {
    gguf.is_some_and(|summary| {
        summary.contains_tensor_fragment("motion_encoder.")
            || summary.contains_tensor_fragment("face_adapter.")
            || summary.contains_tensor_fragment("pose_patch_embedding.")
    })
}

fn detect_wan_variant(lower_name: &str, gguf: Option<&GgufSummary>) -> WanVariant {
    if lower_name.contains("ti2v") {
        return WanVariant::Ti2v;
    }

    if lower_name.contains("vace")
        || lower_name.contains("control")
        || gguf.is_some_and(|summary| summary.contains_tensor_fragment("vace_blocks."))
    {
        return WanVariant::Vace;
    }

    if lower_name.contains("flf2v")
        || gguf.is_some_and(|summary| summary.contains_tensor_fragment("img_emb.emb_pos"))
    {
        return WanVariant::Flf2v;
    }

    if lower_name.contains("i2v")
        || lower_name.contains("fun")
        || gguf.is_some_and(|summary| summary.contains_tensor_fragment("img_emb"))
    {
        return WanVariant::I2v;
    }

    WanVariant::T2v
}

fn wan_variant_label(variant: WanVariant) -> &'static str {
    match variant {
        WanVariant::T2v => "Wan T2V",
        WanVariant::I2v => "Wan I2V",
        WanVariant::Flf2v => "Wan FLF2V",
        WanVariant::Ti2v => "Wan TI2V",
        WanVariant::Vace => "Wan VACE",
    }
}

fn wan_needs_clip_vision(variant: WanVariant) -> bool {
    matches!(variant, WanVariant::I2v | WanVariant::Flf2v)
}

fn wan_requires_image_reference(variant: WanVariant) -> bool {
    matches!(variant, WanVariant::I2v | WanVariant::Flf2v)
}

fn wan_supports_image_reference(variant: WanVariant) -> bool {
    matches!(
        variant,
        WanVariant::I2v | WanVariant::Flf2v | WanVariant::Ti2v | WanVariant::Vace
    )
}

fn wan_requires_end_image_reference(variant: WanVariant) -> bool {
    matches!(variant, WanVariant::Flf2v)
}

fn wan_supports_end_image_reference(variant: WanVariant) -> bool {
    matches!(variant, WanVariant::Flf2v)
}

fn wan_supports_video_reference(variant: WanVariant) -> bool {
    matches!(variant, WanVariant::Vace)
}

fn wan_vace_has_expected_layout(gguf: Option<&GgufSummary>) -> bool {
    gguf.is_some_and(|summary| {
        summary.contains_tensor_fragment("vace_patch_embedding")
            || summary.contains_tensor_fragment("model.diffusion_model.vace_patch_embedding")
    })
}

fn wan_needs_high_noise_pair(lower_name: &str, variant: WanVariant) -> bool {
    lower_name.contains("wan2.2") && !matches!(variant, WanVariant::Ti2v | WanVariant::Vace)
}

fn resolve_wan_model_paths(
    models_dir: &Path,
    model_path: &Path,
    lower_name: &str,
) -> (PathBuf, Option<PathBuf>, Option<String>) {
    let file_name = model_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let direct_pair = find_high_noise_pair(model_path);
    if file_name.contains("lownoise") || file_name.contains("low_noise") {
        return (model_path.to_path_buf(), direct_pair, None);
    }

    if file_name.contains("highnoise") || file_name.contains("high_noise") {
        if let Some(low_noise) = direct_pair {
            return (
                low_noise,
                Some(model_path.to_path_buf()),
                Some(
                    "Chatty-art used the matching low-noise Wan file as the primary diffusion model."
                        .to_string(),
                ),
            );
        }
    }

    if lower_name.contains("wan2.2")
        || lower_name.contains("animate")
        || lower_name.contains("fun")
        || lower_name.contains("control")
    {
        if let Some((low_noise, high_noise)) = find_any_wan_noise_pair(models_dir) {
            return (
                low_noise,
                Some(high_noise),
                Some(
                    "Chatty-art paired this Wan alias with the detected low-noise and high-noise companion files from models/."
                        .to_string(),
                ),
            );
        }
    }

    (model_path.to_path_buf(), None, None)
}

fn find_any_wan_noise_pair(models_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut low_noise = None;
    let mut high_noise = None;

    for entry in WalkDir::new(models_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.into_path();
        let lower = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if !lower.contains("wan2.2") {
            continue;
        }

        if lower.contains("lownoise") || lower.contains("low_noise") {
            low_noise = Some(path.clone());
        } else if lower.contains("highnoise") || lower.contains("high_noise") {
            high_noise = Some(path.clone());
        }
    }

    match (low_noise, high_noise) {
        (Some(low_noise), Some(high_noise)) => Some((low_noise, high_noise)),
        _ => None,
    }
}

fn recipe_note(
    diffuse_runtime_dir: &Path,
    family: &str,
    ready_note: &str,
    missing: &[String],
) -> String {
    if !diffuse_runtime_dir.join("CMakeLists.txt").exists() {
        return "Add the stable-diffusion.cpp source tree to diffuse_runtime/ to enable this realism backend.".to_string();
    }

    if !diffuse_runtime_dir
        .join("ggml")
        .join("CMakeLists.txt")
        .exists()
    {
        return "diffuse_runtime/ggml is missing. The stable-diffusion.cpp source tree needs the ggml submodule too.".to_string();
    }

    if missing.is_empty() {
        let mut note = format!(
            "{family} is ready for local stable-diffusion.cpp generation. Chatty-art will build sd-cli automatically the first time you use realism mode."
        );
        match detect_sdcpp_build_mode(diffuse_runtime_dir) {
            Some(SdcppBuildMode::Vulkan) => {
                note.push_str(
                    " The current stable-diffusion.cpp build has Vulkan acceleration enabled.",
                );
            }
            Some(SdcppBuildMode::CpuOnly) => {
                note.push_str(" The current stable-diffusion.cpp build is CPU-only, so large Wan video jobs can take a very long time.");
            }
            None if !has_vulkan_sdk() => {
                note.push_str(" No Vulkan SDK was detected, so the first stable-diffusion.cpp build will be CPU-only unless you install the Vulkan SDK first.");
            }
            None => {}
        }
        note
    } else {
        format!("{ready_note} Missing: {}.", missing.join(", "))
    }
}

fn find_first_file(models_dir: &Path, fragments: &[&str], extensions: &[&str]) -> Option<PathBuf> {
    WalkDir::new(models_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .find_map(|entry| {
            let path = entry.into_path();
            let lower = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let extension_ok = extensions.is_empty()
                || extensions.iter().any(|extension| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .map(|value| value.eq_ignore_ascii_case(extension))
                        .unwrap_or(false)
                });
            let fragment_ok =
                fragments.is_empty() || fragments.iter().any(|fragment| lower.contains(fragment));
            if extension_ok && fragment_ok {
                Some(path)
            } else {
                None
            }
        })
}

fn find_first_file_with_all_fragments(
    models_dir: &Path,
    fragments: &[&str],
    extensions: &[&str],
) -> Option<PathBuf> {
    WalkDir::new(models_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .find_map(|entry| {
            let path = entry.into_path();
            let lower = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let extension_ok = extensions.is_empty()
                || extensions.iter().any(|extension| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .map(|value| value.eq_ignore_ascii_case(extension))
                        .unwrap_or(false)
                });
            let fragment_ok = fragments.iter().all(|fragment| lower.contains(fragment));
            if extension_ok && fragment_ok {
                Some(path)
            } else {
                None
            }
        })
}

fn find_high_noise_pair(model_path: &Path) -> Option<PathBuf> {
    let file_name = model_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let parent = model_path.parent()?;
    let entries = std::fs::read_dir(parent).ok()?;

    if file_name.contains("highnoise") {
        return entries.flatten().find_map(|entry| {
            let path = entry.path();
            let lower = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if lower.contains("lownoise") {
                Some(path)
            } else {
                None
            }
        });
    }

    if file_name.contains("lownoise") {
        return entries.flatten().find_map(|entry| {
            let path = entry.path();
            let lower = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if lower.contains("highnoise") {
                Some(path)
            } else {
                None
            }
        });
    }

    None
}

fn missing_components(items: &[(&str, Option<&Path>)]) -> Vec<String> {
    items
        .iter()
        .filter_map(|(label, path)| {
            if path.is_none() {
                Some((*label).to_string())
            } else {
                None
            }
        })
        .collect()
}

fn encode_png_sequence_to_gif(frame_dir: &Path, output_path: &Path, fps: u32) -> Result<()> {
    let mut frames = std::fs::read_dir(frame_dir)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    frames.sort();

    if frames.is_empty() {
        bail!(
            "stable-diffusion.cpp did not write any PNG frames into {}.",
            frame_dir.display()
        );
    }

    let first = image::open(&frames[0])
        .with_context(|| format!("failed to open {}", frames[0].display()))?
        .into_rgba8();
    let width = first.width();
    let height = first.height();

    let file = File::create(output_path)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    let mut encoder = Encoder::new(file, width as u16, height as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for frame_path in frames {
        let frame = image::open(&frame_path)
            .with_context(|| format!("failed to open {}", frame_path.display()))?
            .into_rgba8();

        if frame.width() != width || frame.height() != height {
            bail!(
                "stable-diffusion.cpp wrote mixed frame sizes, which cannot be packed into one GIF."
            );
        }

        let mut pixels = frame.into_raw();
        let mut gif_frame = Frame::from_rgba_speed(width as u16, height as u16, &mut pixels, 10);
        gif_frame.delay = ((100.0 / fps.max(1) as f32).round() as u16).max(1);
        encoder.write_frame(&gif_frame)?;
    }

    Ok(())
}

fn summarize_output(label: &str, output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return format!("{label}: <empty>");
    }

    let lines = trimmed.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(24);
    format!("{label}:\n{}", lines[start..].join("\n"))
}

fn explain_sdcpp_failure(stdout: &str, stderr: &str) -> Option<&'static str> {
    let stdout = stdout.to_ascii_lowercase();
    let stderr = stderr.to_ascii_lowercase();
    let combined = format!("{stdout}\n{stderr}");

    if combined.contains("wan_vae_alloc compute buffer failed")
        || combined.contains("failed to decode latents")
        || combined.contains("vae alloc compute buffer failed")
    {
        return Some(
            "The denoising phase finished, but the VAE decode step ran out of memory while turning latents into final frames. Chatty-art can spill Wan VAE work to CPU and tile the decode path, but if this still happens try fewer frames or a smaller video resolution.",
        );
    }

    if combined.contains("requested buffer size exceeds device buffer size limit")
        || combined.contains("device memory allocation of size")
        || combined.contains("erroroutofmemory")
        || combined.contains("failed to allocate vulkan0 buffer")
        || combined.contains("alloc_tensor_range: failed to allocate")
        || combined.contains("t5 alloc runtime params backend buffer failed")
        || combined.contains("t5 offload params to runtime backend failed")
    {
        return Some(
            "The model loaded, but Vulkan ran out of VRAM or hit a GPU buffer-size limit while preparing runtime tensors. Chatty-art now keeps large text-encoder weights on CPU for Wan-style jobs, but if this still happens try a lighter model, fewer frames, or a smaller output size.",
        );
    }

    if combined.contains("get sd version from file failed") {
        return Some(
            "This GGUF does not look like a stable-diffusion.cpp-compatible drop-in SD checkpoint. SD3 / SD3.5 merge-style GGUFs often need a different conversion path or the documented SD3.5 safetensors + clip_l + clip_g + t5xxl setup instead.",
        );
    }

    if combined.contains("text_encoders.t5xxl.transformer.shared.weight has wrong shape")
        || combined.contains("load t5xxl from")
            && combined.contains("wan2.1")
            && combined.contains("wrong shape in model file")
    {
        return Some(
            "Chatty-art paired this Wan model with the wrong text encoder companion. Wan models usually need the UMT5 encoder file, not a generic FLUX or SD3 t5xxl weight. Keep `umt5-xxl-encoder` in models/ and Chatty-art will prefer it on the next run.",
        );
    }

    if combined.contains("vace_patch_embedding.weight") && combined.contains("not in model file") {
        return Some(
            "This Wan VACE GGUF was detected, but its conversion does not include the VACE patch-embedding tensors expected by the current stable-diffusion.cpp VACE runtime path. This is a model-conversion mismatch, not a prompt or VRAM problem.",
        );
    }

    if combined.contains("wrong shape in model file")
        || combined.contains("invalid number of dimensions")
        || combined.contains("unknown tensor")
        || combined.contains("load tensors from model loader failed")
    {
        return Some(
            "This model variant does not match the adapter Chatty-art used. The GGUF was detected, but its tensor layout is for a different family or conversion than this local runtime path supports.",
        );
    }

    None
}

fn native_relative_path(relative: &str) -> PathBuf {
    relative
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn has_vulkan_sdk() -> bool {
    std::env::var_os("VULKAN_SDK").is_some() || Path::new("C:\\VulkanSDK").exists()
}

fn detect_sdcpp_build_mode(diffuse_runtime_dir: &Path) -> Option<SdcppBuildMode> {
    let cache_path = diffuse_runtime_dir
        .join(BUILD_DIR_NAME)
        .join("CMakeCache.txt");
    let cache = std::fs::read_to_string(cache_path).ok()?;
    let vulkan_enabled = cache.lines().find_map(|line| {
        line.strip_prefix("SD_VULKAN:BOOL=")
            .or_else(|| line.strip_prefix("GGML_VULKAN:BOOL="))
            .map(|value| value.eq_ignore_ascii_case("ON"))
    })?;

    Some(if vulkan_enabled {
        SdcppBuildMode::Vulkan
    } else {
        SdcppBuildMode::CpuOnly
    })
}

fn runtime_tree_ready(diffuse_runtime_dir: &Path) -> bool {
    diffuse_runtime_dir.join("CMakeLists.txt").exists()
        && diffuse_runtime_dir
            .join("ggml")
            .join("CMakeLists.txt")
            .exists()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        RuntimeFamily, base_recipe, build_base_sdcpp_args, detect_runtime_recipe,
        detect_sdcpp_support, explain_sdcpp_failure, native_relative_path,
        normalize_sdcpp_video_output, primary_model_flag,
    };
    use crate::types::{
        GenerateRequest, GenerationSettings, GenerationStyle, MediaKind, PromptAssistMode,
        ReferenceIntent, ResolutionPreset, VideoResolutionPreset,
    };

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("chatty-art-{label}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn ready_diffuse_dir(label: &str) -> PathBuf {
        let diffuse_dir = temp_dir(label);
        fs::write(
            diffuse_dir.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.12)\n",
        )
        .unwrap();
        fs::create_dir_all(diffuse_dir.join("ggml")).unwrap();
        fs::write(
            diffuse_dir.join("ggml").join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.12)\n",
        )
        .unwrap();
        diffuse_dir
    }

    fn image_request(negative_prompt: Option<&str>) -> GenerateRequest {
        GenerateRequest {
            prompt: "a cat in a hat".to_string(),
            negative_prompt: negative_prompt.map(str::to_string),
            prompt_assist: PromptAssistMode::Off,
            model: "test.gguf".to_string(),
            kind: MediaKind::Image,
            style: GenerationStyle::Realism,
            settings: GenerationSettings {
                temperature: 0.8,
                steps: 24,
                cfg_scale: 6.5,
                resolution: ResolutionPreset::Square512,
                video_resolution: VideoResolutionPreset::Square256,
                video_duration_seconds: 2,
                video_fps: 8,
                low_vram_mode: false,
                seed: Some(7),
            },
            reference_asset: None,
            reference_intent: ReferenceIntent::Guide,
            end_reference_asset: None,
            control_reference_asset: None,
        }
    }

    fn tiny_diffusion_gguf() -> Vec<u8> {
        fn encode_string(value: &str) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
            bytes.extend_from_slice(value.as_bytes());
            bytes
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&3_u64.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());

        for tensor_name in [
            "model.diffusion_model.input_blocks.0.0.weight",
            "cond_stage_model.transformer.text_model.embeddings.token_embedding.weight",
            "first_stage_model.decoder.conv_in.weight",
        ] {
            bytes.extend_from_slice(&encode_string(tensor_name));
            bytes.extend_from_slice(&2_u32.to_le_bytes());
            bytes.extend_from_slice(&8_u64.to_le_bytes());
            bytes.extend_from_slice(&8_u64.to_le_bytes());
            bytes.extend_from_slice(&0_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }

        bytes
    }

    fn tiny_sd3_gguf(include_internal_encoders: bool) -> Vec<u8> {
        fn encode_string(value: &str) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
            bytes.extend_from_slice(value.as_bytes());
            bytes
        }

        let mut tensor_names = vec!["model.diffusion_model.joint_blocks.0.x_block.attn.qkv.weight"];
        if include_internal_encoders {
            tensor_names.extend([
                "text_encoders.clip_l.transformer.text_model.embeddings.token_embedding.weight",
                "text_encoders.clip_g.transformer.text_model.embeddings.token_embedding.weight",
                "text_encoders.t5xxl.transformer.shared.weight",
            ]);
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&(tensor_names.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());

        bytes.extend_from_slice(&encode_string("general.architecture"));
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(&encode_string("sd3"));

        for tensor_name in tensor_names {
            bytes.extend_from_slice(&encode_string(tensor_name));
            bytes.extend_from_slice(&2_u32.to_le_bytes());
            bytes.extend_from_slice(&8_u64.to_le_bytes());
            bytes.extend_from_slice(&8_u64.to_le_bytes());
            bytes.extend_from_slice(&0_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }

        bytes
    }

    #[test]
    fn qwen_requires_local_companion_files() {
        let diffuse_dir = ready_diffuse_dir("diffuse");

        let models_dir = temp_dir("models");
        fs::write(models_dir.join("qwen-image-Q4_K_M.gguf"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "qwen-image-Q4_K_M.gguf",
            "qwen-image-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(!support.runtime_supported);
        assert!(support.compatibility_note.contains("qwen_image_vae"));
    }

    #[test]
    fn qwen_becomes_ready_when_companions_exist() {
        let diffuse_dir = ready_diffuse_dir("diffuse-ready");

        let models_dir = temp_dir("models-ready");
        fs::write(models_dir.join("qwen-image-Q4_K_M.gguf"), b"").unwrap();
        fs::write(models_dir.join("qwen_image_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("Qwen2.5-VL-7B-Instruct-Q8_0.gguf"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "qwen-image-Q4_K_M.gguf",
            "qwen-image-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported);
        assert_eq!(
            native_relative_path("models/support/file.gguf"),
            PathBuf::from("models").join("support").join("file.gguf")
        );
    }

    #[test]
    fn stable_diffusion_self_contained_model_is_ready_without_companions() {
        let diffuse_dir = ready_diffuse_dir("diffuse-sd15");

        let models_dir = temp_dir("models-sd15");
        fs::write(
            models_dir.join("stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf"),
            b"",
        )
        .unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf",
            "stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported);
        assert_eq!(support.family, "Self-contained Diffusion GGUF");
        assert_eq!(
            support.supported_kinds,
            vec![MediaKind::Image, MediaKind::Gif]
        );
    }

    #[test]
    fn sd3_gguf_requires_a_text_encoder_when_not_self_contained() {
        let diffuse_dir = ready_diffuse_dir("diffuse-sd3");

        let models_dir = temp_dir("models-sd3");
        fs::write(
            models_dir.join("SD3.5-SLG-Weighted-Merge-Q8_0.gguf"),
            tiny_sd3_gguf(false),
        )
        .unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "SD3.5-SLG-Weighted-Merge-Q8_0.gguf",
            "SD3.5-SLG-Weighted-Merge-Q8_0.gguf",
        )
        .unwrap();
        assert!(!support.runtime_supported, "{support:?}");
        assert_eq!(support.family, "SD3 / SD3.5 GGUF");
        assert!(
            support
                .compatibility_note
                .contains("at least one SD3 text encoder"),
            "{support:?}"
        );
    }

    #[test]
    fn sd3_gguf_becomes_ready_with_companion_encoders() {
        let diffuse_dir = ready_diffuse_dir("diffuse-sd3-ready");

        let models_dir = temp_dir("models-sd3-ready");
        fs::write(
            models_dir.join("SD3.5-SLG-Weighted-Merge-Q8_0.gguf"),
            tiny_sd3_gguf(false),
        )
        .unwrap();
        fs::write(models_dir.join("clip_l.safetensors"), b"").unwrap();
        fs::write(models_dir.join("clip_g.safetensors"), b"").unwrap();
        fs::write(models_dir.join("t5xxl_fp16.safetensors"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "SD3.5-SLG-Weighted-Merge-Q8_0.gguf",
            "SD3.5-SLG-Weighted-Merge-Q8_0.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert_eq!(support.family, "SD3 / SD3.5 GGUF");
        assert_eq!(
            support.supported_kinds,
            vec![MediaKind::Image, MediaKind::Gif]
        );
    }

    #[test]
    fn self_contained_sd3_gguf_is_routed_to_sd3_family() {
        let diffuse_dir = ready_diffuse_dir("diffuse-sd3-self-contained");

        let models_dir = temp_dir("models-sd3-self-contained");
        fs::write(
            models_dir.join("sd3-medium-all-in-one.gguf"),
            tiny_sd3_gguf(true),
        )
        .unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "sd3-medium-all-in-one.gguf",
            "sd3-medium-all-in-one.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert_eq!(support.family, "SD3 / SD3.5 GGUF");
    }

    #[test]
    fn stable_diffusion_uses_full_model_flag() {
        assert_eq!(primary_model_flag(&RuntimeFamily::StableDiffusion), "-m");
        assert_eq!(primary_model_flag(&RuntimeFamily::StableDiffusion3), "-m");
        assert_eq!(
            primary_model_flag(&RuntimeFamily::QwenImage),
            "--diffusion-model"
        );
        assert_eq!(
            primary_model_flag(&RuntimeFamily::FluxKontext),
            "--diffusion-model"
        );
        assert_eq!(primary_model_flag(&RuntimeFamily::Wan), "--diffusion-model");
        assert_eq!(
            primary_model_flag(&RuntimeFamily::StandaloneDiffusion),
            "--diffusion-model"
        );
    }

    #[test]
    fn realism_args_forward_negative_prompt() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::StableDiffusion,
            Path::new("models/sd15.gguf"),
        );
        recipe.runtime_tree_ready = true;
        let request = image_request(Some("blurry, low quality"));
        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-n", "blurry, low quality"])
        );
    }

    #[test]
    fn realism_args_fall_back_to_default_negative_prompt() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::StableDiffusion,
            Path::new("models/sd15.gguf"),
        );
        recipe.runtime_tree_ready = true;
        let request = image_request(None);
        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-n", "low quality, blurry, distorted"])
        );
    }

    #[test]
    fn wan_args_keep_text_encoder_on_cpu() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::Wan,
            Path::new("models/wan.gguf"),
        );
        recipe.runtime_tree_ready = true;
        let request = image_request(Some("blurry"));
        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();
        assert!(args.iter().any(|value| value == "--clip-on-cpu"));
    }

    #[test]
    fn wan_args_use_requested_video_resolution_duration_and_fps() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::Wan,
            Path::new("models/wan.gguf"),
        );
        recipe.runtime_tree_ready = true;
        let mut request = image_request(Some("blurry"));
        request.kind = MediaKind::Gif;
        request.settings.video_resolution = VideoResolutionPreset::Square768;
        request.settings.video_duration_seconds = 5;
        request.settings.video_fps = 16;

        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();

        assert!(
            args.windows(2).any(|pair| pair == ["-W", "768"]),
            "{args:?}"
        );
        assert!(
            args.windows(2).any(|pair| pair == ["-H", "768"]),
            "{args:?}"
        );
        assert!(
            args.windows(2).any(|pair| pair == ["--video-frames", "80"]),
            "{args:?}"
        );
        assert!(
            args.windows(2).any(|pair| pair == ["--fps", "16"]),
            "{args:?}"
        );
    }

    #[test]
    fn wan_video_args_enable_low_vram_vae_protection() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::Wan,
            Path::new("models/wan.gguf"),
        );
        recipe.runtime_tree_ready = true;
        recipe.vae_path = Some(PathBuf::from("models/wan_2.1_vae.safetensors"));
        let mut request = image_request(Some("blurry"));
        request.kind = MediaKind::Video;
        request.settings.video_resolution = VideoResolutionPreset::Square512;
        request.settings.video_duration_seconds = 2;
        request.settings.video_fps = 8;

        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();

        assert!(args.iter().any(|value| value == "--vae-on-cpu"), "{args:?}");
        assert!(args.iter().any(|value| value == "--vae-tiling"), "{args:?}");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--vae-relative-tile-size", "2x2"]),
            "{args:?}"
        );
    }

    #[test]
    fn high_res_image_args_enable_tiling_and_cpu_offload() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::StableDiffusion,
            Path::new("models/sd15.gguf"),
        );
        recipe.runtime_tree_ready = true;
        recipe.vae_path = Some(PathBuf::from("models/vae.safetensors"));
        let mut request = image_request(Some("blurry"));
        request.settings.resolution = ResolutionPreset::Landscape1024;

        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();

        assert!(
            args.iter().any(|value| value == "--offload-to-cpu"),
            "{args:?}"
        );
        assert!(args.iter().any(|value| value == "--vae-tiling"), "{args:?}");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--vae-relative-tile-size", "3x3"]),
            "{args:?}"
        );
    }

    #[test]
    fn low_vram_mode_enables_safer_image_decode_even_at_512() {
        let mut recipe = base_recipe(
            Path::new("."),
            RuntimeFamily::StableDiffusion,
            Path::new("models/sd15.gguf"),
        );
        recipe.runtime_tree_ready = true;
        recipe.vae_path = Some(PathBuf::from("models/vae.safetensors"));
        let mut request = image_request(Some("blurry"));
        request.settings.low_vram_mode = true;

        let args = build_base_sdcpp_args(&recipe, &request, 7).unwrap();

        assert!(
            args.iter().any(|value| value == "--offload-to-cpu"),
            "{args:?}"
        );
        assert!(args.iter().any(|value| value == "--vae-tiling"), "{args:?}");
        assert!(args.iter().any(|value| value == "--vae-on-cpu"), "{args:?}");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--vae-relative-tile-size", "3x3"]),
            "{args:?}"
        );
    }

    #[test]
    fn video_output_normalizer_accepts_runtime_appended_avi_extension() {
        let dir = temp_dir("video-output-normalizer");
        let expected = dir.join("result.avi");
        let actual = PathBuf::from(format!("{}.avi", expected.display()));
        fs::write(&actual, b"avi").unwrap();

        normalize_sdcpp_video_output(&expected).unwrap();

        assert!(expected.exists(), "expected {}", expected.display());
        assert!(!actual.exists(), "unexpected {}", actual.display());
    }

    #[test]
    fn vulkan_oom_failure_is_reported_before_generic_tensor_layout_warning() {
        let friendly = explain_sdcpp_failure(
            "tensor 'patch_embedding.weight' has invalid number of dimensions: 5 > 4",
            "Requested buffer size exceeds device buffer size limit: ErrorOutOfDeviceMemory\n\
             t5 alloc runtime params backend buffer failed",
        );
        let message = friendly.expect("expected a friendly error message");
        assert!(message.contains("Vulkan ran out of VRAM"), "{message}");
    }

    #[test]
    fn self_contained_diffusion_is_detected_from_tensor_names() {
        let diffuse_dir = ready_diffuse_dir("diffuse-sig");

        let models_dir = temp_dir("models-sig");
        fs::write(models_dir.join("mystery-merge.gguf"), tiny_diffusion_gguf()).unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "mystery-merge.gguf",
            "mystery-merge.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported);
        assert_eq!(support.family, "Self-contained Diffusion GGUF");
        assert_eq!(
            support.supported_kinds,
            vec![MediaKind::Image, MediaKind::Gif]
        );
    }

    #[test]
    fn wan_supports_image_and_video() {
        let diffuse_dir = ready_diffuse_dir("diffuse-wan");

        let models_dir = temp_dir("models-wan");
        fs::write(models_dir.join("wan2.1-t2v-14b-Q8_0.gguf"), b"").unwrap();
        fs::write(models_dir.join("wan_2.1_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("umt5-xxl-encoder-Q8_0.gguf"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "wan2.1-t2v-14b-Q8_0.gguf",
            "wan2.1-t2v-14b-Q8_0.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert_eq!(
            support.supported_kinds,
            vec![MediaKind::Image, MediaKind::Gif, MediaKind::Video]
        );
    }

    #[test]
    fn wan_prefers_umt5_over_generic_t5xxl_when_both_exist() {
        let diffuse_dir = ready_diffuse_dir("diffuse-wan-t5-priority");

        let models_dir = temp_dir("models-wan-t5-priority");
        fs::write(models_dir.join("wan2.1-t2v-14b-q8_0.gguf"), b"").unwrap();
        fs::write(models_dir.join("wan_2.1_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("umt5-xxl-encoder-Q4_K_M.gguf"), b"").unwrap();
        fs::write(models_dir.join("t5xxl_fp16.safetensors"), b"").unwrap();

        let recipe = detect_runtime_recipe(
            &diffuse_dir,
            &models_dir,
            &models_dir.join("wan2.1-t2v-14b-q8_0.gguf"),
            "wan2.1-t2v-14b-q8_0.gguf",
        )
        .unwrap();

        let chosen = recipe
            .t5_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        assert!(chosen.contains("umt5"), "{chosen}");
    }

    fn tiny_wan_vace_gguf(has_vace_patch_embedding: bool) -> Vec<u8> {
        fn encode_string(value: &str) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
            bytes.extend_from_slice(value.as_bytes());
            bytes
        }

        let mut tensor_names = vec!["vace_blocks.0.attn.qkv.weight"];
        if has_vace_patch_embedding {
            tensor_names.push("model.diffusion_model.vace_patch_embedding.weight");
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&(tensor_names.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());

        bytes.extend_from_slice(&encode_string("general.architecture"));
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(&encode_string("wan"));

        for tensor_name in tensor_names {
            bytes.extend_from_slice(&encode_string(tensor_name));
            bytes.extend_from_slice(&2_u32.to_le_bytes());
            bytes.extend_from_slice(&8_u64.to_le_bytes());
            bytes.extend_from_slice(&8_u64.to_le_bytes());
            bytes.extend_from_slice(&0_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }

        bytes
    }

    #[test]
    fn wan_vace_without_patch_embedding_is_marked_not_ready() {
        let diffuse_dir = ready_diffuse_dir("diffuse-wan-vace-layout");
        let models_dir = temp_dir("models-wan-vace-layout");
        fs::write(
            models_dir.join("wan2.1-vace-1.3b-q8_0.gguf"),
            tiny_wan_vace_gguf(false),
        )
        .unwrap();
        fs::write(models_dir.join("wan_2.1_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("umt5-xxl-encoder-Q8_0.gguf"), b"").unwrap();

        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "wan2.1-vace-1.3b-q8_0.gguf",
            "wan2.1-vace-1.3b-q8_0.gguf",
        )
        .unwrap();

        assert!(!support.runtime_supported, "{support:?}");
        assert!(
            support.compatibility_note.contains("VACE")
                && support.compatibility_note.contains("patch-embedding"),
            "{support:?}"
        );
    }

    #[test]
    fn wan_flf2v_requires_start_and_end_images() {
        let diffuse_dir = ready_diffuse_dir("diffuse-wan-flf2v");

        let models_dir = temp_dir("models-wan-flf2v");
        fs::write(models_dir.join("wan2.1-flf2v-14b-720p-Q8_0.gguf"), b"").unwrap();
        fs::write(models_dir.join("wan_2.1_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("umt5-xxl-encoder-Q8_0.gguf"), b"").unwrap();
        fs::write(models_dir.join("clip_vision_h.safetensors"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "wan2.1-flf2v-14b-720p-Q8_0.gguf",
            "wan2.1-flf2v-14b-720p-Q8_0.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert!(support.requires_reference, "{support:?}");
        assert!(support.requires_end_image_reference, "{support:?}");
        assert!(support.supports_end_image_reference, "{support:?}");
    }

    #[test]
    fn wan_vace_supports_control_video_guidance() {
        let diffuse_dir = ready_diffuse_dir("diffuse-wan-vace");

        let models_dir = temp_dir("models-wan-vace");
        fs::write(
            models_dir.join("Wan2.1_14B_VACE-Q8_0.gguf"),
            tiny_wan_vace_gguf(true),
        )
        .unwrap();
        fs::write(models_dir.join("wan_2.1_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("umt5-xxl-encoder-Q8_0.gguf"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "Wan2.1_14B_VACE-Q8_0.gguf",
            "Wan2.1_14B_VACE-Q8_0.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert!(support.supports_image_reference, "{support:?}");
        assert!(support.supports_video_reference, "{support:?}");
        assert!(!support.requires_reference, "{support:?}");
    }

    #[test]
    fn wan_animate_alias_is_routed_to_wan_recipe_and_reports_missing_parts() {
        let diffuse_dir = ready_diffuse_dir("diffuse-wan-animate");

        let models_dir = temp_dir("models-wan-animate");
        fs::write(models_dir.join("Wan2.2-Animate-14B-Q4_K_M.gguf"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "Wan2.2-Animate-14B-Q4_K_M.gguf",
            "Wan2.2-Animate-14B-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(!support.runtime_supported, "{support:?}");
        assert_eq!(support.family, "Wan GGUF");
        assert!(
            support.compatibility_note.contains("Wan T2V")
                || support.compatibility_note.contains("Wan I2V")
                || support.compatibility_note.contains("Wan FLF2V")
                || support.compatibility_note.contains("Wan TI2V")
                || support.compatibility_note.contains("Wan VACE"),
            "{support:?}"
        );
        assert!(
            support.compatibility_note.contains("Missing:")
                || support.compatibility_note.contains("enable it"),
            "{support:?}"
        );
    }

    #[test]
    fn motion_diffusion_checkpoint_is_not_treated_as_standard_self_contained_sd() {
        let diffuse_dir = ready_diffuse_dir("diffuse-motion");

        let models_dir = temp_dir("models-motion");
        fs::write(models_dir.join("3dAnimationDiffusion_v10-f16.gguf"), b"").unwrap();
        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "3dAnimationDiffusion_v10-f16.gguf",
            "3dAnimationDiffusion_v10-f16.gguf",
        )
        .unwrap();
        assert!(!support.runtime_supported);
        assert!(support.compatibility_note.contains("animation"));
    }

    #[test]
    fn flux_becomes_ready_with_companion_weights() {
        let diffuse_dir = ready_diffuse_dir("diffuse-flux");
        let models_dir = temp_dir("models-flux");
        fs::write(models_dir.join("flux1-dev-Q4_K_M.gguf"), b"").unwrap();
        fs::write(models_dir.join("ae.sft"), b"").unwrap();
        fs::write(models_dir.join("clip_l.safetensors"), b"").unwrap();
        fs::write(models_dir.join("t5xxl_fp16.safetensors"), b"").unwrap();

        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "flux1-dev-Q4_K_M.gguf",
            "flux1-dev-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert_eq!(support.family, "FLUX GGUF");
    }

    #[test]
    fn kontext_requires_reference_and_companion_weights() {
        let diffuse_dir = ready_diffuse_dir("diffuse-kontext");
        let models_dir = temp_dir("models-kontext");
        fs::write(models_dir.join("flux1-kontext-dev-Q4_K_M.gguf"), b"").unwrap();
        fs::write(models_dir.join("ae.sft"), b"").unwrap();
        fs::write(models_dir.join("clip_l.safetensors"), b"").unwrap();
        fs::write(models_dir.join("t5xxl_fp16.safetensors"), b"").unwrap();

        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "flux1-kontext-dev-Q4_K_M.gguf",
            "flux1-kontext-dev-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert!(support.requires_reference);
        assert!(support.supports_image_reference);
    }

    #[test]
    fn qwen_image_edit_2509_requires_mmproj() {
        let diffuse_dir = ready_diffuse_dir("diffuse-qwen-edit");
        let models_dir = temp_dir("models-qwen-edit");
        fs::write(models_dir.join("Qwen-Image-Edit-2509-Q4_K_M.gguf"), b"").unwrap();
        fs::write(models_dir.join("qwen_image_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("Qwen2.5-VL-7B-Instruct-Q8_0.gguf"), b"").unwrap();

        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "Qwen-Image-Edit-2509-Q4_K_M.gguf",
            "Qwen-Image-Edit-2509-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(!support.runtime_supported, "{support:?}");
        assert!(support.compatibility_note.contains("mmproj"), "{support:?}");
    }

    #[test]
    fn qwen_image_edit_2511_is_ready_without_mmproj() {
        let diffuse_dir = ready_diffuse_dir("diffuse-qwen-edit-2511");
        let models_dir = temp_dir("models-qwen-edit-2511");
        fs::write(models_dir.join("qwen-image-edit-2511-Q4_K_M.gguf"), b"").unwrap();
        fs::write(models_dir.join("qwen_image_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("Qwen2.5-VL-7B-Instruct-Q8_0.gguf"), b"").unwrap();

        let support = detect_sdcpp_support(
            &diffuse_dir,
            &models_dir,
            "qwen-image-edit-2511-Q4_K_M.gguf",
            "qwen-image-edit-2511-Q4_K_M.gguf",
        )
        .unwrap();
        assert!(support.runtime_supported, "{support:?}");
        assert!(support.requires_reference);
    }
}
