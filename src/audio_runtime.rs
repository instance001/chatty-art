use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};

use anyhow::{Context, Result, bail};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    runtime::derive_spoken_text_heuristic,
    types::{AudioPromptSegment, GenerateRequest, InputAsset, MediaKind, ModelInfo},
};

#[derive(Debug, Clone)]
pub struct AudioRuntimeSupport {
    pub family: String,
    pub runtime_supported: bool,
    pub compatibility_note: String,
    pub supported_kinds: Vec<MediaKind>,
    pub supports_audio_reference: bool,
    pub supports_voice_output: bool,
}

#[derive(Debug, Clone)]
pub struct AudioGenerationResult {
    pub mime: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RuntimeProbe {
    ready: bool,
    supports_audio_reference: bool,
    note: String,
}

#[derive(Debug, Clone, Deserialize)]
struct StableAudioInterpreterProbe {
    ready: bool,
    gpu_available: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OuteTtsInterpreterProbe {
    ready: bool,
    supports_audio_reference: bool,
}

#[derive(Debug, Clone)]
struct RenderedAudioSegment {
    path: PathBuf,
    same_time_as_previous: bool,
}

#[derive(Debug, Clone)]
struct DecodedStereoWav {
    sample_rate: u32,
    samples: Vec<f32>,
}

const OUTETTS_VOICE_ARCHETYPES: &[&str] = &[
    "warm and steady adult delivery",
    "clear bright conversational delivery",
    "measured low-register narrator delivery",
    "soft intimate reflective delivery",
    "crisp energetic confident delivery",
    "calm slightly husky grounded delivery",
    "light playful curious delivery",
    "precise clipped synthetic delivery",
];

const STABLE_AUDIO_LAYER_ARCHETYPES: &[&str] = &[
    "keep the layer focused and dry in the foreground",
    "keep the layer wide and atmospheric in the background",
    "keep the layer crisp with punchy transient detail",
    "keep the layer smooth, airy, and textural",
    "keep the layer weighty and low-end driven",
    "keep the layer bright, glittering, and spacious",
    "keep the layer mechanical, tight, and rhythmic",
    "keep the layer organic, diffuse, and naturalistic",
];

pub fn detect_audio_runtime_support(
    file_name: &str,
    audio_runtime_dir: &Path,
) -> Option<AudioRuntimeSupport> {
    let lower = file_name.to_ascii_lowercase();

    if is_outetts_family(&lower) {
        let probe = probe_outetts_runtime(audio_runtime_dir);
        return Some(AudioRuntimeSupport {
            family: "OuteTTS".to_string(),
            runtime_supported: probe.ready,
            compatibility_note: probe.note,
            supported_kinds: vec![MediaKind::Audio],
            supports_audio_reference: probe.supports_audio_reference,
            supports_voice_output: true,
        });
    }

    if is_qwen3_tts_family(&lower) {
        return Some(AudioRuntimeSupport {
            family: "Qwen3-TTS".to_string(),
            runtime_supported: false,
            compatibility_note: "Detected as a Qwen3-TTS speech model. Qwen3-TTS uses its own local runtime path rather than Chatty-art's current backends, so this family is detected but not wired yet.".to_string(),
            supported_kinds: vec![MediaKind::Audio],
            supports_audio_reference: false,
            supports_voice_output: true,
        });
    }

    if is_kokoro_family(&lower) {
        return Some(AudioRuntimeSupport {
            family: "Kokoro".to_string(),
            runtime_supported: false,
            compatibility_note: "Detected as a Kokoro speech model. Chatty-art has not wired a dedicated Kokoro runtime yet, so this family is detected but not ready.".to_string(),
            supported_kinds: vec![MediaKind::Audio],
            supports_audio_reference: false,
            supports_voice_output: true,
        });
    }

    None
}

pub fn detect_audio_runtime_package_support(
    model_name: &str,
    model_path: &Path,
    audio_runtime_dir: &Path,
) -> Option<AudioRuntimeSupport> {
    if is_stable_audio_package(model_name, model_path) {
        let probe = probe_stable_audio_runtime(audio_runtime_dir, model_path);
        return Some(AudioRuntimeSupport {
            family: "Stable Audio Open".to_string(),
            runtime_supported: probe.ready,
            compatibility_note: probe.note,
            supported_kinds: vec![MediaKind::Audio],
            supports_audio_reference: false,
            supports_voice_output: false,
        });
    }

    None
}

pub async fn generate_with_audio_runtime(
    audio_runtime_dir: &Path,
    models_dir: &Path,
    input_dir: &Path,
    model: &ModelInfo,
    request: &GenerateRequest,
    reference_asset: Option<&InputAsset>,
    used_seed: u32,
    output_path: &Path,
) -> Result<AudioGenerationResult> {
    let family = model.family.to_ascii_lowercase();
    if family.contains("outetts") {
        return generate_with_outetts(
            audio_runtime_dir,
            models_dir,
            input_dir,
            model,
            request,
            reference_asset,
            used_seed,
            output_path,
        )
        .await;
    }
    if family.contains("stable audio") {
        return generate_with_stable_audio(
            audio_runtime_dir,
            models_dir,
            model,
            request,
            reference_asset,
            used_seed,
            output_path,
        )
        .await;
    }

    bail!(
        "Realism audio is not wired yet for '{}'. {}",
        model.name,
        model.compatibility_note
    )
}

fn is_outetts_family(name: &str) -> bool {
    name.contains("outetts")
        || name.contains("oute-tts")
        || name.contains("llama-outetts")
        || name.contains("llama_oute")
}

fn is_qwen3_tts_family(name: &str) -> bool {
    name.contains("qwen3-tts") || (name.contains("qwen") && name.contains("tts"))
}

fn is_kokoro_family(name: &str) -> bool {
    name.contains("kokoro")
}

fn is_stable_audio_package(model_name: &str, model_path: &Path) -> bool {
    if !model_path.is_dir() {
        return false;
    }

    let lower = model_name.to_ascii_lowercase();
    if !lower.contains("stable-audio-open") && !lower.contains("stable audio open") {
        return false;
    }

    let required = [
        "model_index.json",
        "model_config.json",
        "model.safetensors",
        "projection_model",
        "scheduler",
        "text_encoder",
        "tokenizer",
        "transformer",
        "vae",
    ];

    if required.iter().any(|name| !model_path.join(name).exists()) {
        return false;
    }

    match std::fs::read_to_string(model_path.join("model_index.json")) {
        Ok(contents) => contents
            .to_ascii_lowercase()
            .contains("\"stableaudiopipeline\""),
        Err(_) => false,
    }
}

fn probe_outetts_runtime(audio_runtime_dir: &Path) -> RuntimeProbe {
    let cache = outetts_probe_cache();
    let cache_key = audio_runtime_dir.to_string_lossy().to_string();
    if let Ok(guard) = cache.lock() {
        if let Some(probe) = guard.get(&cache_key) {
            return probe.clone();
        }
    }

    let source_dir = outetts_source_dir(audio_runtime_dir);
    let runner_path = outetts_runner_path(audio_runtime_dir);

    if !source_dir.exists() {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as an OuteTTS speech GGUF, but Chatty-art could not find the local OuteTTS source tree at audio_runtime/outetts/OuteTTS-main.".to_string(),
        };
    }

    if !runner_path.exists() {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as an OuteTTS speech GGUF, but Chatty-art's local OuteTTS runner script is missing from audio_runtime/.".to_string(),
        };
    }

    let Some(interpreter) = outetts_python_interpreter(audio_runtime_dir) else {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as an OuteTTS speech GGUF, but Chatty-art could not find a usable Python runtime for the local OuteTTS source tree. Install the speech dependencies into a Python interpreter and set CHATTY_ART_OUTETTS_PYTHON if you want to force a specific one.".to_string(),
        };
    };

    let probe_script = r#"
import importlib
import json
import sys
from pathlib import Path

source_dir = Path(sys.argv[1])
sys.path.insert(0, str(source_dir))

try:
    import torch.distributed as dist
    if not hasattr(dist, "ReduceOp"):
        class _ReduceOp:
            AVG = "avg"
            SUM = "sum"
            MIN = "min"
            MAX = "max"
            PRODUCT = "product"
        dist.ReduceOp = _ReduceOp
except Exception:
    pass

required = [
    "loguru",
    "polars",
    "ftfy",
    "transformers",
    "llama_cpp",
    "huggingface_hub",
    "soundfile",
    "pyloudnorm",
    "MeCab",
    "uroman",
]
reference_optional = [
    "whisper",
]

missing = []
for module_name in required:
    try:
        importlib.import_module(module_name)
    except Exception:
        missing.append(module_name)

reference_missing = []
for module_name in reference_optional:
    try:
        importlib.import_module(module_name)
    except Exception:
        reference_missing.append(module_name)

try:
    import outetts  # noqa: F401
except Exception as exc:
    print(json.dumps({
        "ready": False,
        "supports_audio_reference": False,
        "note": f"Detected as an OuteTTS speech GGUF, but the local OuteTTS runtime could not import from audio_runtime/outetts/OuteTTS-main: {exc}",
    }))
    raise SystemExit(0)

if missing:
    pretty = ", ".join(sorted(missing))
    print(json.dumps({
        "ready": False,
        "supports_audio_reference": False,
        "note": "Detected as an OuteTTS speech GGUF. Install the missing Python packages for the local OuteTTS runtime: " + pretty,
    }))
    raise SystemExit(0)

if reference_missing:
    pretty = ", ".join(sorted(reference_missing))
    print(json.dumps({
        "ready": True,
        "supports_audio_reference": False,
        "note": "Ready to run with the local OuteTTS runtime. Chatty-art can synthesize speech now, but audio-reference voice cloning still needs: " + pretty + ". First run may download tokenizer files from Hugging Face.",
    }))
    raise SystemExit(0)

print(json.dumps({
    "ready": True,
    "supports_audio_reference": True,
    "note": "Ready to run with the local OuteTTS runtime. Chatty-art can synthesize speech, and audio-reference voice cloning is available. First run may download tokenizer files from Hugging Face.",
}))
"#;

    let probe = match run_python_sync_with_interpreter(
        &interpreter,
        [
            "-c".to_string(),
            probe_script.to_string(),
            source_dir.to_string_lossy().to_string(),
        ],
    ) {
        Ok(output) => parse_runtime_probe(&output).unwrap_or(RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as an OuteTTS speech GGUF, but Chatty-art could not parse the local OuteTTS runtime probe result.".to_string(),
        }),
        Err(error) => RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: format!(
                "Detected as an OuteTTS speech GGUF, but Chatty-art could not start the local Python runtime probe: {error}"
            ),
        },
    };

    if let Ok(mut guard) = cache.lock() {
        guard.insert(cache_key, probe.clone());
    }

    probe
}

fn probe_stable_audio_runtime(audio_runtime_dir: &Path, package_dir: &Path) -> RuntimeProbe {
    let cache = stable_audio_probe_cache();
    let cache_key = format!(
        "{}|{}",
        audio_runtime_dir.to_string_lossy(),
        package_dir.to_string_lossy()
    );
    if let Ok(guard) = cache.lock() {
        if let Some(probe) = guard.get(&cache_key)
            && probe.ready
        {
            return probe.clone();
        }
    }

    let source_dir = stable_audio_source_dir(audio_runtime_dir);
    let runner_path = stable_audio_runner_path(audio_runtime_dir);

    if !source_dir.exists() {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as a Stable Audio Open package, but Chatty-art could not find the local Stable Audio Tools source tree at audio_runtime/stable_audio_tools/stable-audio-tools-main.".to_string(),
        };
    }

    if !runner_path.exists() {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as a Stable Audio Open package, but Chatty-art's local Stable Audio runner script is missing from audio_runtime/.".to_string(),
        };
    }

    if !is_stable_audio_package(
        package_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("stable-audio-open"),
        package_dir,
    ) {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as a Stable Audio package, but the local folder is missing one or more required files like model.safetensors, model_config.json, model_index.json, or the projection/text_encoder/tokenizer/transformer/vae folders.".to_string(),
        };
    }

    let Some(interpreter) = stable_audio_python_interpreter(audio_runtime_dir) else {
        return RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as a Stable Audio Open package. Chatty-art could not find a usable Python runtime for Stable Audio. Create a dedicated audio environment or point CHATTY_ART_STABLE_AUDIO_PYTHON at a Python that can import the local Stable Audio source tree and its inference dependencies.".to_string(),
        };
    };

    let probe_script = r#"
import importlib
import json
import sys
from pathlib import Path

source_dir = Path(sys.argv[1])
package_dir = Path(sys.argv[2])
sys.path.insert(0, str(source_dir))

required = [
    "torch",
    "torchaudio",
    "einops",
    "soundfile",
    "safetensors",
    "transformers",
    "k_diffusion",
]
missing = []
for module_name in required:
    try:
        importlib.import_module(module_name)
    except Exception:
        missing.append(module_name)

if missing:
    pretty = ", ".join(sorted(missing))
    print(json.dumps({
        "ready": False,
        "supports_audio_reference": False,
        "note": "Stable Audio Open detected. The dedicated audio runtime is missing Python packages: " + pretty + ". Install them into the Stable Audio environment."
    }))
    raise SystemExit(0)

try:
    import stable_audio_tools  # noqa: F401
    from stable_audio_tools.inference.generation import generate_diffusion_cond  # noqa: F401
    from stable_audio_tools.models.factory import create_model_from_config  # noqa: F401
except Exception as exc:
    print(json.dumps({
        "ready": False,
        "supports_audio_reference": False,
        "note": "Stable Audio Open detected, but Chatty-art could not import the local Stable Audio inference runtime from audio_runtime/stable_audio_tools/stable-audio-tools-main: " + str(exc)
    }))
    raise SystemExit(0)

if not (package_dir / "model.safetensors").exists():
    print(json.dumps({
        "ready": False,
        "supports_audio_reference": False,
        "note": "Stable Audio Open detected, but model.safetensors is missing from the package folder."
    }))
    raise SystemExit(0)

print(json.dumps({
    "ready": True,
    "supports_audio_reference": False,
    "note": "Ready to run with the local Stable Audio runtime. Best suited to sound effects, ambience, and texture-driven audio. First run may still warm up slowly."
}))
"#;

    let probe = match run_python_sync_with_interpreter(
        &interpreter,
        [
            "-c".to_string(),
            probe_script.to_string(),
            source_dir.to_string_lossy().to_string(),
            package_dir.to_string_lossy().to_string(),
        ],
    ) {
        Ok(output) => parse_runtime_probe(&output).unwrap_or(RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: "Detected as a Stable Audio Open package, but Chatty-art could not parse the Stable Audio runtime probe result.".to_string(),
        }),
        Err(error) => RuntimeProbe {
            ready: false,
            supports_audio_reference: false,
            note: format!(
                "Detected as a Stable Audio Open package, but Chatty-art could not start the dedicated Stable Audio Python runtime: {error}"
            ),
        },
    };

    if let Ok(mut guard) = cache.lock() {
        if probe.ready {
            guard.insert(cache_key, probe.clone());
        } else {
            guard.remove(&cache_key);
        }
    }

    probe
}

async fn generate_with_outetts(
    audio_runtime_dir: &Path,
    models_dir: &Path,
    input_dir: &Path,
    model: &ModelInfo,
    request: &GenerateRequest,
    reference_asset: Option<&InputAsset>,
    used_seed: u32,
    output_path: &Path,
) -> Result<AudioGenerationResult> {
    let probe = probe_outetts_runtime(audio_runtime_dir);
    if !probe.ready {
        bail!("{}", probe.note);
    }

    if let Some(asset) = reference_asset {
        if asset.kind != MediaKind::Audio {
            bail!("OuteTTS voice reference must come from input/audio/.");
        }
        if !probe.supports_audio_reference {
            bail!(
                "OuteTTS speech generation is ready, but audio-reference voice cloning is not. {}",
                probe.note
            );
        }
    }

    let model_path = models_dir.join(&model.relative_path);
    let runner_path = outetts_runner_path(audio_runtime_dir);
    let interpreter = outetts_python_interpreter(audio_runtime_dir).ok_or_else(|| {
        anyhow::anyhow!(
            "OuteTTS needs a usable Python runtime with the local speech dependencies. Set CHATTY_ART_OUTETTS_PYTHON if you want to force a specific interpreter."
        )
    })?;
    let tokenizer_repo = outetts_tokenizer_repo(&model.name).ok_or_else(|| {
        anyhow::anyhow!(
            "Chatty-art could not infer the tokenizer repo for '{}'.",
            model.name
        )
    })?;

    let speaker_audio_path = reference_asset
        .map(|asset| input_dir.join(&asset.relative_path))
        .map(|path| path.to_string_lossy().to_string());
    let segments = build_outetts_segments(request);
    let request_dir = temp_audio_request_dir("outetts", &model.slug, used_seed)?;
    let mut rendered_segments = Vec::new();

    for (index, segment) in segments.iter().enumerate() {
        let segment_output_path = request_dir.join(format!("segment-{index:02}.wav"));
        let request_path = request_dir.join(format!("segment-{index:02}.json"));
        let segment_label = normalized_audio_label(segment);
        let speaker_mode = if speaker_audio_path.is_some() {
            "reference"
        } else if segment_label.is_some() {
            "characterized"
        } else if segments.len() > 1 {
            "random"
        } else {
            "default"
        };
        let segment_seed =
            deterministic_segment_seed(used_seed, "outetts", segment_label.as_deref(), index);
        let request_payload = serde_json::json!({
            "model_path": model_path.clone(),
            "tokenizer_repo": tokenizer_repo,
            "output_path": segment_output_path.clone(),
            "text": segment.literal,
            "temperature": request.settings.temperature,
            "seed": segment_seed,
            "low_vram_mode": request.settings.low_vram_mode,
            "speaker_audio_path": speaker_audio_path.clone(),
            "default_speaker": "en-female-1-neutral",
            "speaker_mode": speaker_mode,
            "voice_characteristics": segment_label
                .as_deref()
                .map(derive_outetts_voice_characteristics),
        });

        fs::write(
            &request_path,
            serde_json::to_vec_pretty(&request_payload)
                .context("failed to serialize OuteTTS runner request")?,
        )
        .context("failed to write OuteTTS runner request file")?;

        let output = run_python_async_with_interpreter(
            &interpreter,
            [
                runner_path.to_string_lossy().to_string(),
                "--request".to_string(),
                request_path.to_string_lossy().to_string(),
            ],
        )
        .await;

        let _ = fs::remove_file(&request_path);
        let output = output?;
        if !segment_output_path.exists() {
            let _ = fs::remove_dir_all(&request_dir);
            bail!(
                "OuteTTS segment {} finished without creating '{}'. Stdout: {} Stderr: {}",
                index + 1,
                segment_output_path.display(),
                output.stdout,
                output.stderr
            );
        }

        rendered_segments.push(RenderedAudioSegment {
            path: segment_output_path,
            same_time_as_previous: index > 0 && segment.same_time_as_previous,
        });
    }

    mix_rendered_audio_segments(&rendered_segments, output_path)?;
    let _ = fs::remove_dir_all(&request_dir);

    Ok(AudioGenerationResult {
        mime: MediaKind::Audio.output_mime().to_string(),
        note: if reference_asset.is_some() {
            format!(
                "Generated locally with the OuteTTS speech runtime using an audio voice reference across {} segment(s).",
                segments.len()
            )
        } else {
            let has_named_cast = segments
                .iter()
                .any(|segment| normalized_audio_label(segment).is_some());
            let voice_note = if has_named_cast {
                "using deterministic per-label character casting"
            } else if segments.len() > 1 {
                "using stable per-segment voice variation"
            } else {
                "using the built-in default speaker"
            };
            format!(
                "Generated locally with the OuteTTS speech runtime {}, across {} segment(s).",
                voice_note,
                segments.len()
            )
        },
    })
}

async fn generate_with_stable_audio(
    audio_runtime_dir: &Path,
    models_dir: &Path,
    model: &ModelInfo,
    request: &GenerateRequest,
    reference_asset: Option<&InputAsset>,
    used_seed: u32,
    output_path: &Path,
) -> Result<AudioGenerationResult> {
    if reference_asset.is_some() {
        bail!(
            "Stable Audio Open is currently wired for prompt-driven realism audio, not reference-audio editing."
        );
    }

    let package_dir = models_dir.join(&model.relative_path);
    let probe = probe_stable_audio_runtime(audio_runtime_dir, &package_dir);
    if !probe.ready {
        bail!("{}", probe.note);
    }

    let interpreter = stable_audio_python_interpreter(audio_runtime_dir).ok_or_else(|| {
        anyhow::anyhow!(
            "Stable Audio Open needs a usable Python runtime with the local Stable Audio inference dependencies. Set CHATTY_ART_STABLE_AUDIO_PYTHON if you want to force a specific interpreter."
        )
    })?;
    let runner_path = stable_audio_runner_path(audio_runtime_dir);

    let duration_seconds = request.settings.audio_duration_seconds.max(2);
    let segments = build_stable_audio_segments(request);
    let request_dir = temp_audio_request_dir("stable-audio", &model.slug, used_seed)?;
    let mut rendered_segments = Vec::new();

    for (index, segment) in segments.iter().enumerate() {
        let segment_output_path = request_dir.join(format!("segment-{index:02}.wav"));
        let request_path = request_dir.join(format!("segment-{index:02}.json"));
        let prompt = compose_stable_audio_segment_prompt(request, segment);
        let segment_label = normalized_audio_label(segment);
        let request_payload = serde_json::json!({
            "model_dir": package_dir.clone(),
            "output_path": segment_output_path.clone(),
            "prompt": prompt,
            "negative_prompt": request.negative_prompt.clone(),
            "duration_seconds": duration_seconds,
            "steps": request.settings.steps.clamp(10, 250),
            "cfg_scale": request.settings.cfg_scale,
            "seed": deterministic_segment_seed(
                used_seed,
                "stable-audio",
                segment_label.as_deref(),
                index,
            ),
            "low_vram_mode": request.settings.low_vram_mode,
        });

        fs::write(
            &request_path,
            serde_json::to_vec_pretty(&request_payload)
                .context("failed to serialize Stable Audio runner request")?,
        )
        .context("failed to write Stable Audio runner request file")?;

        let output = run_python_async_with_interpreter(
            &interpreter,
            [
                runner_path.to_string_lossy().to_string(),
                "--request".to_string(),
                request_path.to_string_lossy().to_string(),
            ],
        )
        .await;

        let _ = fs::remove_file(&request_path);
        let output = output?;
        if !segment_output_path.exists() {
            let _ = fs::remove_dir_all(&request_dir);
            bail!(
                "Stable Audio Open segment {} finished without creating '{}'. Stdout: {} Stderr: {}",
                index + 1,
                segment_output_path.display(),
                output.stdout,
                output.stderr
            );
        }

        rendered_segments.push(RenderedAudioSegment {
            path: segment_output_path,
            same_time_as_previous: index > 0 && segment.same_time_as_previous,
        });
    }

    mix_rendered_audio_segments(&rendered_segments, output_path)?;
    let _ = fs::remove_dir_all(&request_dir);

    Ok(AudioGenerationResult {
        mime: MediaKind::Audio.output_mime().to_string(),
        note: format!(
            "Generated locally with Stable Audio Open using the local Stable Audio runtime across {} segment(s). Best for sound effects, ambience, and texture-driven audio. Reusing the same layer name keeps a stable seeded sound identity. Duration target per segment: {}s.",
            segments.len(),
            duration_seconds
        ),
    })
}

fn build_outetts_segments(request: &GenerateRequest) -> Vec<AudioPromptSegment> {
    let segments = request.normalized_audio_segments();
    if !segments.is_empty() {
        return segments;
    }

    let spoken_text = request
        .prepared_spoken_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| request.combined_audio_literal_prompt())
        .unwrap_or_else(|| derive_spoken_text_heuristic(&request.prompt));

    vec![AudioPromptSegment {
        label: None,
        literal: spoken_text,
        same_time_as_previous: false,
    }]
}

fn build_stable_audio_segments(request: &GenerateRequest) -> Vec<AudioPromptSegment> {
    let segments = request.normalized_audio_segments();
    if !segments.is_empty() {
        return segments;
    }

    let literal = request
        .combined_audio_literal_prompt()
        .unwrap_or_else(|| request.prompt.trim().to_string());

    vec![AudioPromptSegment {
        label: None,
        literal,
        same_time_as_previous: false,
    }]
}

fn compose_stable_audio_segment_prompt(
    request: &GenerateRequest,
    segment: &AudioPromptSegment,
) -> String {
    let base = request.prompt.trim();
    let label = normalized_audio_label(segment);
    let literal = segment.literal.trim();
    let layer_direction = label
        .as_deref()
        .map(derive_stable_audio_layer_direction);

    match (base.is_empty(), label.as_deref(), layer_direction.as_deref()) {
        (true, Some(label), maybe_direction) => {
            format!(
                "{label}. {}. Preserve these exact words or sound cues: {literal}",
                maybe_direction.unwrap_or("Keep this recurring layer consistent")
            )
        }
        (true, None, _) => literal.to_string(),
        (false, Some(label), Some(direction)) if base.contains(literal) => {
            format!("{base}. Layer name: {label}. {direction}")
        }
        (false, Some(label), Some(direction)) => {
            format!(
                "{base}. Layer name: {label}. {direction}. Preserve these exact words or sound cues: {literal}"
            )
        }
        (false, Some(label), None) if base.contains(literal) => {
            format!("{base}. Layer name: {label}")
        }
        (false, Some(label), None) => {
            format!("{base}. Layer name: {label}. Preserve these exact words or sound cues: {literal}")
        }
        (false, None, _) if base.contains(literal) => base.to_string(),
        (false, None, _) => {
            format!("{base}. Preserve these exact words or sound cues: {literal}")
        }
    }
}

fn normalized_audio_label(segment: &AudioPromptSegment) -> Option<String> {
    segment
        .label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn deterministic_segment_seed(
    base_seed: u32,
    family_salt: &str,
    label: Option<&str>,
    index: usize,
) -> u32 {
    let family_hash = fnv1a_32(family_salt.as_bytes());
    let mut seed = base_seed ^ family_hash.rotate_left(7);
    if let Some(label) = label {
        seed ^= fnv1a_32(label.as_bytes()).rotate_left(13);
    } else {
        seed = seed.wrapping_add(((index as u32) + 1).wrapping_mul(9_973));
    }

    if seed == 0 { 1 } else { seed }
}

fn fnv1a_32(bytes: &[u8]) -> u32 {
    let mut hash = 0x811C9DC5u32;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn derive_outetts_voice_characteristics(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return "clear neutral speech".to_string();
    }

    let hash = fnv1a_32(trimmed.as_bytes()) as usize;
    let archetype = OUTETTS_VOICE_ARCHETYPES[hash % OUTETTS_VOICE_ARCHETYPES.len()];
    format!(
        "Voice identity for {trimmed}. Keep this same character voice consistent across every segment that uses this same name. {archetype}."
    )
}

fn derive_stable_audio_layer_direction(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return "Keep this recurring sound layer consistent across matching labels".to_string();
    }

    let hash = fnv1a_32(trimmed.as_bytes()) as usize;
    let archetype = STABLE_AUDIO_LAYER_ARCHETYPES[hash % STABLE_AUDIO_LAYER_ARCHETYPES.len()];
    format!(
        "Keep the recurring sound identity for {trimmed} consistent across every segment that reuses this same layer name, and {archetype}"
    )
}

fn temp_audio_request_dir(prefix: &str, model_slug: &str, used_seed: u32) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "chatty-art-{prefix}-{model_slug}-{used_seed}-{}",
        Uuid::new_v4()
    ));
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create temporary audio directory {}", dir.display()))?;
    Ok(dir)
}

fn mix_rendered_audio_segments(
    segments: &[RenderedAudioSegment],
    output_path: &Path,
) -> Result<()> {
    if segments.is_empty() {
        bail!("No rendered audio segments were available to mix.");
    }

    let mut placements = Vec::new();
    let mut sample_rate = None;
    let mut previous_start_frames = 0usize;
    let mut previous_end_frames = 0usize;
    let mut total_frames = 0usize;

    for (index, segment) in segments.iter().enumerate() {
        let clip = decode_wav_as_stereo(&segment.path)?;
        let clip_frames = clip.samples.len() / 2;
        if clip_frames == 0 {
            continue;
        }

        if let Some(expected) = sample_rate {
            if expected != clip.sample_rate {
                bail!(
                    "Audio segment sample rates did not match while mixing. Expected {} Hz, got {} Hz from '{}'.",
                    expected,
                    clip.sample_rate,
                    segment.path.display()
                );
            }
        } else {
            sample_rate = Some(clip.sample_rate);
        }

        let start_frames = if index == 0 {
            0
        } else if segment.same_time_as_previous {
            previous_start_frames
        } else {
            previous_end_frames
        };
        let end_frames = start_frames + clip_frames;
        total_frames = total_frames.max(end_frames);
        previous_start_frames = start_frames;
        previous_end_frames = end_frames;
        placements.push((clip, start_frames));
    }

    let sample_rate = sample_rate.ok_or_else(|| anyhow::anyhow!("No audio segments were decodable."))?;
    let mut mixed = vec![0.0f32; total_frames.saturating_mul(2)];

    for (clip, start_frames) in placements {
        let clip_frames = clip.samples.len() / 2;
        for frame_index in 0..clip_frames {
            let target_index = (start_frames + frame_index) * 2;
            let source_index = frame_index * 2;
            mixed[target_index] += clip.samples[source_index];
            mixed[target_index + 1] += clip.samples[source_index + 1];
        }
    }

    let peak = mixed
        .iter()
        .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    if peak > 0.98 {
        let scale = 0.92 / peak;
        for sample in &mut mixed {
            *sample *= scale;
        }
    }

    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(output_path, spec)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    for sample in mixed {
        writer.write_sample((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
    }
    writer.finalize()?;
    Ok(())
}

fn decode_wav_as_stereo(path: &Path) -> Result<DecodedStereoWav> {
    let mut reader =
        WavReader::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let spec = reader.spec();
    let raw_samples = match spec.sample_format {
        SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("failed to read float samples from {}", path.display()))?,
        SampleFormat::Int => {
            if spec.bits_per_sample <= 16 {
                reader
                    .samples::<i16>()
                    .map(|sample| sample.map(|value| value as f32 / i16::MAX as f32))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .with_context(|| format!("failed to read i16 samples from {}", path.display()))?
            } else {
                let scale = ((1i64 << (spec.bits_per_sample - 1)) - 1) as f32;
                reader
                    .samples::<i32>()
                    .map(|sample| sample.map(|value| value as f32 / scale))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .with_context(|| format!("failed to read i32 samples from {}", path.display()))?
            }
        }
    };

    let channels = spec.channels.max(1) as usize;
    let mut stereo = Vec::with_capacity((raw_samples.len() / channels.max(1)).saturating_mul(2));
    for frame in raw_samples.chunks(channels) {
        let left = frame.first().copied().unwrap_or(0.0);
        let right = if channels == 1 {
            left
        } else {
            frame.get(1).copied().unwrap_or(left)
        };
        stereo.push(left);
        stereo.push(right);
    }

    Ok(DecodedStereoWav {
        sample_rate: spec.sample_rate,
        samples: stereo,
    })
}

fn outetts_source_dir(audio_runtime_dir: &Path) -> PathBuf {
    audio_runtime_dir.join("outetts").join("OuteTTS-main")
}

fn outetts_runner_path(audio_runtime_dir: &Path) -> PathBuf {
    audio_runtime_dir.join("outetts_runner.py")
}

fn stable_audio_source_dir(audio_runtime_dir: &Path) -> PathBuf {
    audio_runtime_dir
        .join("stable_audio_tools")
        .join("stable-audio-tools-main")
}

fn stable_audio_runner_path(audio_runtime_dir: &Path) -> PathBuf {
    audio_runtime_dir.join("stable_audio_runner.py")
}

fn outetts_python_interpreter(audio_runtime_dir: &Path) -> Option<PathBuf> {
    let source_dir = outetts_source_dir(audio_runtime_dir);
    if !source_dir.exists() {
        return None;
    }

    let mut candidates = Vec::new();
    if let Ok(value) = std::env::var("CHATTY_ART_OUTETTS_PYTHON") {
        candidates.push(PathBuf::from(value));
    }

    candidates.extend([
        audio_runtime_dir
            .join("outetts_venv")
            .join("Scripts")
            .join("python.exe"),
        audio_runtime_dir
            .join("outetts_venv")
            .join("bin")
            .join("python"),
        audio_runtime_dir
            .join("outetts")
            .join(".venv")
            .join("Scripts")
            .join("python.exe"),
        audio_runtime_dir
            .join("outetts")
            .join(".venv")
            .join("bin")
            .join("python"),
        audio_runtime_dir
            .join("outetts")
            .join("OuteTTS-main")
            .join(".venv")
            .join("Scripts")
            .join("python.exe"),
        audio_runtime_dir
            .join("outetts")
            .join("OuteTTS-main")
            .join(".venv")
            .join("bin")
            .join("python"),
    ]);

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let python_root = PathBuf::from(local_app_data)
            .join("Programs")
            .join("Python");
        candidates.extend([
            python_root.join("Python312").join("python.exe"),
            python_root.join("Python311").join("python.exe"),
            python_root.join("Python310").join("python.exe"),
        ]);
    }

    let mut deduped = Vec::new();
    for candidate in candidates {
        if candidate.exists()
            && !deduped
                .iter()
                .any(|existing: &PathBuf| existing == &candidate)
        {
            deduped.push(candidate);
        }
    }

    let mut basic_ready = None;
    for candidate in deduped {
        let Some(probe) = probe_outetts_interpreter(&candidate, &source_dir) else {
            continue;
        };
        if probe.ready && probe.supports_audio_reference {
            return Some(candidate);
        }
        if probe.ready && basic_ready.is_none() {
            basic_ready = Some(candidate);
        }
    }

    basic_ready
}

fn stable_audio_python_interpreter(audio_runtime_dir: &Path) -> Option<PathBuf> {
    let source_dir = stable_audio_source_dir(audio_runtime_dir);
    if !source_dir.exists() {
        return None;
    }

    let mut candidates = Vec::new();
    if let Ok(value) = std::env::var("CHATTY_ART_STABLE_AUDIO_PYTHON") {
        candidates.push(PathBuf::from(value));
    }

    candidates.extend([
        audio_runtime_dir
            .join("stable_audio_venv")
            .join("Scripts")
            .join("python.exe"),
        audio_runtime_dir
            .join("stable_audio_venv")
            .join("bin")
            .join("python"),
        audio_runtime_dir
            .join("stable_audio_tools")
            .join(".venv")
            .join("Scripts")
            .join("python.exe"),
        audio_runtime_dir
            .join("stable_audio_tools")
            .join(".venv")
            .join("bin")
            .join("python"),
        audio_runtime_dir
            .join("stable_audio_tools")
            .join("stable-audio-tools-main")
            .join(".venv")
            .join("Scripts")
            .join("python.exe"),
        audio_runtime_dir
            .join("stable_audio_tools")
            .join("stable-audio-tools-main")
            .join(".venv")
            .join("bin")
            .join("python"),
    ]);

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let python_root = PathBuf::from(local_app_data)
            .join("Programs")
            .join("Python");
        candidates.extend([
            python_root.join("Python312").join("python.exe"),
            python_root.join("Python311").join("python.exe"),
            python_root.join("Python310").join("python.exe"),
        ]);
    }

    let mut deduped = Vec::new();
    for candidate in candidates {
        if candidate.exists()
            && !deduped
                .iter()
                .any(|existing: &PathBuf| existing == &candidate)
        {
            deduped.push(candidate);
        }
    }

    let mut cpu_ready = None;
    for candidate in deduped {
        let Some(probe) = probe_stable_audio_interpreter(&candidate, &source_dir) else {
            continue;
        };
        if probe.ready && probe.gpu_available {
            return Some(candidate);
        }
        if probe.ready && cpu_ready.is_none() {
            cpu_ready = Some(candidate);
        }
    }

    cpu_ready
}

fn probe_outetts_interpreter(
    interpreter: &Path,
    source_dir: &Path,
) -> Option<OuteTtsInterpreterProbe> {
    let probe_script = r#"
import importlib
import json
import sys
from pathlib import Path

source_dir = Path(sys.argv[1])
sys.path.insert(0, str(source_dir))

try:
    import torch.distributed as dist
    if not hasattr(dist, "ReduceOp"):
        class _ReduceOp:
            AVG = "avg"
            SUM = "sum"
            MIN = "min"
            MAX = "max"
            PRODUCT = "product"
        dist.ReduceOp = _ReduceOp
except Exception:
    pass

required = [
    "loguru",
    "polars",
    "ftfy",
    "transformers",
    "llama_cpp",
    "huggingface_hub",
    "soundfile",
    "pyloudnorm",
    "MeCab",
    "uroman",
]
reference_optional = [
    "whisper",
]

for module_name in required:
    try:
        importlib.import_module(module_name)
    except Exception:
        print(json.dumps({"ready": False, "supports_audio_reference": False}))
        raise SystemExit(0)

supports_audio_reference = True
for module_name in reference_optional:
    try:
        importlib.import_module(module_name)
    except Exception:
        supports_audio_reference = False

try:
    import outetts  # noqa: F401
except Exception:
    print(json.dumps({"ready": False, "supports_audio_reference": False}))
    raise SystemExit(0)

print(json.dumps({
    "ready": True,
    "supports_audio_reference": supports_audio_reference,
}))
"#;

    let output = run_python_sync_with_interpreter(
        interpreter,
        [
            "-c".to_string(),
            probe_script.to_string(),
            source_dir.to_string_lossy().to_string(),
        ],
    )
    .ok()?;

    serde_json::from_str::<OuteTtsInterpreterProbe>(output.trim()).ok()
}

fn probe_stable_audio_interpreter(
    interpreter: &Path,
    source_dir: &Path,
) -> Option<StableAudioInterpreterProbe> {
    let probe_script = r#"
import importlib
import json
import sys
from pathlib import Path

source_dir = Path(sys.argv[1])
sys.path.insert(0, str(source_dir))

required = [
    "torch",
    "torchaudio",
    "einops",
    "soundfile",
    "safetensors",
    "transformers",
    "k_diffusion",
]

for module_name in required:
    try:
        importlib.import_module(module_name)
    except Exception:
        print(json.dumps({"ready": False, "gpu_available": False}))
        raise SystemExit(0)

try:
    import torch
    import stable_audio_tools  # noqa: F401
    from stable_audio_tools.inference.generation import generate_diffusion_cond  # noqa: F401
    from stable_audio_tools.models.factory import create_model_from_config  # noqa: F401
except Exception:
    print(json.dumps({"ready": False, "gpu_available": False}))
    raise SystemExit(0)

print(json.dumps({
    "ready": True,
    "gpu_available": bool(torch.cuda.is_available()),
}))
"#;

    let output = run_python_sync_with_interpreter(
        interpreter,
        [
            "-c".to_string(),
            probe_script.to_string(),
            source_dir.to_string_lossy().to_string(),
        ],
    )
    .ok()?;

    serde_json::from_str::<StableAudioInterpreterProbe>(output.trim()).ok()
}

fn outetts_tokenizer_repo(model_name: &str) -> Option<&'static str> {
    let lower = model_name.to_ascii_lowercase();
    if lower.contains("llama-outetts-1.0-1b")
        || (lower.contains("outetts") && lower.contains("1.0") && lower.contains("1b"))
    {
        return Some("OuteAI/Llama-OuteTTS-1.0-1B");
    }
    if lower.contains("outetts-1.0-0.6b") || (lower.contains("outetts") && lower.contains("0.6b")) {
        return Some("OuteAI/OuteTTS-1.0-0.6B");
    }
    None
}

fn parse_runtime_probe(output: &str) -> Option<RuntimeProbe> {
    for line in output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(probe) = serde_json::from_str::<RuntimeProbe>(trimmed) {
                return Some(probe);
            }
        }
    }

    serde_json::from_str::<RuntimeProbe>(output.trim()).ok()
}

fn outetts_probe_cache() -> &'static Mutex<HashMap<String, RuntimeProbe>> {
    static CACHE: OnceLock<Mutex<HashMap<String, RuntimeProbe>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn stable_audio_probe_cache() -> &'static Mutex<HashMap<String, RuntimeProbe>> {
    static CACHE: OnceLock<Mutex<HashMap<String, RuntimeProbe>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn run_python_sync_with_interpreter<I, S>(interpreter: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();

    let output = Command::new(interpreter)
        .args(args.iter())
        .stdin(Stdio::null())
        .output()
        .context("failed to start the requested Python interpreter")?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    Err(anyhow::anyhow!(
        "{}",
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    ))
}

struct PythonRunResult {
    stdout: String,
    stderr: String,
}

async fn run_python_async_with_interpreter<I, S>(
    interpreter: &Path,
    args: I,
) -> Result<PythonRunResult>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();

    let output = tokio::process::Command::new(interpreter)
        .args(args.iter())
        .stdin(Stdio::null())
        .output()
        .await
        .context("failed to start the requested Python interpreter")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        return Ok(PythonRunResult { stdout, stderr });
    }

    Err(anyhow::anyhow!(
        "The local Python runtime failed. Stdout: {} Stderr: {}",
        stdout.trim(),
        stderr.trim()
    ))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        compose_stable_audio_segment_prompt, detect_audio_runtime_package_support,
        deterministic_segment_seed, derive_outetts_voice_characteristics,
        outetts_tokenizer_repo, parse_runtime_probe,
    };
    use std::{fs, path::PathBuf};

    use crate::types::{AudioPromptSegment, GenerateRequest, GenerationSettings, MediaKind, PromptAssistMode, ReferenceIntent, ResolutionPreset, VideoResolutionPreset};

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("chatty-art-audio-runtime-{label}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn maps_outetts_1b_model_to_repo() {
        assert_eq!(
            outetts_tokenizer_repo("Llama-OuteTTS-1.0-1B-FP16"),
            Some("OuteAI/Llama-OuteTTS-1.0-1B")
        );
    }

    #[test]
    fn maps_outetts_0_6b_model_to_repo() {
        assert_eq!(
            outetts_tokenizer_repo("OuteTTS-1.0-0.6B-FP16"),
            Some("OuteAI/OuteTTS-1.0-0.6B")
        );
    }

    #[test]
    fn parses_runtime_probe_from_last_json_line() {
        let output = "warning from python\n{\"ready\":true,\"supports_audio_reference\":false,\"note\":\"ok\"}\n";
        let probe = parse_runtime_probe(output).expect("probe should parse");
        assert!(probe.ready);
        assert!(!probe.supports_audio_reference);
        assert_eq!(probe.note, "ok");
    }

    #[test]
    fn parses_runtime_probe_from_direct_json_output() {
        let probe = parse_runtime_probe(
            "{\"ready\":false,\"supports_audio_reference\":true,\"note\":\"hi\"}",
        )
        .expect("probe should parse");
        assert!(!probe.ready);
        assert!(probe.supports_audio_reference);
        assert_eq!(probe.note, "hi");
    }

    #[test]
    fn detects_stable_audio_package_directory() {
        let package_dir = temp_dir("stable-audio");
        let audio_runtime_dir = temp_dir("stable-audio-runtime");
        fs::write(
            package_dir.join("model_index.json"),
            "{\"_class_name\":\"StableAudioPipeline\"}",
        )
        .unwrap();
        fs::write(package_dir.join("model_config.json"), "{}").unwrap();
        fs::write(package_dir.join("model.safetensors"), b"").unwrap();
        for folder in [
            "projection_model",
            "scheduler",
            "text_encoder",
            "tokenizer",
            "transformer",
            "vae",
        ] {
            fs::create_dir_all(package_dir.join(folder)).unwrap();
        }
        fs::create_dir_all(
            audio_runtime_dir
                .join("stable_audio_tools")
                .join("stable-audio-tools-main"),
        )
        .unwrap();
        fs::write(audio_runtime_dir.join("stable_audio_runner.py"), "# test\n").unwrap();

        let support = detect_audio_runtime_package_support(
            "stable-audio-open-1.0",
            &package_dir,
            &audio_runtime_dir,
        )
        .expect("stable audio package should be detected");

        assert_eq!(support.family, "Stable Audio Open");
        assert_eq!(
            support.supported_kinds,
            vec![crate::types::MediaKind::Audio]
        );
        assert!(!support.runtime_supported);
        assert!(
            support.compatibility_note.contains("Stable Audio")
                || support.compatibility_note.contains("stable audio")
        );
    }

    #[test]
    fn deterministic_seed_reuses_same_label_identity() {
        let seed_a = deterministic_segment_seed(1234, "outetts", Some("Narrator"), 0);
        let seed_b = deterministic_segment_seed(1234, "outetts", Some("Narrator"), 7);
        let seed_c = deterministic_segment_seed(1234, "outetts", Some("Caller"), 0);

        assert_eq!(seed_a, seed_b);
        assert_ne!(seed_a, seed_c);
    }

    #[test]
    fn outetts_voice_characteristics_include_label() {
        let description = derive_outetts_voice_characteristics("Narrator");
        assert!(description.contains("Narrator"));
        assert!(description.contains("same character voice"));
    }

    #[test]
    fn stable_audio_prompt_uses_layer_identity_language() {
        let request = GenerateRequest {
            prompt: "rainy city alley, neon reflections".to_string(),
            negative_prompt: None,
            prompt_assist: PromptAssistMode::Off,
            model: "stable-audio-open-1.0".to_string(),
            kind: MediaKind::Audio,
            style: crate::types::GenerationStyle::Realism,
            settings: GenerationSettings {
                temperature: 0.6,
                steps: 24,
                cfg_scale: 6.0,
                resolution: ResolutionPreset::Square512,
                video_resolution: VideoResolutionPreset::Square256,
                video_duration_seconds: 2,
                video_fps: 8,
                audio_duration_seconds: 10,
                low_vram_mode: true,
                seed: Some(1234),
            },
            reference_asset: None,
            reference_intent: ReferenceIntent::Guide,
            end_reference_asset: None,
            control_reference_asset: None,
            prepared_prompt: None,
            prepared_negative_prompt: None,
            prepared_note: None,
            prepared_interpreter_model: None,
            prepared_spoken_text: None,
            audio_literal_prompt: None,
            audio_segments: vec![],
        };

        let prompt = compose_stable_audio_segment_prompt(
            &request,
            &AudioPromptSegment {
                label: Some("Rain Bed".to_string()),
                literal: "gentle rain, wet pavement hiss".to_string(),
                same_time_as_previous: false,
            },
        );

        assert!(prompt.contains("Layer name: Rain Bed"));
        assert!(prompt.contains("same layer name"));
        assert!(prompt.contains("gentle rain, wet pavement hiss"));
    }
}
