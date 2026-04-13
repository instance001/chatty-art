use std::{
    cmp::Ordering,
    collections::HashSet,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use tokio::{fs, process::Command, time::timeout};

use crate::{
    render::{
        AudioLayerPlan, AudioPlan, ImagePlan, MotionShapePlan, SceneRole, ShapeKind, ShapePlan,
        VideoPlan, Waveform,
    },
    types::{
        GenerationSettings, GenerationStyle, InputAsset, MediaKind, ModelInfo, PromptAssistMode,
        ReferenceIntent, ReferenceSummary, VideoResolutionPreset,
    },
};

enum InvokeError {
    Runtime(anyhow::Error),
    Parse {
        raw_output: String,
        stderr: String,
        extracted_json: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannerTrace {
    pub used_fallback: bool,
    pub raw_output: String,
    pub stderr: String,
    pub extracted_json: Option<String>,
}

pub struct PlannedPlan<T> {
    pub plan: T,
    pub note: String,
    pub trace: PlannerTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAssistBrief {
    pub expanded_prompt: String,
    pub negative_prompt: String,
    pub assumptions: Vec<String>,
    pub focus_tags: Vec<String>,
    #[serde(default)]
    pub spoken_text: Option<String>,
}

pub struct CompiledPrompt {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub spoken_text: Option<String>,
    pub note: String,
    pub brief: PromptAssistBrief,
    pub trace: PlannerTrace,
    pub used_original_prompt: bool,
}

struct InvokedPlan<T> {
    pub value: T,
    pub raw_output: String,
    pub stderr: String,
    pub extracted_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneGraphPlan {
    pub background_top: String,
    pub background_bottom: String,
    pub accent: String,
    pub horizon_y: f32,
    pub ground_y: f32,
    pub focus_x: f32,
    pub focus_y: f32,
    pub elements: Vec<SceneElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MotionSceneGraphPlan {
    #[serde(flatten)]
    pub scene: SceneGraphPlan,
    pub fps: u16,
    pub frames: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneElement {
    pub motif: SceneMotif,
    pub role: SceneRole,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub emphasis: f32,
    pub rotation: f32,
    #[serde(default)]
    pub motion: MotionCue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SceneMotif {
    Figure,
    Creature,
    Pair,
    Seat,
    Swing,
    Tree,
    Water,
    Path,
    Bench,
    Sun,
    Moon,
    Cloud,
    StarCluster,
    Hill,
    Structure,
    Frame,
    Accent,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MotionCue {
    #[default]
    Still,
    Bob,
    Sway,
    Drift,
    Pulse,
    Orbit,
    Ripple,
    Glimmer,
}

#[derive(Debug, Clone, Default)]
struct SceneIntentHints {
    wants_creature: bool,
    wants_person: bool,
    wants_pair: bool,
    wants_swing: bool,
    wants_tree: bool,
    wants_water: bool,
    wants_bench: bool,
    wants_path: bool,
    wants_sun: bool,
    wants_moon: bool,
    wants_cloud: bool,
    wants_hill: bool,
    wants_field: bool,
}

#[derive(Debug, Clone)]
struct PlannedShape {
    base: ShapePlan,
    motion: MotionCue,
}

#[derive(Debug, Clone)]
struct ScenePalette {
    top: String,
    bottom: String,
    accent: String,
    shadow: String,
    light: String,
    foliage: String,
    earth: String,
    water: String,
    wood: String,
    cloud: String,
}

pub async fn compile_prompt(
    runtime_dir: &Path,
    model_dir: &Path,
    model: &ModelInfo,
    user_prompt: &str,
    user_negative_prompt: Option<&str>,
    style: GenerationStyle,
    kind: MediaKind,
    mode: PromptAssistMode,
    reference: Option<&ReferenceSummary>,
    supports_voice_output: bool,
    seed: u32,
) -> Result<CompiledPrompt> {
    let schema = prompt_assist_schema(kind, supports_voice_output);
    let prompt = prompt_assist_prompt(
        user_prompt,
        user_negative_prompt,
        style,
        kind,
        mode,
        reference,
        supports_voice_output,
    );
    let max_tokens = match mode {
        PromptAssistMode::Off => 0,
        PromptAssistMode::Gentle => 280,
        PromptAssistMode::Strong => 440,
    };

    match invoke_llama_json::<PromptAssistBrief>(
        runtime_dir,
        model_dir.join(relative_to_native_path(&model.relative_path)),
        model,
        &compiler_settings(),
        seed,
        max_tokens,
        prompt,
        schema,
        None,
        "prompt-compiler",
    )
    .await
    {
        Ok(invoked) => {
            let brief = normalize_prompt_assist_brief(invoked.value);
            let spoken_text = if kind == MediaKind::Audio && supports_voice_output {
                brief
                    .spoken_text
                    .as_deref()
                    .and_then(optional_text)
                    .map(str::to_string)
                    .or_else(|| Some(derive_spoken_text_heuristic(user_prompt)))
            } else {
                None
            };
            let compiled_prompt = if kind == MediaKind::Audio && supports_voice_output {
                optional_text(&brief.expanded_prompt)
                    .map(|value| polish_compiled_prompt(value, &brief.focus_tags, style, kind))
                    .or_else(|| {
                        derive_speech_direction_heuristic(user_prompt, spoken_text.as_deref())
                    })
                    .unwrap_or_default()
            } else if brief.expanded_prompt.trim().is_empty() {
                user_prompt.trim().to_string()
            } else {
                polish_compiled_prompt(&brief.expanded_prompt, &brief.focus_tags, style, kind)
            };
            let negative_prompt =
                merge_negative_prompts(user_negative_prompt, optional_text(&brief.negative_prompt));
            Ok(CompiledPrompt {
                prompt: compiled_prompt,
                negative_prompt,
                spoken_text,
                note: prompt_assist_note(mode, &brief, kind, supports_voice_output),
                brief,
                trace: PlannerTrace {
                    used_fallback: false,
                    raw_output: invoked.raw_output,
                    stderr: invoked.stderr,
                    extracted_json: invoked.extracted_json,
                },
                used_original_prompt: false,
            })
        }
        Err(InvokeError::Parse {
            raw_output,
            stderr,
            extracted_json,
        }) => {
            let spoken_text = if kind == MediaKind::Audio && supports_voice_output {
                Some(derive_spoken_text_heuristic(user_prompt))
            } else {
                None
            };
            let fallback_prompt = if kind == MediaKind::Audio && supports_voice_output {
                derive_speech_direction_heuristic(user_prompt, spoken_text.as_deref())
                    .unwrap_or_default()
            } else {
                user_prompt.trim().to_string()
            };
            Ok(CompiledPrompt {
                prompt: fallback_prompt.clone(),
                negative_prompt: user_negative_prompt
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                spoken_text: spoken_text.clone(),
                note: "Prompt Assist could not extract a clean brief, so Chatty-art fell back to a simpler local handoff."
                    .to_string(),
                brief: PromptAssistBrief {
                    expanded_prompt: fallback_prompt,
                    negative_prompt: user_negative_prompt.unwrap_or_default().trim().to_string(),
                    assumptions: Vec::new(),
                    focus_tags: Vec::new(),
                    spoken_text,
                },
                trace: PlannerTrace {
                    used_fallback: true,
                    raw_output,
                    stderr,
                    extracted_json,
                },
                used_original_prompt: true,
            })
        }
        Err(InvokeError::Runtime(error)) => Err(error),
    }
}

pub async fn build_reference_summary(
    asset_path: &Path,
    asset: &InputAsset,
    intent: ReferenceIntent,
) -> Result<ReferenceSummary> {
    let palette = if asset.kind == MediaKind::Image {
        extract_image_palette(asset_path)?
    } else {
        Vec::new()
    };

    let note = match asset.kind {
        MediaKind::Image => {
            let subject = match intent {
                ReferenceIntent::Guide => "Using image guide",
                ReferenceIntent::Edit => "Editing from image source",
            };
            if palette.is_empty() {
                format!("{subject} '{}'.", asset.name)
            } else {
                format!(
                    "{subject} '{}' with palette {}.",
                    asset.name,
                    palette.join(", ")
                )
            }
        }
        MediaKind::Audio => match intent {
            ReferenceIntent::Guide => format!("Using audio guide '{}'.", asset.name),
            ReferenceIntent::Edit => format!("Editing from audio source '{}'.", asset.name),
        },
        MediaKind::Gif | MediaKind::Video => match intent {
            ReferenceIntent::Guide => format!("Using motion guide '{}'.", asset.name),
            ReferenceIntent::Edit => format!("Editing from motion source '{}'.", asset.name),
        },
    };

    Ok(ReferenceSummary {
        name: asset.name.clone(),
        relative_path: asset.relative_path.clone(),
        kind: asset.kind,
        palette,
        intent,
        note,
    })
}

pub async fn build_image_plan(
    runtime_dir: &Path,
    model_dir: &Path,
    model: &ModelInfo,
    prompt: &str,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
) -> Result<PlannedPlan<ImagePlan>> {
    let max_elements = max_scene_elements(settings, MediaKind::Image);
    let shape_budget = image_shape_budget(settings);
    let max_tokens = 320 + max_elements as usize * 120;
    let schema = image_scene_graph_schema(max_elements);
    let prompt_text = image_scene_graph_prompt(prompt, settings, reference, max_elements);
    let prompt_fingerprint = prompt_hash(prompt_text.as_str());

    match invoke_llama_json::<SceneGraphPlan>(
        runtime_dir,
        model_dir.join(relative_to_native_path(&model.relative_path)),
        model,
        settings,
        seed,
        max_tokens,
        prompt_text.clone(),
        schema,
        reference,
        "image",
    )
    .await
    {
        Ok(invoked) => {
            let scene_graph = repair_scene_graph(
                normalize_scene_graph(invoked.value, max_elements as usize),
                &prompt_text,
            );
            Ok(PlannedPlan {
                plan: compose_image_plan(&scene_graph, settings, seed, reference, shape_budget),
                note: "The local artist model planned a reusable scene graph, then Chatty-art translated it into motif-based shapes.".to_string(),
                trace: PlannerTrace {
                    used_fallback: false,
                    raw_output: invoked.raw_output,
                    stderr: invoked.stderr,
                    extracted_json: invoked.extracted_json,
                },
            })
        }
        Err(InvokeError::Parse {
            raw_output,
            stderr,
            extracted_json,
        }) => Ok(PlannedPlan {
            plan: fallback_image_plan(prompt_fingerprint, settings, seed, reference),
            note:
                "The model returned loose JSON, so Chatty-art used a deterministic fallback scene."
                    .to_string(),
            trace: PlannerTrace {
                used_fallback: true,
                raw_output,
                stderr,
                extracted_json,
            },
        }),
        Err(InvokeError::Runtime(error)) => Err(error),
    }
}

pub async fn build_video_plan(
    runtime_dir: &Path,
    model_dir: &Path,
    model: &ModelInfo,
    prompt: &str,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
) -> Result<PlannedPlan<VideoPlan>> {
    let max_elements = max_scene_elements(settings, MediaKind::Video);
    let shape_budget = video_shape_budget(settings);
    let max_tokens = 420 + max_elements as usize * 140;
    let schema = video_scene_graph_schema(max_elements);
    let prompt_text = video_scene_graph_prompt(prompt, settings, reference, max_elements);
    let prompt_fingerprint = prompt_hash(prompt_text.as_str());

    match invoke_llama_json::<MotionSceneGraphPlan>(
        runtime_dir,
        model_dir.join(relative_to_native_path(&model.relative_path)),
        model,
        settings,
        seed,
        max_tokens,
        prompt_text.clone(),
        schema,
        reference,
        "video",
    )
    .await
    {
        Ok(invoked) => {
            let scene_graph = repair_motion_scene_graph(
                normalize_motion_scene_graph(invoked.value, max_elements as usize),
                &prompt_text,
            );
            Ok(PlannedPlan {
                plan: compose_video_plan(&scene_graph, settings, seed, reference, shape_budget),
                note: "The local artist model planned a reusable motion scene graph, then Chatty-art translated it into motif-based animated shapes.".to_string(),
                trace: PlannerTrace {
                    used_fallback: false,
                    raw_output: invoked.raw_output,
                    stderr: invoked.stderr,
                    extracted_json: invoked.extracted_json,
                },
            })
        }
        Err(InvokeError::Parse {
            raw_output,
            stderr,
            extracted_json,
        }) => Ok(PlannedPlan {
            plan: fallback_video_plan(prompt_fingerprint, settings, seed, reference),
            note: "The model returned loose JSON, so Chatty-art used a deterministic fallback motion plan."
                .to_string(),
            trace: PlannerTrace {
                used_fallback: true,
                raw_output,
                stderr,
                extracted_json,
            },
        }),
        Err(InvokeError::Runtime(error)) => Err(error),
    }
}

pub async fn build_audio_plan(
    runtime_dir: &Path,
    model_dir: &Path,
    model: &ModelInfo,
    prompt: &str,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
) -> Result<PlannedPlan<AudioPlan>> {
    let max_layers = (2 + settings.steps / 30).clamp(2, 4);
    let max_tokens = 220 + max_layers as usize * 120;
    let schema = audio_schema(max_layers);
    let prompt = audio_prompt(prompt, settings, reference, max_layers);
    let prompt_fingerprint = prompt_hash(prompt.as_str());

    match invoke_llama_json::<AudioPlan>(
        runtime_dir,
        model_dir.join(relative_to_native_path(&model.relative_path)),
        model,
        settings,
        seed,
        max_tokens,
        prompt,
        schema,
        reference,
        "audio",
    )
    .await
    {
        Ok(invoked) => Ok(PlannedPlan {
            plan: normalize_audio_plan(invoked.value),
            note: String::new(),
            trace: PlannerTrace {
                used_fallback: false,
                raw_output: invoked.raw_output,
                stderr: invoked.stderr,
                extracted_json: invoked.extracted_json,
            },
        }),
        Err(InvokeError::Parse {
            raw_output,
            stderr,
            extracted_json,
        }) => Ok(PlannedPlan {
            plan: fallback_audio_plan(prompt_fingerprint, settings, seed, reference),
            note: "The model returned loose JSON, so Chatty-art used a deterministic fallback sound plan."
                .to_string(),
            trace: PlannerTrace {
                used_fallback: true,
                raw_output,
                stderr,
                extracted_json,
            },
        }),
        Err(InvokeError::Runtime(error)) => Err(error),
    }
}

async fn invoke_llama_json<T: DeserializeOwned>(
    runtime_dir: &Path,
    model_path: PathBuf,
    model: &ModelInfo,
    settings: &GenerationSettings,
    seed: u32,
    max_tokens: usize,
    prompt_text: String,
    schema: Value,
    reference: Option<&ReferenceSummary>,
    temp_stem: &str,
) -> std::result::Result<InvokedPlan<T>, InvokeError> {
    let temp_root = std::env::temp_dir().join("chatty-art");
    fs::create_dir_all(&temp_root)
        .await
        .map_err(|error| InvokeError::Runtime(error.into()))?;

    let prompt_file = temp_root.join(format!("{}-{}-prompt.txt", temp_stem, seed));
    let schema_file = temp_root.join(format!("{}-{}-schema.json", temp_stem, seed));
    fs::write(&prompt_file, prompt_text.as_bytes())
        .await
        .map_err(|error| InvokeError::Runtime(error.into()))?;
    fs::write(&schema_file, schema.to_string().as_bytes())
        .await
        .map_err(|error| InvokeError::Runtime(error.into()))?;

    let executable = runtime_dir.join("llama-cli.exe");
    let mut command = Command::new(&executable);
    command
        .current_dir(runtime_dir)
        .arg("-m")
        .arg(&model_path)
        .arg("--file")
        .arg(&prompt_file)
        .arg("--json-schema-file")
        .arg(&schema_file)
        .arg("-n")
        .arg(max_tokens.to_string())
        .arg("-s")
        .arg(seed.to_string())
        .arg("--temp")
        .arg(planner_temperature(settings).to_string())
        .arg("--ctx-size")
        .arg("4096")
        .arg("--gpu-layers")
        .arg("999")
        .arg("--device")
        .arg("Vulkan0")
        .arg("--fit")
        .arg("on")
        .arg("--single-turn")
        .arg("--no-display-prompt")
        .arg("--no-jinja")
        .arg("--log-disable");

    if let Some(reference) = reference {
        let relative = relative_to_native_path(&reference.relative_path);
        let input_path = runtime_dir
            .parent()
            .map(|root| root.join("input").join(relative))
            .ok_or_else(|| InvokeError::Runtime(anyhow!("could not resolve input path")))?;

        if reference.kind == MediaKind::Image && model.supports_image_reference {
            if let Some(mmproj) = &model.mmproj_path {
                command.arg("--mmproj").arg(
                    runtime_dir
                        .parent()
                        .map(|root| root.join("models").join(relative_to_native_path(mmproj)))
                        .ok_or_else(|| InvokeError::Runtime(anyhow!("missing mmproj base path")))?,
                );
            }
            command.arg("--image").arg(input_path);
        } else if reference.kind == MediaKind::Audio && model.supports_audio_reference {
            command.arg("--audio").arg(input_path);
        }
    }

    let output = timeout(Duration::from_secs(600), command.output())
        .await
        .map_err(|_| InvokeError::Runtime(anyhow!("model planning timed out after 10 minutes")))?
        .map_err(|error| InvokeError::Runtime(error.into()))?;

    let _ = fs::remove_file(&prompt_file).await;
    let _ = fs::remove_file(&schema_file).await;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let extracted = extract_json_object(&stdout);

    match extracted
        .as_deref()
        .and_then(|json| serde_json::from_str::<T>(json).ok())
    {
        Some(value) => Ok(InvokedPlan {
            value,
            raw_output: stdout,
            stderr,
            extracted_json: extracted,
        }),
        None if output.status.success() => Err(InvokeError::Parse {
            raw_output: stdout,
            stderr,
            extracted_json: extracted,
        }),
        None => Err(InvokeError::Runtime(anyhow!(
            "llama.cpp exited with {}. {}",
            output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "no exit code".to_string()),
            stderr.trim()
        ))),
    }
}

fn extract_json_object(output: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in output.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match character {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => {
                if start.is_none() {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start_index) = start {
                        return Some(output[start_index..=index].to_string());
                    }
                }
            }
            _ => {}
        }
    }

    None
}

fn prompt_assist_schema(kind: MediaKind, supports_voice_output: bool) -> Value {
    if kind == MediaKind::Audio && supports_voice_output {
        return json!({
            "type": "object",
            "properties": {
                "expanded_prompt": { "type": "string" },
                "spoken_text": { "type": "string" },
                "negative_prompt": { "type": "string" },
                "assumptions": {
                    "type": "array",
                    "maxItems": 6,
                    "items": { "type": "string" }
                },
                "focus_tags": {
                    "type": "array",
                    "maxItems": 10,
                    "items": { "type": "string" }
                }
            },
            "required": ["expanded_prompt", "spoken_text", "negative_prompt", "assumptions", "focus_tags"],
            "additionalProperties": false
        });
    }

    json!({
        "type": "object",
        "properties": {
            "expanded_prompt": { "type": "string" },
            "negative_prompt": { "type": "string" },
            "assumptions": {
                "type": "array",
                "maxItems": 6,
                "items": { "type": "string" }
            },
            "focus_tags": {
                "type": "array",
                "maxItems": 10,
                "items": { "type": "string" }
            }
        },
        "required": ["expanded_prompt", "negative_prompt", "assumptions", "focus_tags"],
        "additionalProperties": false
    })
}

fn image_scene_graph_schema(max_elements: u32) -> Value {
    json!({
        "type": "object",
        "properties": {
            "background_top": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "background_bottom": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "accent": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "horizon_y": { "type": "number", "minimum": 0.24, "maximum": 0.72 },
            "ground_y": { "type": "number", "minimum": 0.58, "maximum": 0.94 },
            "focus_x": { "type": "number", "minimum": 0.1, "maximum": 0.9 },
            "focus_y": { "type": "number", "minimum": 0.15, "maximum": 0.85 },
            "elements": {
                "type": "array",
                "minItems": 3,
                "maxItems": max_elements,
                "items": scene_element_schema(false)
            }
        },
        "required": [
            "background_top",
            "background_bottom",
            "accent",
            "horizon_y",
            "ground_y",
            "focus_x",
            "focus_y",
            "elements"
        ],
        "additionalProperties": false
    })
}

fn video_scene_graph_schema(max_elements: u32) -> Value {
    json!({
        "type": "object",
        "properties": {
            "background_top": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "background_bottom": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "accent": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "horizon_y": { "type": "number", "minimum": 0.24, "maximum": 0.72 },
            "ground_y": { "type": "number", "minimum": 0.58, "maximum": 0.94 },
            "focus_x": { "type": "number", "minimum": 0.1, "maximum": 0.9 },
            "focus_y": { "type": "number", "minimum": 0.15, "maximum": 0.85 },
            "fps": { "type": "integer", "minimum": 8, "maximum": 24 },
            "frames": { "type": "integer", "minimum": 16, "maximum": 480 },
            "elements": {
                "type": "array",
                "minItems": 3,
                "maxItems": max_elements,
                "items": scene_element_schema(true)
            }
        },
        "required": [
            "background_top",
            "background_bottom",
            "accent",
            "horizon_y",
            "ground_y",
            "focus_x",
            "focus_y",
            "fps",
            "frames",
            "elements"
        ],
        "additionalProperties": false
    })
}

fn scene_element_schema(include_motion: bool) -> Value {
    let mut properties = serde_json::Map::from_iter([
        (
            "motif".to_string(),
            json!({
                    "type": "string",
                    "enum": [
                        "figure",
                        "creature",
                        "pair",
                        "seat",
                        "swing",
                    "tree",
                    "water",
                    "path",
                    "bench",
                    "sun",
                    "moon",
                    "cloud",
                    "star_cluster",
                    "hill",
                    "structure",
                    "frame",
                    "accent"
                ]
            }),
        ),
        (
            "role".to_string(),
            json!({
                "type": "string",
                "enum": [
                    "background",
                    "horizon",
                    "ground",
                    "subject",
                    "celestial",
                    "reflection",
                    "detail"
                ]
            }),
        ),
        (
            "x".to_string(),
            json!({ "type": "number", "minimum": 0.08, "maximum": 0.92 }),
        ),
        (
            "y".to_string(),
            json!({ "type": "number", "minimum": 0.08, "maximum": 0.92 }),
        ),
        (
            "scale".to_string(),
            json!({ "type": "number", "minimum": 0.2, "maximum": 1.0 }),
        ),
        (
            "emphasis".to_string(),
            json!({ "type": "number", "minimum": 0.2, "maximum": 1.0 }),
        ),
        (
            "rotation".to_string(),
            json!({ "type": "number", "minimum": -180.0, "maximum": 180.0 }),
        ),
    ]);
    let mut required = vec!["motif", "role", "x", "y", "scale", "emphasis", "rotation"];

    if include_motion {
        properties.insert(
            "motion".to_string(),
            json!({
                "type": "string",
                "enum": [
                    "still",
                    "bob",
                    "sway",
                    "drift",
                    "pulse",
                    "orbit",
                    "ripple",
                    "glimmer"
                ]
            }),
        );
        required.push("motion");
    }

    Value::Object(serde_json::Map::from_iter([
        ("type".to_string(), json!("object")),
        ("properties".to_string(), Value::Object(properties)),
        (
            "required".to_string(),
            Value::Array(required.into_iter().map(|value| json!(value)).collect()),
        ),
        ("additionalProperties".to_string(), json!(false)),
    ]))
}

fn audio_schema(max_layers: u32) -> Value {
    json!({
        "type": "object",
        "properties": {
            "bpm": { "type": "integer", "minimum": 60, "maximum": 160 },
            "duration_seconds": { "type": "number", "minimum": 3.0, "maximum": 10.0 },
            "layers": {
                "type": "array",
                "minItems": 2,
                "maxItems": max_layers,
                "items": {
                    "type": "object",
                    "properties": {
                        "wave": { "type": "string", "enum": ["sine", "triangle", "square", "saw"] },
                        "gain": { "type": "number", "minimum": 0.08, "maximum": 0.45 },
                        "pan": { "type": "number", "minimum": -1.0, "maximum": 1.0 },
                        "octave": { "type": "integer", "minimum": 2, "maximum": 6 },
                        "notes": {
                            "type": "array",
                            "minItems": 4,
                            "maxItems": 12,
                            "items": { "type": "integer", "minimum": 0, "maximum": 11 }
                        },
                        "rhythm": {
                            "type": "array",
                            "minItems": 4,
                            "maxItems": 12,
                            "items": { "type": "number", "minimum": 0.25, "maximum": 2.0 }
                        }
                    },
                    "required": ["wave", "gain", "pan", "octave", "notes", "rhythm"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["bpm", "duration_seconds", "layers"],
        "additionalProperties": false
    })
}

#[cfg(test)]
fn shape_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kind": { "type": "string", "enum": ["circle", "rectangle", "line", "ring"] },
            "role": {
                "type": "string",
                "enum": [
                    "background",
                    "horizon",
                    "ground",
                    "subject",
                    "celestial",
                    "reflection",
                    "detail"
                ]
            },
            "x": { "type": "number", "minimum": 0.05, "maximum": 0.95 },
            "y": { "type": "number", "minimum": 0.05, "maximum": 0.95 },
            "size": { "type": "number", "minimum": 0.05, "maximum": 0.6 },
            "aspect": { "type": "number", "minimum": 0.2, "maximum": 2.2 },
            "rotation": { "type": "number", "minimum": 0.0, "maximum": 360.0 },
            "color": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "secondary_color": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "opacity": { "type": "number", "minimum": 0.15, "maximum": 1.0 }
        },
        "required": ["kind", "x", "y", "size", "aspect", "rotation", "color", "secondary_color", "opacity"],
        "additionalProperties": false
    })
}

fn prompt_assist_prompt(
    user_prompt: &str,
    user_negative_prompt: Option<&str>,
    style: GenerationStyle,
    kind: MediaKind,
    mode: PromptAssistMode,
    reference: Option<&ReferenceSummary>,
    supports_voice_output: bool,
) -> String {
    if kind == MediaKind::Audio && supports_voice_output {
        return format!(
            "You are Prompt Assist for a local speech generation tool.\nReturn only JSON matching the schema.\nSeparate the request into two different lanes.\nspoken_text: only the exact words that should be spoken aloud.\nexpanded_prompt: delivery direction only, such as tone, pacing, accent, age, gender, emotion, microphone feel, pauses, or emphasis.\nDo not include the spoken words inside expanded_prompt.\nIf the user supplied quoted dialogue or an explicit script, preserve that wording closely in spoken_text.\nIf the user described the kind of line they want but did not supply exact wording, write a short natural line that fulfills the request.\nKeep spoken_text concise and human-sounding.\nThe negative_prompt should be a concise comma-separated list of speech problems to avoid, or an empty string if not needed.\nfocus_tags should be short speech-direction cues.\nAssist strength: {}.\nReference: {}.\nOriginal negative prompt: {}.\nOriginal user prompt: {}",
            prompt_assist_strength(mode),
            reference_summary(reference),
            user_negative_prompt.unwrap_or("None."),
            user_prompt.trim()
        );
    }

    format!(
        "You are Prompt Assist for a local creative generation tool.\nReturn only JSON matching the schema.\nExpand short human prompts into a compact generator-ready brief.\nWrite the expanded_prompt as direct descriptive cue phrases, not as a conversation, analysis, explanation, or story.\nUse concrete subject, setting, composition, camera/framing, lighting, texture, color, atmosphere, motion, and quality cues when relevant.\nPrefer dense comma-separated or clause-separated descriptors over chatty sentences.\nDo not mention the user, what they did or did not specify, or your own reasoning.\nDo not contradict explicit user details.\nOnly fill in omitted details with reasonable defaults.\nIf the user leaves something open, choose common, low-risk defaults instead of something bizarre.\nDo not invent unusual anatomy, anthropomorphic behavior, extra limbs, impossible poses, or extra subjects unless the user clearly asked for them.\nIf the subject is an animal, keep it in a normal natural pose unless told otherwise.\nThe negative_prompt should be a concise comma-separated artifact-avoidance list, not a sentence.\nfocus_tags should be short generator-friendly cue tags.\nAssist strength: {}.\nTarget mode: {}.\nTarget media: {}.\nMedia guidance: {}.\nReference: {}.\nOriginal negative prompt: {}.\nOriginal user prompt: {}",
        prompt_assist_strength(mode),
        generation_style_label(style),
        kind.as_str(),
        prompt_assist_media_guidance(kind, style, supports_voice_output),
        reference_summary(reference),
        user_negative_prompt.unwrap_or("None."),
        user_prompt.trim()
    )
}

fn image_scene_graph_prompt(
    user_prompt: &str,
    settings: &GenerationSettings,
    reference: Option<&ReferenceSummary>,
    max_elements: u32,
) -> String {
    format!(
        "You are a compact scene graph planner for a local abstract image generator.\nReturn only JSON matching the schema.\nDo not output primitive shapes directly.\nFirst describe the scene as a reusable layout with a few important motifs and strong spatial anchors.\nChoose only the most important visual elements.\nUse universal motifs from the schema such as figure, creature, pair, swing, tree, water, path, bench, sun, moon, cloud, star_cluster, hill, structure, frame, or accent.\nUse creature for animals or pets.\nUse horizon_y and ground_y to anchor the world.\nUse role to describe layering importance.\nPrefer fewer, stronger elements over many weak ones.\nUse {max_elements} or fewer elements.\nUse compact numbers with one or two decimals when possible.\nLean {}.\nSettings: temperature {:.2}, steps {}, cfg scale {:.1}, resolution {}.\nReference: {}.\nUser prompt: {}",
        adherence_hint(settings.cfg_scale),
        settings.temperature,
        settings.steps,
        settings.cfg_scale,
        settings.resolution.label(),
        reference_summary(reference),
        user_prompt.trim()
    )
}

fn video_scene_graph_prompt(
    user_prompt: &str,
    settings: &GenerationSettings,
    reference: Option<&ReferenceSummary>,
    max_elements: u32,
) -> String {
    format!(
        "You are a compact scene graph planner for a local looping abstract animation generator.\nReturn only JSON matching the schema.\nDo not output primitive shapes directly.\nDescribe a reusable motion scene graph with strong anchors, a small set of important motifs, and motion hints that loop clearly.\nChoose only the most important visual elements.\nUse universal motifs from the schema such as figure, creature, pair, swing, tree, water, path, bench, sun, moon, cloud, star_cluster, hill, structure, frame, or accent.\nUse creature for animals or pets.\nUse horizon_y and ground_y to anchor the world.\nUse role for layer importance and motion for loop feel.\nPrefer fewer, stronger elements over many weak ones.\nUse {max_elements} or fewer elements.\nUse strong contrast and loop-friendly motion.\nUse compact numbers with one or two decimals when possible.\nLean {}.\nSettings: temperature {:.2}, steps {}, cfg scale {:.1}, image resolution {}.\nVideo target: {} for {} seconds at {} fps ({} frames).\nReference: {}.\nUser prompt: {}",
        adherence_hint(settings.cfg_scale),
        settings.temperature,
        settings.steps,
        settings.cfg_scale,
        settings.resolution.label(),
        settings.video_resolution.label(),
        settings.video_duration_seconds,
        settings.video_fps,
        settings.video_frame_count(),
        reference_summary(reference),
        user_prompt.trim()
    )
}

fn audio_prompt(
    user_prompt: &str,
    settings: &GenerationSettings,
    reference: Option<&ReferenceSummary>,
    max_layers: u32,
) -> String {
    format!(
        "You are a compact audio planner for a local synth renderer.\nReturn only JSON matching the schema.\nCreate an instrumental loop or soundscape with up to {max_layers} layers.\nMake it musical, concise, and suitable for immediate playback.\nLean {}.\nSettings: temperature {:.2}, steps {}, cfg scale {:.1}, resolution {}.\nReference: {}.\nUser prompt: {}",
        adherence_hint(settings.cfg_scale),
        settings.temperature,
        settings.steps,
        settings.cfg_scale,
        settings.resolution.label(),
        reference_summary(reference),
        user_prompt.trim()
    )
}

fn adherence_hint(cfg_scale: f32) -> &'static str {
    if cfg_scale >= 13.0 {
        "literal and closely aligned to the nouns and mood in the prompt"
    } else if cfg_scale <= 5.0 {
        "playful and metaphorical while still recognizable"
    } else {
        "balanced between fidelity and stylistic surprise"
    }
}

fn prompt_assist_strength(mode: PromptAssistMode) -> &'static str {
    match mode {
        PromptAssistMode::Off => "off",
        PromptAssistMode::Gentle => {
            "gentle: keep the user's idea intact and only add a few sensible defaults"
        }
        PromptAssistMode::Strong => {
            "strong: make more composition, material, lighting, motion, or mood decisions while preserving explicit user details"
        }
    }
}

fn generation_style_label(style: GenerationStyle) -> &'static str {
    match style {
        GenerationStyle::Expressive => "expressive",
        GenerationStyle::Realism => "realism",
    }
}

fn prompt_assist_media_guidance(
    kind: MediaKind,
    style: GenerationStyle,
    supports_voice_output: bool,
) -> &'static str {
    match (kind, style) {
        (MediaKind::Image, GenerationStyle::Expressive) => {
            "Use concise scene-brief cues for subject, setting, layout, silhouette, palette, lighting, depth, and a few strong visual motifs that an artist-planner can translate into stylized composition."
        }
        (MediaKind::Image, GenerationStyle::Realism) => {
            "Use concrete image-generation cues: subject, setting, camera distance and angle, framing, lighting, materials, textures, color palette, atmosphere, and quality/detail cues."
        }
        (MediaKind::Gif, GenerationStyle::Expressive) => {
            "Use concise animation-brief cues for subject, setting, motion, loop feel, palette, and a few recurring motifs for a stylized animation plan."
        }
        (MediaKind::Gif, GenerationStyle::Realism) => {
            "Use concrete animation-generation cues: subject, setting, motion, camera feel, temporal consistency, loopability, lighting continuity, and artifact-avoidance."
        }
        (MediaKind::Video, GenerationStyle::Expressive) => {
            "Use concise motion-brief cues for subject, setting, motion, rhythm, and camera feel for a true video clip."
        }
        (MediaKind::Video, GenerationStyle::Realism) => {
            "Use concrete video-generation cues: subject, setting, motion, camera feel, temporal consistency, continuity, and clean subject anatomy across frames."
        }
        (MediaKind::Audio, _) if supports_voice_output => {
            "Separate the spoken script from the delivery direction. Preserve quoted dialogue closely, and keep delivery cues in a separate compact direction brief."
        }
        (MediaKind::Audio, _) => {
            "Describe mood, instrumentation, tempo, rhythm, texture, ambience, and how the sound should evolve over a short loop."
        }
    }
}

fn max_scene_elements(settings: &GenerationSettings, kind: MediaKind) -> u32 {
    match kind {
        MediaKind::Image => (3 + settings.steps / 18).clamp(3, 6),
        MediaKind::Gif | MediaKind::Video => (3 + settings.steps / 20).clamp(3, 6),
        MediaKind::Audio => 0,
    }
}

fn image_shape_budget(settings: &GenerationSettings) -> usize {
    (8 + settings.steps / 6).clamp(8, 16) as usize
}

fn video_shape_budget(settings: &GenerationSettings) -> usize {
    (8 + settings.steps / 7).clamp(8, 15) as usize
}

fn planner_temperature(settings: &GenerationSettings) -> f32 {
    (0.15 + settings.temperature.clamp(0.0, 2.0) * 0.1).clamp(0.15, 0.35)
}

fn reference_summary(reference: Option<&ReferenceSummary>) -> String {
    reference
        .map(|value| value.note.clone())
        .unwrap_or_else(|| "No reference asset selected.".to_string())
}

fn prompt_assist_note(
    mode: PromptAssistMode,
    brief: &PromptAssistBrief,
    kind: MediaKind,
    supports_voice_output: bool,
) -> String {
    let assumption_count = brief.assumptions.len();
    let focus_count = brief.focus_tags.len();
    let base = match mode {
        PromptAssistMode::Off => String::new(),
        PromptAssistMode::Gentle => format!(
            "Prompt Assist (gentle) expanded the prompt with {assumption_count} assumption(s) and {focus_count} focus cue(s)."
        ),
        PromptAssistMode::Strong => format!(
            "Prompt Assist (strong) expanded the prompt with {assumption_count} assumption(s) and {focus_count} focus cue(s)."
        ),
    };

    if kind == MediaKind::Audio && supports_voice_output && !base.is_empty() {
        format!(
            "{base} Chatty-art also separated the spoken words from the delivery direction so the whole request is not read aloud verbatim."
        )
    } else {
        base
    }
}

pub fn derive_spoken_text_heuristic(user_prompt: &str) -> String {
    let prompt = user_prompt.trim();
    if prompt.is_empty() {
        return String::new();
    }

    if let Some(quoted) = extract_quoted_spoken_text(prompt) {
        return quoted;
    }

    let lower = prompt.to_ascii_lowercase();
    for marker in [
        "say:",
        "says:",
        "saying:",
        "read:",
        "speak:",
        "spoken:",
        "line:",
        "script:",
        "dialogue:",
        "narration:",
        "narrator says:",
        "voice says:",
    ] {
        if let Some(index) = lower.find(marker) {
            let candidate = clean_spoken_text(&prompt[index + marker.len()..]);
            if !candidate.is_empty() {
                return candidate;
            }
        }
    }

    for marker in [
        " narrator says ",
        " voice says ",
        " says ",
        " saying ",
        " say ",
        " read ",
        " speak ",
        " narrate ",
    ] {
        if let Some(index) = lower.find(marker) {
            let candidate = clean_spoken_text(&prompt[index + marker.len()..]);
            if !candidate.is_empty() {
                return candidate;
            }
        }
    }

    clean_spoken_text(prompt)
}

pub fn derive_speech_direction_heuristic(
    user_prompt: &str,
    spoken_text: Option<&str>,
) -> Option<String> {
    let mut direction = user_prompt
        .trim()
        .replace("\u{201C}", "\"")
        .replace("\u{201D}", "\"");
    if direction.is_empty() {
        return None;
    }

    if let Some(spoken) = spoken_text.map(str::trim).filter(|value| !value.is_empty()) {
        for wrapped in [
            format!("\"{spoken}\""),
            format!("\"{spoken}\""),
            format!("'{spoken}'"),
            spoken.to_string(),
        ] {
            if direction.contains(&wrapped) {
                direction = direction.replacen(&wrapped, " ", 1);
                break;
            }
        }
    }

    let lower = direction.to_ascii_lowercase();
    let mut cleaned = direction.clone();
    for marker in [
        "narrator says",
        "voice says",
        "says",
        "saying",
        "say",
        "read",
        "speak",
        "spoken",
        "script",
        "dialogue",
        "line",
        "narration",
        "text",
    ] {
        if let Some(index) = lower.find(marker) {
            cleaned.replace_range(index..index + marker.len(), " ");
            break;
        }
    }

    let cleaned = collapse_whitespace(
        &cleaned
            .replace(':', " ")
            .replace(';', " ")
            .replace('-', " ")
            .replace(',', " "),
    );
    let cleaned = cleaned
        .trim_matches(|character: char| {
            character.is_whitespace() || ['"', '\'', '.', ',', ';', ':'].contains(&character)
        })
        .to_string();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn extract_quoted_spoken_text(prompt: &str) -> Option<String> {
    let normalized = prompt.replace("\u{201C}", "\"").replace("\u{201D}", "\"");
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for character in normalized.chars() {
        if character == '"' {
            if in_quote {
                let cleaned = clean_spoken_text(&current);
                if !cleaned.is_empty() {
                    segments.push(cleaned);
                }
                current.clear();
            }
            in_quote = !in_quote;
            continue;
        }

        if in_quote {
            current.push(character);
        }
    }

    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" "))
    }
}

fn clean_spoken_text(value: &str) -> String {
    collapse_whitespace(
        value
            .trim()
            .trim_matches(|character: char| {
                character.is_whitespace() || ['"', '\''].contains(&character)
            })
            .trim_start_matches(':')
            .trim(),
    )
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_prompt_assist_brief(mut brief: PromptAssistBrief) -> PromptAssistBrief {
    brief.expanded_prompt = brief.expanded_prompt.trim().to_string();
    brief.negative_prompt = brief.negative_prompt.trim().to_string();
    brief.spoken_text = brief
        .spoken_text
        .take()
        .map(|value| clean_spoken_text(&value))
        .filter(|value| !value.is_empty());
    brief.assumptions = brief
        .assumptions
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .take(6)
        .collect();
    brief.focus_tags = brief
        .focus_tags
        .into_iter()
        .map(|value| value.trim().trim_matches(',').to_string())
        .filter(|value| !value.is_empty())
        .take(10)
        .collect();
    brief
}

fn polish_compiled_prompt(
    raw_prompt: &str,
    focus_tags: &[String],
    style: GenerationStyle,
    kind: MediaKind,
) -> String {
    let mut parts = Vec::new();
    let mut seen = HashSet::new();

    for segment in raw_prompt
        .replace('\n', " ")
        .split(['.', ';', '\n'])
        .map(clean_prompt_segment)
        .filter(|segment| !segment.is_empty())
    {
        for part in segment
            .split(',')
            .map(clean_prompt_segment)
            .filter(|part| !part.is_empty())
        {
            push_prompt_part(&mut parts, &mut seen, part);
        }
    }

    for tag in focus_tags {
        let cleaned = clean_prompt_segment(tag);
        if !cleaned.is_empty() {
            push_prompt_part(&mut parts, &mut seen, cleaned);
        }
    }

    if parts.is_empty() {
        return raw_prompt.trim().to_string();
    }

    let mut joined = parts.join(", ");
    if matches!(style, GenerationStyle::Realism)
        && matches!(kind, MediaKind::Image | MediaKind::Gif | MediaKind::Video)
    {
        joined = joined.replace("  ", " ");
    }
    joined
}

fn push_prompt_part(parts: &mut Vec<String>, seen: &mut HashSet<String>, value: String) {
    let key = value.to_ascii_lowercase();
    if seen.insert(key) {
        parts.push(value);
    }
}

fn clean_prompt_segment(segment: &str) -> String {
    let mut cleaned = segment.trim().to_string();
    if cleaned.is_empty() {
        return String::new();
    }

    let lower = cleaned.to_ascii_lowercase();
    for prefix in [
        "the user said ",
        "the user wants ",
        "the user asked for ",
        "the user requested ",
        "the user did not specify ",
        "user prompt: ",
        "original prompt: ",
        "the scene shows ",
        "this image shows ",
        "this scene shows ",
        "the image shows ",
        "the video shows ",
        "the prompt should show ",
        "depicting ",
        "depict ",
        "showing ",
    ] {
        if lower.starts_with(prefix) {
            cleaned = cleaned[prefix.len()..].trim().to_string();
            break;
        }
    }

    cleaned = cleaned
        .trim_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | ' '))
        .to_string();

    if cleaned.eq_ignore_ascii_case("none") {
        return String::new();
    }

    cleaned
}

fn optional_text(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() { None } else { Some(value) }
}

fn merge_negative_prompts(left: Option<&str>, right: Option<&str>) -> Option<String> {
    let mut parts = Vec::new();
    let mut seen = HashSet::new();

    push_negative_prompt_parts(&mut parts, &mut seen, left);
    push_negative_prompt_parts(&mut parts, &mut seen, right);

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn push_negative_prompt_parts(
    parts: &mut Vec<String>,
    seen: &mut HashSet<String>,
    value: Option<&str>,
) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };

    for part in value
        .split([',', ';', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let cleaned = part
            .trim_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | ' '))
            .trim();
        if cleaned.is_empty() {
            continue;
        }
        let normalized = cleaned.to_ascii_lowercase();
        if seen.insert(normalized) {
            parts.push(cleaned.to_string());
        }
    }
}

fn normalize_scene_graph(mut graph: SceneGraphPlan, max_elements: usize) -> SceneGraphPlan {
    graph.background_top = normalize_hex(&graph.background_top, "#253552");
    graph.background_bottom = normalize_hex(&graph.background_bottom, "#0C1322");
    graph.accent = normalize_hex(&graph.accent, "#F5C657");
    graph.horizon_y = graph.horizon_y.clamp(0.24, 0.72);
    graph.ground_y = graph.ground_y.clamp(graph.horizon_y + 0.12, 0.94);
    graph.focus_x = graph.focus_x.clamp(0.1, 0.9);
    graph.focus_y = graph.focus_y.clamp(0.15, 0.85);
    graph.elements = graph
        .elements
        .into_iter()
        .take(max_elements)
        .map(normalize_scene_element)
        .collect();
    graph
}

fn normalize_motion_scene_graph(
    mut graph: MotionSceneGraphPlan,
    max_elements: usize,
) -> MotionSceneGraphPlan {
    graph.scene = normalize_scene_graph(graph.scene, max_elements);
    graph.fps = graph.fps.clamp(8, 18);
    graph.frames = graph.frames.clamp(12, 32);
    graph
}

fn repair_scene_graph(mut graph: SceneGraphPlan, prompt: &str) -> SceneGraphPlan {
    graph.horizon_y = graph.horizon_y.clamp(0.24, 0.72);
    graph.ground_y = graph.ground_y.max(graph.horizon_y + 0.12).min(0.94);
    graph.focus_x = graph.focus_x.clamp(0.1, 0.9);
    graph.focus_y = graph.focus_y.clamp(0.15, 0.85);
    let hints = infer_scene_intent(prompt);
    let mut seen_positions = Vec::new();
    let element_count = graph.elements.len().max(1);
    let focus_x = graph.focus_x;
    let focus_y = graph.focus_y;
    let horizon_y = graph.horizon_y;
    let ground_y = graph.ground_y;

    for (index, element) in graph.elements.iter_mut().enumerate() {
        repair_scene_element(
            element,
            focus_x,
            focus_y,
            horizon_y,
            ground_y,
            &hints,
            &mut seen_positions,
            index,
            element_count,
        );
    }

    ensure_required_scene_elements(&mut graph, &hints);
    graph.elements.sort_by(|left, right| {
        right
            .emphasis
            .partial_cmp(&left.emphasis)
            .unwrap_or(Ordering::Equal)
    });
    graph
}

fn repair_motion_scene_graph(
    mut graph: MotionSceneGraphPlan,
    prompt: &str,
) -> MotionSceneGraphPlan {
    graph.scene = repair_scene_graph(graph.scene, prompt);
    for element in &mut graph.scene.elements {
        if element.motion == MotionCue::Still {
            element.motion = default_motion_for_motif(element.motif);
        }
    }
    graph
}

fn infer_scene_intent(prompt: &str) -> SceneIntentHints {
    let lower = prompt.to_ascii_lowercase();
    let has = |terms: &[&str]| terms.iter().any(|term| lower.contains(term));

    SceneIntentHints {
        wants_creature: has(&[
            " dog", "dog ", "cat", "horse", "fox", "wolf", "bear", "lion", "tiger", "rabbit",
            "bunny", "puppy", "kitten", "animal", "pet", "bird", "deer",
        ]),
        wants_person: has(&[
            "person", "man", "woman", "child", "kid", "mother", "father", "human", "girl", "boy",
            "couple", "family",
        ]),
        wants_pair: has(&[
            "pair", "couple", "together", "two ", "mother", "father", "family",
        ]),
        wants_swing: has(&["swing"]),
        wants_tree: has(&["tree", "forest", "woods", "park"]),
        wants_water: has(&["water", "river", "lake", "pond", "ocean", "sea", "beach"]),
        wants_bench: has(&["bench", "park"]),
        wants_path: has(&["path", "road", "trail", "lane"]),
        wants_sun: has(&["sun", "sunlit", "sunny", "sunrise", "sunset", "golden hour"]),
        wants_moon: has(&["moon", "moonlit", "night"]),
        wants_cloud: has(&["cloud", "sky"]),
        wants_hill: has(&["hill", "mountain", "meadow", "field"]),
        wants_field: has(&["field", "meadow", "grass", "pasture"]),
    }
}

fn repair_scene_element(
    element: &mut SceneElement,
    focus_x: f32,
    focus_y: f32,
    horizon_y: f32,
    ground_y: f32,
    hints: &SceneIntentHints,
    seen_positions: &mut Vec<(f32, f32)>,
    index: usize,
    element_count: usize,
) {
    element.motif = repair_scene_motif(element.motif, element.role, hints, index);
    element.role = repair_role_for_motif(element.motif, element.role);
    element.motion = normalize_motion_for_motif(element.motion, element.motif);

    let (anchor_x, anchor_y) = motif_anchor(
        element.motif,
        focus_x,
        focus_y,
        horizon_y,
        ground_y,
        index,
        element_count,
    );
    let duplicate = seen_positions
        .iter()
        .any(|(x, y)| ((element.x - *x).powi(2) + (element.y - *y).powi(2)).sqrt() < 0.08);
    let wrong_zone = motif_out_of_zone(element.motif, element.x, element.y, horizon_y, ground_y);
    let anchor_mix = if duplicate || wrong_zone { 0.78 } else { 0.34 };
    element.x = lerp(element.x, anchor_x, anchor_mix).clamp(0.08, 0.92);
    element.y = lerp(element.y, anchor_y, anchor_mix).clamp(0.08, 0.92);

    if hints.wants_creature && element.motif == SceneMotif::Creature {
        element.scale = element.scale.max(0.5);
        element.emphasis = element.emphasis.max(0.78);
    } else if matches!(element.motif, SceneMotif::Sun | SceneMotif::Moon) {
        element.scale = element.scale.max(0.36);
    }

    seen_positions.push((element.x, element.y));
}

fn repair_scene_motif(
    motif: SceneMotif,
    role: SceneRole,
    hints: &SceneIntentHints,
    index: usize,
) -> SceneMotif {
    if hints.wants_creature {
        return match motif {
            SceneMotif::Figure | SceneMotif::Pair if !hints.wants_person => SceneMotif::Creature,
            SceneMotif::Swing if !hints.wants_swing => {
                if hints.wants_tree {
                    SceneMotif::Tree
                } else if hints.wants_field {
                    SceneMotif::Hill
                } else {
                    SceneMotif::Creature
                }
            }
            other => other,
        };
    }

    match motif {
        SceneMotif::Pair if !hints.wants_pair => {
            if hints.wants_person {
                SceneMotif::Figure
            } else if hints.wants_tree {
                SceneMotif::Tree
            } else if index.is_multiple_of(2) {
                SceneMotif::Accent
            } else {
                SceneMotif::Figure
            }
        }
        SceneMotif::Swing if !hints.wants_swing => {
            if hints.wants_tree {
                SceneMotif::Tree
            } else if hints.wants_field {
                SceneMotif::Hill
            } else if role == SceneRole::Celestial {
                SceneMotif::Cloud
            } else {
                SceneMotif::Accent
            }
        }
        other => other,
    }
}

fn repair_role_for_motif(motif: SceneMotif, role: SceneRole) -> SceneRole {
    match motif {
        SceneMotif::Sun | SceneMotif::Moon | SceneMotif::Cloud | SceneMotif::StarCluster => {
            SceneRole::Celestial
        }
        SceneMotif::Hill => SceneRole::Background,
        SceneMotif::Water => SceneRole::Reflection,
        SceneMotif::Path | SceneMotif::Bench | SceneMotif::Seat => SceneRole::Ground,
        SceneMotif::Figure
        | SceneMotif::Creature
        | SceneMotif::Pair
        | SceneMotif::Swing
        | SceneMotif::Tree
        | SceneMotif::Structure
        | SceneMotif::Frame => SceneRole::Subject,
        SceneMotif::Accent => role,
    }
}

fn motif_anchor(
    motif: SceneMotif,
    focus_x: f32,
    focus_y: f32,
    horizon_y: f32,
    ground_y: f32,
    index: usize,
    element_count: usize,
) -> (f32, f32) {
    let phase = if element_count <= 1 {
        0.0
    } else {
        index as f32 / (element_count - 1) as f32
    };
    match motif {
        SceneMotif::Figure | SceneMotif::Creature | SceneMotif::Swing | SceneMotif::Frame => (
            (focus_x + (phase - 0.5) * 0.18).clamp(0.2, 0.8),
            clamp_range(
                lerp(
                    clamp_range(ground_y - 0.18, horizon_y + 0.12, 0.82),
                    focus_y,
                    0.4,
                ),
                horizon_y + 0.12,
                0.82,
            ),
        ),
        SceneMotif::Pair => (
            focus_x.clamp(0.28, 0.72),
            clamp_range(
                lerp(
                    clamp_range(ground_y - 0.18, horizon_y + 0.12, 0.82),
                    focus_y,
                    0.35,
                ),
                horizon_y + 0.12,
                0.82,
            ),
        ),
        SceneMotif::Tree => (
            (if index.is_multiple_of(2) {
                focus_x - 0.28
            } else {
                focus_x + 0.28
            })
            .clamp(0.12, 0.88),
            clamp_range(ground_y - 0.14, horizon_y + 0.12, 0.86),
        ),
        SceneMotif::Water => (
            focus_x.clamp(0.18, 0.82),
            clamp_range(ground_y - 0.04, horizon_y + 0.18, 0.9),
        ),
        SceneMotif::Path => (
            focus_x.clamp(0.18, 0.82),
            clamp_range(ground_y - 0.02, horizon_y + 0.2, 0.92),
        ),
        SceneMotif::Bench | SceneMotif::Seat => (
            (focus_x + (phase - 0.5) * 0.28).clamp(0.16, 0.84),
            clamp_range(ground_y - 0.06, horizon_y + 0.16, 0.9),
        ),
        SceneMotif::Sun | SceneMotif::Moon => (
            (if index.is_multiple_of(2) {
                0.78_f32
            } else {
                0.22_f32
            })
            .clamp(0.14, 0.86),
            (horizon_y - 0.22).clamp(0.12, 0.3),
        ),
        SceneMotif::Cloud => (
            (0.25 + phase * 0.5).clamp(0.16, 0.84),
            (horizon_y - 0.18).clamp(0.12, 0.34),
        ),
        SceneMotif::StarCluster => (
            (0.18 + phase * 0.64).clamp(0.12, 0.88),
            (horizon_y - 0.24).clamp(0.08, 0.28),
        ),
        SceneMotif::Hill => (
            (0.22 + phase * 0.56).clamp(0.12, 0.88),
            clamp_range(horizon_y + 0.08, horizon_y + 0.04, ground_y - 0.08),
        ),
        SceneMotif::Structure => (
            (focus_x + (phase - 0.5) * 0.18).clamp(0.18, 0.82),
            clamp_range(ground_y - 0.2, horizon_y + 0.12, 0.84),
        ),
        SceneMotif::Accent => (
            (focus_x + (phase - 0.5) * 0.34).clamp(0.1, 0.9),
            (horizon_y + 0.12 + phase * 0.26).clamp(0.14, 0.88),
        ),
    }
}

fn motif_out_of_zone(motif: SceneMotif, x: f32, y: f32, horizon_y: f32, ground_y: f32) -> bool {
    let _ = x;
    match motif {
        SceneMotif::Sun | SceneMotif::Moon | SceneMotif::Cloud | SceneMotif::StarCluster => {
            y > horizon_y - 0.02
        }
        SceneMotif::Water | SceneMotif::Path | SceneMotif::Bench | SceneMotif::Seat => {
            y < horizon_y + 0.05
        }
        SceneMotif::Figure
        | SceneMotif::Creature
        | SceneMotif::Pair
        | SceneMotif::Swing
        | SceneMotif::Tree
        | SceneMotif::Structure
        | SceneMotif::Frame => y < horizon_y + 0.02 || y > ground_y + 0.02,
        SceneMotif::Hill => y < horizon_y,
        SceneMotif::Accent => false,
    }
}

fn ensure_required_scene_elements(graph: &mut SceneGraphPlan, hints: &SceneIntentHints) {
    if hints.wants_creature
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Creature)
    {
        graph.elements.insert(
            0,
            SceneElement {
                motif: SceneMotif::Creature,
                role: SceneRole::Subject,
                x: graph.focus_x,
                y: clamp_range(graph.ground_y - 0.18, graph.horizon_y + 0.12, 0.82),
                scale: 0.72,
                emphasis: 0.92,
                rotation: 0.0,
                motion: MotionCue::Bob,
            },
        );
    }

    if hints.wants_tree
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Tree)
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Tree,
            role: SceneRole::Subject,
            x: (graph.focus_x - 0.28).clamp(0.12, 0.88),
            y: clamp_range(graph.ground_y - 0.14, graph.horizon_y + 0.12, 0.86),
            scale: 0.58,
            emphasis: 0.56,
            rotation: 0.0,
            motion: MotionCue::Drift,
        });
    }

    if hints.wants_field
        && !graph.elements.iter().any(|element| {
            matches!(
                element.motif,
                SceneMotif::Hill | SceneMotif::Path | SceneMotif::Accent
            )
        })
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Hill,
            role: SceneRole::Background,
            x: 0.72,
            y: clamp_range(
                graph.horizon_y + 0.08,
                graph.horizon_y + 0.04,
                graph.ground_y - 0.08,
            ),
            scale: 0.46,
            emphasis: 0.42,
            rotation: 0.0,
            motion: MotionCue::Drift,
        });
    }

    if hints.wants_hill
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Hill)
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Hill,
            role: SceneRole::Background,
            x: 0.74,
            y: clamp_range(
                graph.horizon_y + 0.08,
                graph.horizon_y + 0.04,
                graph.ground_y - 0.08,
            ),
            scale: 0.42,
            emphasis: 0.4,
            rotation: 0.0,
            motion: MotionCue::Drift,
        });
    }

    if hints.wants_sun
        && !graph
            .elements
            .iter()
            .any(|element| matches!(element.motif, SceneMotif::Sun | SceneMotif::Moon))
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Sun,
            role: SceneRole::Celestial,
            x: 0.78,
            y: (graph.horizon_y - 0.22).clamp(0.12, 0.3),
            scale: 0.42,
            emphasis: 0.48,
            rotation: 0.0,
            motion: MotionCue::Pulse,
        });
    }

    if hints.wants_moon
        && !graph
            .elements
            .iter()
            .any(|element| matches!(element.motif, SceneMotif::Moon | SceneMotif::Sun))
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Moon,
            role: SceneRole::Celestial,
            x: 0.78,
            y: (graph.horizon_y - 0.22).clamp(0.12, 0.3),
            scale: 0.38,
            emphasis: 0.44,
            rotation: 0.0,
            motion: MotionCue::Pulse,
        });
    }

    if hints.wants_cloud
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Cloud)
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Cloud,
            role: SceneRole::Celestial,
            x: 0.32,
            y: (graph.horizon_y - 0.16).clamp(0.14, 0.34),
            scale: 0.34,
            emphasis: 0.32,
            rotation: 0.0,
            motion: MotionCue::Drift,
        });
    }

    if hints.wants_water
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Water)
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Water,
            role: SceneRole::Reflection,
            x: graph.focus_x,
            y: clamp_range(graph.ground_y - 0.04, graph.horizon_y + 0.18, 0.9),
            scale: 0.46,
            emphasis: 0.4,
            rotation: 0.0,
            motion: MotionCue::Ripple,
        });
    }

    if hints.wants_path
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Path)
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Path,
            role: SceneRole::Ground,
            x: graph.focus_x,
            y: clamp_range(graph.ground_y - 0.02, graph.horizon_y + 0.2, 0.92),
            scale: 0.38,
            emphasis: 0.34,
            rotation: 0.0,
            motion: MotionCue::Ripple,
        });
    }

    if hints.wants_bench
        && !graph
            .elements
            .iter()
            .any(|element| element.motif == SceneMotif::Bench)
    {
        graph.elements.push(SceneElement {
            motif: SceneMotif::Bench,
            role: SceneRole::Ground,
            x: (graph.focus_x + 0.24).clamp(0.16, 0.84),
            y: clamp_range(graph.ground_y - 0.06, graph.horizon_y + 0.16, 0.9),
            scale: 0.3,
            emphasis: 0.3,
            rotation: 0.0,
            motion: MotionCue::Still,
        });
    }
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from * (1.0 - amount) + to * amount
}

fn clamp_range(value: f32, min: f32, max: f32) -> f32 {
    if min <= max {
        value.clamp(min, max)
    } else {
        value.clamp(max, min)
    }
}

fn normalize_scene_element(mut element: SceneElement) -> SceneElement {
    element.role = normalize_element_role(element.role, element.motif);
    element.x = element.x.clamp(0.08, 0.92);
    element.y = element.y.clamp(0.08, 0.92);
    element.scale = element.scale.clamp(0.2, 1.0);
    element.emphasis = element.emphasis.clamp(0.2, 1.0);
    element.rotation = element.rotation.clamp(-180.0, 180.0);
    element.motion = normalize_motion_for_motif(element.motion, element.motif);
    element
}

fn normalize_element_role(role: SceneRole, motif: SceneMotif) -> SceneRole {
    match motif {
        SceneMotif::Sun | SceneMotif::Moon | SceneMotif::Cloud | SceneMotif::StarCluster => {
            SceneRole::Celestial
        }
        SceneMotif::Water => SceneRole::Reflection,
        SceneMotif::Path | SceneMotif::Bench | SceneMotif::Seat => SceneRole::Ground,
        SceneMotif::Hill => SceneRole::Background,
        SceneMotif::Figure
        | SceneMotif::Creature
        | SceneMotif::Pair
        | SceneMotif::Swing
        | SceneMotif::Tree
        | SceneMotif::Structure
        | SceneMotif::Frame => SceneRole::Subject,
        SceneMotif::Accent => role,
    }
}

fn normalize_motion_for_motif(motion: MotionCue, motif: SceneMotif) -> MotionCue {
    match motion {
        MotionCue::Still => default_motion_for_motif(motif),
        other => other,
    }
}

fn default_motion_for_motif(motif: SceneMotif) -> MotionCue {
    match motif {
        SceneMotif::Figure | SceneMotif::Creature | SceneMotif::Pair => MotionCue::Bob,
        SceneMotif::Swing => MotionCue::Sway,
        SceneMotif::Water | SceneMotif::Path => MotionCue::Ripple,
        SceneMotif::Sun | SceneMotif::Moon => MotionCue::Pulse,
        SceneMotif::Cloud | SceneMotif::Hill => MotionCue::Drift,
        SceneMotif::StarCluster | SceneMotif::Accent => MotionCue::Glimmer,
        SceneMotif::Tree => MotionCue::Drift,
        SceneMotif::Seat | SceneMotif::Bench | SceneMotif::Structure | SceneMotif::Frame => {
            MotionCue::Still
        }
    }
}

fn compose_image_plan(
    graph: &SceneGraphPlan,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
    shape_budget: usize,
) -> ImagePlan {
    let shapes = compose_scene_shapes(graph, settings, seed, shape_budget);
    let plan = ImagePlan {
        background_top: graph.background_top.clone(),
        background_bottom: graph.background_bottom.clone(),
        accent: graph.accent.clone(),
        shapes: shapes.into_iter().map(|shape| shape.base).collect(),
    };
    normalize_image_plan(plan, reference, shape_budget.max(6))
}

fn compose_video_plan(
    graph: &MotionSceneGraphPlan,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
    shape_budget: usize,
) -> VideoPlan {
    let shapes = compose_scene_shapes(&graph.scene, settings, seed, shape_budget)
        .into_iter()
        .map(shape_to_motion_shape)
        .collect::<Vec<_>>();
    let plan = VideoPlan {
        background_top: graph.scene.background_top.clone(),
        background_bottom: graph.scene.background_bottom.clone(),
        accent: graph.scene.accent.clone(),
        fps: settings.video_fps.clamp(8, 24) as u16,
        frames: settings.video_frame_count().clamp(16, 480) as u16,
        shapes,
    };
    normalize_video_plan(plan, reference, shape_budget.max(6))
}

fn compose_scene_shapes(
    graph: &SceneGraphPlan,
    settings: &GenerationSettings,
    seed: u32,
    shape_budget: usize,
) -> Vec<PlannedShape> {
    let mut rng = ChaCha8Rng::seed_from_u64(prompt_hash((
        graph.focus_x.to_bits(),
        graph.focus_y.to_bits(),
        u64::from(seed),
        settings.steps,
    )));
    let palette = scene_palette(graph);
    let mut shapes = base_scene_scaffold(graph, &palette);

    let mut elements = graph.elements.clone();
    elements.sort_by(|left, right| {
        right
            .emphasis
            .partial_cmp(&left.emphasis)
            .unwrap_or(Ordering::Equal)
    });

    let bundles = elements
        .iter()
        .enumerate()
        .map(|(index, element)| expand_scene_element(element, graph, &palette, &mut rng, index))
        .collect::<Vec<_>>();

    let mut depth = 0usize;
    while shapes.len() < shape_budget {
        let mut added_any = false;
        for bundle in &bundles {
            if let Some(shape) = bundle.get(depth) {
                shapes.push(shape.clone());
                added_any = true;
                if shapes.len() >= shape_budget {
                    break;
                }
            }
        }
        if !added_any {
            break;
        }
        depth += 1;
    }

    if shapes.len() < 6 {
        let fallback = fallback_image_plan(
            prompt_hash((graph.focus_x.to_bits(), graph.focus_y.to_bits(), seed)),
            settings,
            seed,
            None,
        );
        return fallback
            .shapes
            .into_iter()
            .map(|base| PlannedShape {
                motion: default_motion_for_role(base.role),
                base,
            })
            .collect();
    }

    shapes.truncate(shape_budget);
    shapes
}

fn base_scene_scaffold(graph: &SceneGraphPlan, palette: &ScenePalette) -> Vec<PlannedShape> {
    vec![
        planned_shape(
            ShapeKind::Rectangle,
            SceneRole::Horizon,
            0.5,
            graph.horizon_y,
            0.22,
            8.5,
            0.0,
            &palette.light,
            &palette.top,
            0.24,
            MotionCue::Still,
        ),
        planned_shape(
            ShapeKind::Rectangle,
            SceneRole::Ground,
            0.5,
            graph.ground_y,
            0.24,
            10.0,
            0.0,
            &palette.earth,
            &palette.bottom,
            0.28,
            MotionCue::Ripple,
        ),
    ]
}

fn scene_palette(graph: &SceneGraphPlan) -> ScenePalette {
    ScenePalette {
        top: graph.background_top.clone(),
        bottom: graph.background_bottom.clone(),
        accent: graph.accent.clone(),
        shadow: blend_hex_colors_local(&graph.background_bottom, "#10151D", 0.46),
        light: blend_hex_colors_local(&graph.background_top, "#F8F3DF", 0.34),
        foliage: blend_hex_colors_local(&graph.accent, "#5E9B58", 0.52),
        earth: blend_hex_colors_local(&graph.background_bottom, "#8B7255", 0.28),
        water: blend_hex_colors_local(&graph.background_top, "#3E84B2", 0.42),
        wood: blend_hex_colors_local(&graph.accent, "#8F6748", 0.36),
        cloud: blend_hex_colors_local(&graph.background_top, "#F4F5F3", 0.52),
    }
}

fn expand_scene_element(
    element: &SceneElement,
    graph: &SceneGraphPlan,
    palette: &ScenePalette,
    rng: &mut ChaCha8Rng,
    index: usize,
) -> Vec<PlannedShape> {
    let unit = (0.05 + element.scale * 0.12 + element.emphasis * 0.05).clamp(0.07, 0.22);
    let x = element.x;
    let y = element
        .y
        .clamp(graph.horizon_y - 0.14, (graph.ground_y + 0.08).min(0.9));
    let rotation = element.rotation;

    match element.motif {
        SceneMotif::Figure => vec![
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Subject,
                x,
                y - unit * 1.35,
                unit * 0.52,
                1.0,
                rotation,
                &palette.light,
                &palette.accent,
                0.82,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y - unit * 0.2,
                unit * 0.82,
                0.62,
                rotation,
                &palette.shadow,
                &palette.accent,
                0.78,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x,
                y + unit * 0.95,
                unit * 1.2,
                0.4,
                90.0 + rotation * 0.2,
                &palette.shadow,
                &palette.light,
                0.66,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Detail,
                x + unit * 0.46,
                y + unit * 0.08,
                unit * 0.88,
                0.28,
                20.0 + rotation * 0.25,
                &palette.accent,
                &palette.light,
                0.52,
                element.motion,
            ),
        ],
        SceneMotif::Creature => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y - unit * 0.08,
                unit * 0.82,
                1.42,
                rotation * 0.18,
                &palette.shadow,
                &palette.accent,
                0.82,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Subject,
                x + unit * 0.88,
                y - unit * 0.46,
                unit * 0.38,
                1.0,
                rotation * 0.12,
                &palette.light,
                &palette.accent,
                0.78,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x - unit * 0.28,
                y + unit * 0.96,
                unit * 0.84,
                0.18,
                90.0,
                &palette.shadow,
                &palette.light,
                0.58,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x + unit * 0.38,
                y + unit * 0.96,
                unit * 0.82,
                0.18,
                90.0,
                &palette.shadow,
                &palette.light,
                0.58,
                element.motion,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Detail,
                x - unit * 0.9,
                y - unit * 0.14,
                unit * 0.62,
                0.16,
                24.0 + rotation * 0.2,
                &palette.accent,
                &palette.light,
                0.46,
                MotionCue::Drift,
            ),
        ],
        SceneMotif::Pair => {
            let offset = unit * 0.72;
            let left = SceneElement {
                motif: SceneMotif::Figure,
                x: (x - offset).clamp(0.08, 0.92),
                scale: (element.scale * 0.78).clamp(0.2, 1.0),
                emphasis: element.emphasis,
                ..element.clone()
            };
            let right = SceneElement {
                motif: SceneMotif::Figure,
                x: (x + offset).clamp(0.08, 0.92),
                scale: (element.scale * 0.9).clamp(0.2, 1.0),
                emphasis: element.emphasis,
                ..element.clone()
            };
            let mut shapes = expand_scene_element(&left, graph, palette, rng, index);
            shapes.extend(expand_scene_element(&right, graph, palette, rng, index + 1));
            shapes.push(planned_shape(
                ShapeKind::Line,
                SceneRole::Detail,
                x,
                y - unit * 0.45,
                unit * 0.92,
                0.22,
                -12.0,
                &palette.accent,
                &palette.light,
                0.34,
                element.motion,
            ));
            shapes
        }
        SceneMotif::Seat => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Ground,
                x,
                y + unit * 0.55,
                unit * 0.42,
                1.8,
                rotation,
                &palette.wood,
                &palette.shadow,
                0.72,
                MotionCue::Still,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Detail,
                x,
                y + unit * 0.2,
                unit * 0.95,
                0.24,
                90.0,
                &palette.shadow,
                &palette.wood,
                0.44,
                MotionCue::Still,
            ),
        ],
        SceneMotif::Swing => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y - unit * 2.4,
                unit * 0.42,
                3.8,
                rotation * 0.08,
                &palette.wood,
                &palette.shadow,
                0.58,
                MotionCue::Still,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x - unit * 0.62,
                y - unit * 0.82,
                unit * 1.56,
                0.18,
                92.0,
                &palette.light,
                &palette.wood,
                0.6,
                MotionCue::Sway,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x + unit * 0.62,
                y - unit * 0.82,
                unit * 1.56,
                0.18,
                88.0,
                &palette.light,
                &palette.wood,
                0.6,
                MotionCue::Sway,
            ),
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y + unit * 0.52,
                unit * 0.34,
                1.7,
                rotation * 0.12,
                &palette.wood,
                &palette.accent,
                0.82,
                MotionCue::Sway,
            ),
        ],
        SceneMotif::Tree => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y + unit * 0.72,
                unit * 0.8,
                0.38,
                rotation * 0.1,
                &palette.wood,
                &palette.shadow,
                0.72,
                MotionCue::Drift,
            ),
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Subject,
                x - unit * 0.28,
                y - unit * 0.86,
                unit * 0.92,
                1.0,
                rotation,
                &palette.foliage,
                &palette.accent,
                0.72,
                MotionCue::Drift,
            ),
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Detail,
                x + unit * 0.46,
                y - unit * 0.62,
                unit * 0.68,
                1.0,
                rotation,
                &palette.foliage,
                &palette.light,
                0.56,
                MotionCue::Drift,
            ),
        ],
        SceneMotif::Water => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Reflection,
                x,
                y + unit * 0.28,
                unit * 0.74,
                4.6,
                0.0,
                &palette.water,
                &palette.light,
                0.34,
                MotionCue::Ripple,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Reflection,
                x,
                y + unit * 0.18,
                unit * 1.14,
                0.18,
                0.0,
                &palette.light,
                &palette.water,
                0.26,
                MotionCue::Ripple,
            ),
        ],
        SceneMotif::Path => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Ground,
                x,
                y + unit * 0.6,
                unit * 0.68,
                2.8,
                rotation * 0.12,
                &palette.earth,
                &palette.light,
                0.34,
                MotionCue::Ripple,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Detail,
                x,
                y + unit * 0.28,
                unit * 1.16,
                0.16,
                90.0 + rotation * 0.1,
                &palette.light,
                &palette.earth,
                0.22,
                MotionCue::Ripple,
            ),
        ],
        SceneMotif::Bench => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Ground,
                x,
                y + unit * 0.5,
                unit * 0.3,
                2.1,
                rotation * 0.1,
                &palette.wood,
                &palette.shadow,
                0.74,
                MotionCue::Still,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Ground,
                x,
                y + unit * 0.08,
                unit * 0.92,
                0.16,
                90.0,
                &palette.shadow,
                &palette.wood,
                0.42,
                MotionCue::Still,
            ),
        ],
        SceneMotif::Sun => vec![planned_shape(
            ShapeKind::Circle,
            SceneRole::Celestial,
            x,
            y.min(graph.horizon_y - 0.08),
            unit * 0.86,
            1.0,
            rotation,
            &palette.accent,
            &palette.light,
            0.84,
            MotionCue::Pulse,
        )],
        SceneMotif::Moon => vec![planned_shape(
            ShapeKind::Ring,
            SceneRole::Celestial,
            x,
            y.min(graph.horizon_y - 0.08),
            unit * 0.76,
            0.54,
            rotation,
            &palette.light,
            &palette.accent,
            0.72,
            MotionCue::Pulse,
        )],
        SceneMotif::Cloud => vec![
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Celestial,
                x - unit * 0.42,
                y.min(graph.horizon_y - 0.04),
                unit * 0.56,
                1.0,
                rotation,
                &palette.cloud,
                &palette.top,
                0.48,
                MotionCue::Drift,
            ),
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Celestial,
                x,
                y.min(graph.horizon_y - 0.04),
                unit * 0.68,
                1.0,
                rotation,
                &palette.cloud,
                &palette.light,
                0.54,
                MotionCue::Drift,
            ),
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Celestial,
                x + unit * 0.42,
                y.min(graph.horizon_y - 0.04),
                unit * 0.5,
                1.0,
                rotation,
                &palette.cloud,
                &palette.top,
                0.46,
                MotionCue::Drift,
            ),
        ],
        SceneMotif::StarCluster => {
            let count = 3 + (index % 2);
            (0..count)
                .map(|star_index| {
                    let spread = unit * (0.35 + star_index as f32 * 0.18);
                    planned_shape(
                        if star_index % 2 == 0 {
                            ShapeKind::Circle
                        } else {
                            ShapeKind::Ring
                        },
                        SceneRole::Celestial,
                        (x + rng.random_range(-spread..spread)).clamp(0.08, 0.92),
                        (y + rng.random_range(-spread..spread)).clamp(0.06, graph.horizon_y - 0.04),
                        unit * rng.random_range(0.16..0.28),
                        rng.random_range(0.4..1.0),
                        rng.random_range(-35.0..35.0),
                        &palette.light,
                        &palette.accent,
                        rng.random_range(0.38..0.72),
                        MotionCue::Glimmer,
                    )
                })
                .collect()
        }
        SceneMotif::Hill => vec![
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Background,
                x,
                y.max(graph.horizon_y + 0.04),
                unit * 1.5,
                2.0,
                rotation,
                &palette.bottom,
                &palette.earth,
                0.34,
                MotionCue::Drift,
            ),
            planned_shape(
                ShapeKind::Circle,
                SceneRole::Background,
                x + unit * 0.92,
                y.max(graph.horizon_y + 0.06),
                unit * 1.08,
                1.6,
                rotation,
                &palette.earth,
                &palette.shadow,
                0.24,
                MotionCue::Drift,
            ),
        ],
        SceneMotif::Structure => vec![
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y + unit * 0.05,
                unit * 1.12,
                0.9,
                rotation * 0.14,
                &palette.shadow,
                &palette.light,
                0.56,
                MotionCue::Still,
            ),
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Detail,
                x,
                y - unit * 1.08,
                unit * 0.34,
                2.0,
                rotation * 0.08,
                &palette.light,
                &palette.accent,
                0.42,
                MotionCue::Still,
            ),
        ],
        SceneMotif::Frame => vec![
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x - unit * 0.9,
                y,
                unit * 1.58,
                0.18,
                90.0,
                &palette.shadow,
                &palette.wood,
                0.48,
                MotionCue::Still,
            ),
            planned_shape(
                ShapeKind::Line,
                SceneRole::Subject,
                x + unit * 0.9,
                y,
                unit * 1.58,
                0.18,
                90.0,
                &palette.shadow,
                &palette.wood,
                0.48,
                MotionCue::Still,
            ),
            planned_shape(
                ShapeKind::Rectangle,
                SceneRole::Subject,
                x,
                y - unit * 1.34,
                unit * 0.26,
                3.2,
                0.0,
                &palette.wood,
                &palette.shadow,
                0.42,
                MotionCue::Still,
            ),
        ],
        SceneMotif::Accent => vec![planned_shape(
            if index.is_multiple_of(2) {
                ShapeKind::Ring
            } else {
                ShapeKind::Circle
            },
            element.role,
            x,
            y,
            unit * 0.5,
            rng.random_range(0.5..1.3),
            rotation,
            &palette.accent,
            &palette.light,
            0.32,
            MotionCue::Glimmer,
        )],
    }
}

fn planned_shape(
    kind: ShapeKind,
    role: SceneRole,
    x: f32,
    y: f32,
    size: f32,
    aspect: f32,
    rotation: f32,
    color: &str,
    secondary_color: &str,
    opacity: f32,
    motion: MotionCue,
) -> PlannedShape {
    PlannedShape {
        motion,
        base: ShapePlan {
            kind,
            role,
            x: x.clamp(0.05, 0.95),
            y: y.clamp(0.05, 0.95),
            size: size.clamp(0.05, 0.6),
            aspect: aspect.clamp(0.16, 12.0),
            rotation,
            color: normalize_hex(color, "#5AB9EA"),
            secondary_color: normalize_hex(secondary_color, "#FCE38A"),
            opacity: opacity.clamp(0.15, 1.0),
        },
    }
}

fn shape_to_motion_shape(shape: PlannedShape) -> MotionShapePlan {
    let cue = shape.motion;
    let role = shape.base.role;
    let emphasis = shape.base.opacity.clamp(0.18, 0.9);
    let sign_x = if shape.base.x >= 0.5 { 1.0 } else { -1.0 };
    let sign_y = if shape.base.y >= 0.5 { 1.0 } else { -1.0 };
    let (drift_x, drift_y, pulse, spin): (f32, f32, f32, f32) = match cue {
        MotionCue::Still => (
            0.0,
            0.0,
            if role == SceneRole::Celestial {
                0.04
            } else {
                0.0
            },
            0.0,
        ),
        MotionCue::Bob => (0.02 * sign_x, 0.06 * sign_y, 0.05 * emphasis, 0.08 * sign_x),
        MotionCue::Sway => (0.07 * sign_x, 0.03 * sign_y, 0.04 * emphasis, 0.16 * sign_x),
        MotionCue::Drift => (0.05 * sign_x, 0.02 * sign_y, 0.03 * emphasis, 0.05 * sign_x),
        MotionCue::Pulse => (0.01 * sign_x, 0.01 * sign_y, 0.14 * emphasis, 0.08 * sign_x),
        MotionCue::Orbit => (0.05 * sign_x, 0.05 * sign_y, 0.08 * emphasis, 0.55 * sign_x),
        MotionCue::Ripple => (0.03 * sign_x, 0.05 * sign_y, 0.11 * emphasis, 0.12 * sign_x),
        MotionCue::Glimmer => (0.01 * sign_x, 0.02 * sign_y, 0.18 * emphasis, 0.24 * sign_x),
    };

    MotionShapePlan {
        base: shape.base,
        drift_x: drift_x.clamp(-0.2, 0.2),
        drift_y: drift_y.clamp(-0.2, 0.2),
        pulse: pulse.clamp(0.0, 0.4),
        spin: spin.clamp(-2.0, 2.0),
    }
}

fn default_motion_for_role(role: SceneRole) -> MotionCue {
    match role {
        SceneRole::Celestial => MotionCue::Pulse,
        SceneRole::Reflection => MotionCue::Ripple,
        SceneRole::Subject => MotionCue::Bob,
        _ => MotionCue::Still,
    }
}

fn normalize_image_plan(
    mut plan: ImagePlan,
    reference: Option<&ReferenceSummary>,
    minimum_shapes: usize,
) -> ImagePlan {
    plan.background_top = normalize_hex(&plan.background_top, "#253552");
    plan.background_bottom = normalize_hex(&plan.background_bottom, "#0C1322");
    plan.accent = normalize_hex(&plan.accent, "#F5C657");
    if let Some(reference) = reference {
        if !reference.palette.is_empty() {
            plan.background_top =
                blend_hex_colors_local(&plan.background_top, &reference.palette[0], 0.22);
            if let Some(color) = reference.palette.get(1) {
                plan.background_bottom =
                    blend_hex_colors_local(&plan.background_bottom, color, 0.2);
            }
            if let Some(color) = reference.palette.get(2) {
                plan.accent = blend_hex_colors_local(&plan.accent, color, 0.26);
            }
        }
    }
    plan.shapes = plan
        .shapes
        .into_iter()
        .take(24)
        .map(normalize_shape)
        .collect();
    if plan.shapes.len() < minimum_shapes {
        plan.shapes = fallback_image_plan(11, &default_settings(), 11, None).shapes;
    }
    plan
}

fn normalize_video_plan(
    mut plan: VideoPlan,
    reference: Option<&ReferenceSummary>,
    minimum_shapes: usize,
) -> VideoPlan {
    plan.background_top = normalize_hex(&plan.background_top, "#291F54");
    plan.background_bottom = normalize_hex(&plan.background_bottom, "#060914");
    plan.accent = normalize_hex(&plan.accent, "#F4682A");
    if let Some(reference) = reference {
        if !reference.palette.is_empty() {
            plan.background_top =
                blend_hex_colors_local(&plan.background_top, &reference.palette[0], 0.22);
            if let Some(color) = reference.palette.get(1) {
                plan.background_bottom =
                    blend_hex_colors_local(&plan.background_bottom, color, 0.2);
            }
            if let Some(color) = reference.palette.get(2) {
                plan.accent = blend_hex_colors_local(&plan.accent, color, 0.24);
            }
        }
    }
    plan.fps = plan.fps.clamp(8, 24);
    plan.frames = plan.frames.clamp(16, 480);
    plan.shapes = plan
        .shapes
        .into_iter()
        .take(24)
        .map(|shape| MotionShapePlan {
            base: normalize_shape(shape.base),
            drift_x: shape.drift_x.clamp(-0.2, 0.2),
            drift_y: shape.drift_y.clamp(-0.2, 0.2),
            pulse: shape.pulse.clamp(0.0, 0.4),
            spin: shape.spin.clamp(-2.0, 2.0),
        })
        .collect();
    if plan.shapes.len() < minimum_shapes {
        plan.shapes = fallback_video_plan(17, &default_settings(), 17, None).shapes;
    }
    plan
}

fn normalize_audio_plan(mut plan: AudioPlan) -> AudioPlan {
    plan.bpm = plan.bpm.clamp(60, 160);
    plan.duration_seconds = plan.duration_seconds.clamp(3.0, 10.0);
    plan.layers = plan
        .layers
        .into_iter()
        .take(4)
        .map(|layer| AudioLayerPlan {
            wave: layer.wave,
            gain: layer.gain.clamp(0.08, 0.45),
            pan: layer.pan.clamp(-1.0, 1.0),
            octave: layer.octave.clamp(2, 6),
            notes: if layer.notes.is_empty() {
                vec![0, 3, 7, 10]
            } else {
                layer
                    .notes
                    .into_iter()
                    .map(|note| note.clamp(0, 11))
                    .collect()
            },
            rhythm: if layer.rhythm.is_empty() {
                vec![1.0, 0.5, 0.5, 1.5]
            } else {
                layer
                    .rhythm
                    .into_iter()
                    .map(|value| value.clamp(0.25, 2.0))
                    .collect()
            },
        })
        .collect();
    if plan.layers.is_empty() {
        plan.layers = fallback_audio_plan(23, &default_settings(), 23, None).layers;
    }
    plan
}

fn normalize_shape(shape: ShapePlan) -> ShapePlan {
    ShapePlan {
        kind: shape.kind,
        role: shape.role,
        x: shape.x.clamp(0.05, 0.95),
        y: shape.y.clamp(0.05, 0.95),
        size: shape.size.clamp(0.05, 0.6),
        aspect: shape.aspect.clamp(0.2, 2.2),
        rotation: shape.rotation.rem_euclid(360.0),
        color: normalize_hex(&shape.color, "#5AB9EA"),
        secondary_color: normalize_hex(&shape.secondary_color, "#FCE38A"),
        opacity: shape.opacity.clamp(0.15, 1.0),
    }
}

fn normalize_hex(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.len() == 7 && value.starts_with('#') {
        value.to_uppercase()
    } else {
        fallback.to_string()
    }
}

fn blend_hex_colors_local(left: &str, right: &str, amount: f32) -> String {
    let mixed = mix_rgb_local(
        parse_hex_color_local(left),
        parse_hex_color_local(right),
        amount.clamp(0.0, 1.0),
    );
    format!("#{:02X}{:02X}{:02X}", mixed[0], mixed[1], mixed[2])
}

fn parse_hex_color_local(value: &str) -> [u8; 3] {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return [96, 111, 132];
    }

    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match (parse(0..2), parse(2..4), parse(4..6)) {
        (Some(r), Some(g), Some(b)) => [r, g, b],
        _ => [96, 111, 132],
    }
}

fn mix_rgb_local(left: [u8; 3], right: [u8; 3], t: f32) -> [u8; 3] {
    let blend = t.clamp(0.0, 1.0);
    [
        (left[0] as f32 * (1.0 - blend) + right[0] as f32 * blend) as u8,
        (left[1] as f32 * (1.0 - blend) + right[1] as f32 * blend) as u8,
        (left[2] as f32 * (1.0 - blend) + right[2] as f32 * blend) as u8,
    ]
}

fn fallback_image_plan(
    prompt_hash: u64,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
) -> ImagePlan {
    let mut rng = ChaCha8Rng::seed_from_u64(prompt_hash ^ u64::from(seed));
    let count = (4 + settings.steps / 12).clamp(4, 8);
    let palette = build_palette(&mut rng, reference);
    let shapes = (0..count)
        .map(|index| {
            let role = fallback_scene_role(index, count);
            let kind = fallback_shape_kind(&mut rng, role);
            let (x, y, size, aspect, rotation, opacity) =
                fallback_shape_layout(&mut rng, role, kind);
            ShapePlan {
                kind,
                role,
                x,
                y,
                size,
                aspect,
                rotation,
                color: palette[rng.random_range(0..palette.len())].clone(),
                secondary_color: palette[rng.random_range(0..palette.len())].clone(),
                opacity,
            }
        })
        .collect();

    ImagePlan {
        background_top: palette[0].clone(),
        background_bottom: palette[1].clone(),
        accent: palette[2].clone(),
        shapes,
    }
}

fn fallback_video_plan(
    prompt_hash: u64,
    settings: &GenerationSettings,
    seed: u32,
    reference: Option<&ReferenceSummary>,
) -> VideoPlan {
    let mut rng = ChaCha8Rng::seed_from_u64(prompt_hash ^ u64::from(seed) ^ 0xA11CE);
    let count = (4 + settings.steps / 16).clamp(4, 8);
    let palette = build_palette(&mut rng, reference);
    let shapes = (0..count)
        .map(|index| {
            let role = fallback_scene_role(index, count);
            let kind = fallback_shape_kind(&mut rng, role);
            let (x, y, size, aspect, rotation, opacity) =
                fallback_shape_layout(&mut rng, role, kind);
            MotionShapePlan {
                base: ShapePlan {
                    kind,
                    role,
                    x,
                    y,
                    size: size.clamp(0.08, 0.34),
                    aspect,
                    rotation,
                    color: palette[rng.random_range(0..palette.len())].clone(),
                    secondary_color: palette[rng.random_range(0..palette.len())].clone(),
                    opacity,
                },
                drift_x: match role {
                    SceneRole::Horizon | SceneRole::Ground => rng.random_range(-0.04..0.04),
                    SceneRole::Reflection => rng.random_range(-0.02..0.02),
                    _ => rng.random_range(-0.12..0.12),
                },
                drift_y: match role {
                    SceneRole::Celestial => rng.random_range(-0.04..0.04),
                    SceneRole::Reflection => rng.random_range(-0.08..0.08),
                    _ => rng.random_range(-0.12..0.12),
                },
                pulse: match role {
                    SceneRole::Celestial => rng.random_range(0.06..0.24),
                    SceneRole::Subject => rng.random_range(0.02..0.12),
                    _ => rng.random_range(0.04..0.18),
                },
                spin: match role {
                    SceneRole::Horizon | SceneRole::Ground => rng.random_range(-0.2..0.2),
                    _ => rng.random_range(-1.2..1.2),
                },
            }
        })
        .collect();

    VideoPlan {
        background_top: palette[0].clone(),
        background_bottom: palette[1].clone(),
        accent: palette[2].clone(),
        fps: settings.video_fps.clamp(8, 24) as u16,
        frames: settings.video_frame_count().clamp(16, 480) as u16,
        shapes,
    }
}

fn fallback_audio_plan(
    prompt_hash: u64,
    settings: &GenerationSettings,
    seed: u32,
    _reference: Option<&ReferenceSummary>,
) -> AudioPlan {
    let mut rng = ChaCha8Rng::seed_from_u64(prompt_hash ^ u64::from(seed) ^ 0x50A0D);
    let waves = [
        Waveform::Sine,
        Waveform::Triangle,
        Waveform::Saw,
        Waveform::Square,
    ];
    let note_sets = [
        vec![0, 3, 7, 10],
        vec![0, 4, 7, 11],
        vec![2, 5, 9, 0],
        vec![7, 9, 0, 4],
    ];

    let layer_count = (2 + settings.steps / 35).clamp(2, 4);
    let layers = (0..layer_count)
        .map(|index| AudioLayerPlan {
            wave: waves[(index as usize + rng.random_range(0..waves.len())) % waves.len()],
            gain: rng.random_range(0.10..0.28),
            pan: rng.random_range(-0.8..0.8),
            octave: rng.random_range(2..=5),
            notes: note_sets[rng.random_range(0..note_sets.len())].clone(),
            rhythm: vec![1.0, 0.5, 0.5, 1.5, 0.5, 1.0],
        })
        .collect();

    AudioPlan {
        bpm: rng.random_range(72..132),
        duration_seconds: (3.5 + settings.steps as f32 * 0.05).clamp(3.5, 8.0),
        layers,
    }
}

fn fallback_scene_role(index: u32, count: u32) -> SceneRole {
    match index {
        0 => SceneRole::Background,
        1 => SceneRole::Horizon,
        2 if count >= 5 => SceneRole::Ground,
        2 => SceneRole::Subject,
        3 => SceneRole::Subject,
        value if value == count.saturating_sub(1) => SceneRole::Detail,
        value if value % 3 == 0 => SceneRole::Celestial,
        value if value % 2 == 0 => SceneRole::Reflection,
        _ => SceneRole::Detail,
    }
}

fn fallback_shape_kind(rng: &mut ChaCha8Rng, role: SceneRole) -> ShapeKind {
    match role {
        SceneRole::Background => {
            if rng.random_bool(0.65) {
                ShapeKind::Rectangle
            } else {
                ShapeKind::Circle
            }
        }
        SceneRole::Horizon | SceneRole::Ground => {
            if rng.random_bool(0.7) {
                ShapeKind::Rectangle
            } else {
                ShapeKind::Line
            }
        }
        SceneRole::Subject => match rng.random_range(0..3) {
            0 => ShapeKind::Rectangle,
            1 => ShapeKind::Circle,
            _ => ShapeKind::Ring,
        },
        SceneRole::Celestial => {
            if rng.random_bool(0.55) {
                ShapeKind::Circle
            } else {
                ShapeKind::Ring
            }
        }
        SceneRole::Reflection => {
            if rng.random_bool(0.75) {
                ShapeKind::Line
            } else {
                ShapeKind::Rectangle
            }
        }
        SceneRole::Detail => random_shape(rng),
    }
}

fn fallback_shape_layout(
    rng: &mut ChaCha8Rng,
    role: SceneRole,
    kind: ShapeKind,
) -> (f32, f32, f32, f32, f32, f32) {
    match role {
        SceneRole::Background => (
            rng.random_range(0.18..0.82),
            rng.random_range(0.18..0.45),
            rng.random_range(0.24..0.48),
            rng.random_range(1.2..2.6),
            rng.random_range(0.0..360.0),
            rng.random_range(0.16..0.34),
        ),
        SceneRole::Horizon => (
            rng.random_range(0.35..0.65),
            rng.random_range(0.42..0.62),
            rng.random_range(0.18..0.34),
            rng.random_range(5.0..11.0),
            rng.random_range(-6.0..6.0),
            rng.random_range(0.22..0.4),
        ),
        SceneRole::Ground => (
            rng.random_range(0.35..0.65),
            rng.random_range(0.76..0.9),
            rng.random_range(0.16..0.28),
            rng.random_range(4.8..12.0),
            rng.random_range(-4.0..4.0),
            rng.random_range(0.2..0.34),
        ),
        SceneRole::Subject => (
            rng.random_range(0.3..0.7),
            rng.random_range(0.26..0.72),
            rng.random_range(0.12..0.28),
            match kind {
                ShapeKind::Line => rng.random_range(0.18..0.6),
                _ => rng.random_range(0.6..1.8),
            },
            rng.random_range(-18.0..18.0),
            rng.random_range(0.42..0.88),
        ),
        SceneRole::Celestial => (
            rng.random_range(0.12..0.88),
            rng.random_range(0.08..0.28),
            rng.random_range(0.05..0.16),
            match kind {
                ShapeKind::Ring => rng.random_range(0.35..0.8),
                _ => rng.random_range(0.8..1.2),
            },
            rng.random_range(0.0..360.0),
            rng.random_range(0.28..0.72),
        ),
        SceneRole::Reflection => (
            rng.random_range(0.22..0.78),
            rng.random_range(0.62..0.92),
            rng.random_range(0.08..0.18),
            match kind {
                ShapeKind::Line => rng.random_range(0.12..0.45),
                _ => rng.random_range(0.18..0.65),
            },
            if matches!(kind, ShapeKind::Line) {
                90.0
            } else {
                rng.random_range(-8.0..8.0)
            },
            rng.random_range(0.14..0.32),
        ),
        SceneRole::Detail => (
            rng.random_range(0.1..0.9),
            rng.random_range(0.12..0.88),
            rng.random_range(0.05..0.16),
            rng.random_range(0.3..1.6),
            rng.random_range(0.0..360.0),
            rng.random_range(0.22..0.68),
        ),
    }
}

fn build_palette(rng: &mut ChaCha8Rng, reference: Option<&ReferenceSummary>) -> Vec<String> {
    if let Some(reference) = reference {
        if !reference.palette.is_empty() {
            return reference.palette.clone();
        }
    }

    let base_hue = rng.random_range(0.0..360.0);
    vec![
        hsv_to_hex(base_hue, 0.72, 0.42),
        hsv_to_hex((base_hue + 44.0) % 360.0, 0.76, 0.18),
        hsv_to_hex((base_hue + 188.0) % 360.0, 0.68, 0.94),
        hsv_to_hex((base_hue + 250.0) % 360.0, 0.54, 0.78),
    ]
}

fn random_shape(rng: &mut ChaCha8Rng) -> ShapeKind {
    match rng.random_range(0..4) {
        0 => ShapeKind::Circle,
        1 => ShapeKind::Rectangle,
        2 => ShapeKind::Line,
        _ => ShapeKind::Ring,
    }
}

fn hsv_to_hex(hue: f32, saturation: f32, value: f32) -> String {
    let c = value * saturation;
    let x = c * (1.0 - (((hue / 60.0) % 2.0) - 1.0).abs());
    let m = value - c;
    let (r1, g1, b1) = match hue {
        h if (0.0..60.0).contains(&h) => (c, x, 0.0),
        h if (60.0..120.0).contains(&h) => (x, c, 0.0),
        h if (120.0..180.0).contains(&h) => (0.0, c, x),
        h if (180.0..240.0).contains(&h) => (0.0, x, c),
        h if (240.0..300.0).contains(&h) => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    format!(
        "#{:02X}{:02X}{:02X}",
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8
    )
}

fn extract_image_palette(path: &Path) -> Result<Vec<String>> {
    let image = image::open(path)
        .with_context(|| format!("failed to open image reference {}", path.display()))?;
    let thumb = image.thumbnail(32, 32).to_rgb8();

    let mut totals = [[0u64; 3]; 3];
    let mut counts = [0u64; 3];
    for (index, pixel) in thumb.pixels().enumerate() {
        let bucket = match index % 3 {
            0 => 0,
            1 => 1,
            _ => 2,
        };
        totals[bucket][0] += pixel[0] as u64;
        totals[bucket][1] += pixel[1] as u64;
        totals[bucket][2] += pixel[2] as u64;
        counts[bucket] += 1;
    }

    let mut palette = Vec::new();
    for bucket in 0..3 {
        if counts[bucket] > 0 {
            palette.push(format!(
                "#{:02X}{:02X}{:02X}",
                (totals[bucket][0] / counts[bucket]) as u8,
                (totals[bucket][1] / counts[bucket]) as u8,
                (totals[bucket][2] / counts[bucket]) as u8
            ));
        }
    }

    if palette.is_empty() {
        palette.push("#63748B".to_string());
    }
    Ok(palette)
}

fn prompt_hash<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn relative_to_native_path(relative: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for segment in relative.split('/') {
        if !segment.is_empty() {
            path.push(segment);
        }
    }
    path
}

fn default_settings() -> GenerationSettings {
    GenerationSettings {
        temperature: 0.6,
        steps: 28,
        cfg_scale: 7.5,
        resolution: crate::types::ResolutionPreset::Square512,
        video_resolution: VideoResolutionPreset::Square256,
        video_duration_seconds: 2,
        video_fps: 8,
        audio_duration_seconds: 10,
        low_vram_mode: false,
        seed: Some(1),
    }
}

fn compiler_settings() -> GenerationSettings {
    GenerationSettings {
        temperature: 0.2,
        steps: 20,
        cfg_scale: 12.0,
        resolution: crate::types::ResolutionPreset::Square512,
        video_resolution: VideoResolutionPreset::Square512,
        video_duration_seconds: 2,
        video_fps: 12,
        audio_duration_seconds: 10,
        low_vram_mode: false,
        seed: Some(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_schema_exposes_universal_scene_roles() {
        let schema = shape_schema();
        let roles = schema["properties"]["role"]["enum"]
            .as_array()
            .expect("role enum should exist");
        assert!(roles.iter().any(|value| value == "subject"));
        assert!(roles.iter().any(|value| value == "horizon"));
        assert!(roles.iter().any(|value| value == "reflection"));
    }

    #[test]
    fn fallback_image_plan_builds_a_scene_scaffold() {
        let plan = fallback_image_plan(123, &default_settings(), 77, None);
        let roles = plan
            .shapes
            .iter()
            .map(|shape| shape.role)
            .collect::<Vec<_>>();
        assert!(roles.contains(&SceneRole::Background));
        assert!(roles.contains(&SceneRole::Horizon));
        assert!(roles.contains(&SceneRole::Subject));
    }

    #[test]
    fn scene_graph_schema_exposes_reusable_motifs() {
        let schema = scene_element_schema(true);
        let motifs = schema["properties"]["motif"]["enum"]
            .as_array()
            .expect("motif enum should exist");
        assert!(motifs.iter().any(|value| value == "creature"));
        assert!(motifs.iter().any(|value| value == "swing"));
        assert!(motifs.iter().any(|value| value == "tree"));
        assert!(motifs.iter().any(|value| value == "figure"));
    }

    #[test]
    fn scene_graph_translation_preserves_key_scene_motifs() {
        let graph = SceneGraphPlan {
            background_top: "#88C8FF".to_string(),
            background_bottom: "#394B2C".to_string(),
            accent: "#F26D3D".to_string(),
            horizon_y: 0.46,
            ground_y: 0.82,
            focus_x: 0.5,
            focus_y: 0.56,
            elements: vec![
                SceneElement {
                    motif: SceneMotif::Swing,
                    role: SceneRole::Subject,
                    x: 0.48,
                    y: 0.55,
                    scale: 0.82,
                    emphasis: 0.9,
                    rotation: 0.0,
                    motion: MotionCue::Sway,
                },
                SceneElement {
                    motif: SceneMotif::Pair,
                    role: SceneRole::Subject,
                    x: 0.52,
                    y: 0.59,
                    scale: 0.78,
                    emphasis: 0.95,
                    rotation: 0.0,
                    motion: MotionCue::Bob,
                },
                SceneElement {
                    motif: SceneMotif::Tree,
                    role: SceneRole::Subject,
                    x: 0.23,
                    y: 0.56,
                    scale: 0.84,
                    emphasis: 0.7,
                    rotation: 0.0,
                    motion: MotionCue::Drift,
                },
                SceneElement {
                    motif: SceneMotif::Sun,
                    role: SceneRole::Celestial,
                    x: 0.78,
                    y: 0.18,
                    scale: 0.62,
                    emphasis: 0.6,
                    rotation: 0.0,
                    motion: MotionCue::Pulse,
                },
            ],
        };

        let plan = compose_image_plan(&graph, &default_settings(), 42, None, 14);
        assert!(plan.shapes.len() >= 8);
        assert!(plan.shapes.iter().any(|shape| {
            shape.role == SceneRole::Celestial
                && matches!(shape.kind, ShapeKind::Circle | ShapeKind::Ring)
        }));
        assert!(
            plan.shapes
                .iter()
                .filter(|shape| shape.role == SceneRole::Subject)
                .count()
                >= 4
        );
        assert!(
            plan.shapes
                .iter()
                .any(|shape| shape.role == SceneRole::Ground || shape.role == SceneRole::Horizon)
        );
    }

    #[test]
    fn repair_scene_graph_swaps_bad_animal_motifs_for_creature_layout() {
        let graph = SceneGraphPlan {
            background_top: "#87CEFF".to_string(),
            background_bottom: "#FFFFFF".to_string(),
            accent: "#1F1F1F".to_string(),
            horizon_y: 0.8,
            ground_y: 0.5,
            focus_x: 0.5,
            focus_y: 0.5,
            elements: vec![
                SceneElement {
                    motif: SceneMotif::Figure,
                    role: SceneRole::Background,
                    x: 0.5,
                    y: 0.5,
                    scale: 1.0,
                    emphasis: 1.0,
                    rotation: 0.0,
                    motion: MotionCue::Still,
                },
                SceneElement {
                    motif: SceneMotif::Pair,
                    role: SceneRole::Background,
                    x: 0.5,
                    y: 0.5,
                    scale: 1.0,
                    emphasis: 1.0,
                    rotation: 0.0,
                    motion: MotionCue::Still,
                },
                SceneElement {
                    motif: SceneMotif::Swing,
                    role: SceneRole::Background,
                    x: 0.5,
                    y: 0.5,
                    scale: 1.0,
                    emphasis: 1.0,
                    rotation: 0.0,
                    motion: MotionCue::Still,
                },
            ],
        };

        let repaired = repair_scene_graph(graph, "a dog standing in a field");
        assert!(
            repaired
                .elements
                .iter()
                .any(|element| element.motif == SceneMotif::Creature)
        );
        assert!(
            repaired
                .elements
                .iter()
                .all(|element| element.motif != SceneMotif::Swing)
        );
        assert!(repaired.elements.iter().any(|element| {
            element.motif == SceneMotif::Creature && element.role == SceneRole::Subject
        }));
    }

    #[test]
    fn prompt_assist_prompt_mentions_realism_negative_prompt_context() {
        let prompt = prompt_assist_prompt(
            "a merry-go-round in a field",
            Some("blurry, low quality"),
            GenerationStyle::Realism,
            MediaKind::Image,
            PromptAssistMode::Gentle,
            None,
            false,
        );
        assert!(prompt.contains("Target mode: realism."));
        assert!(prompt.contains("Original negative prompt: blurry, low quality."));
        assert!(
            prompt.contains("Expand short human prompts into a compact generator-ready brief.")
        );
        assert!(prompt.contains(
            "Use concrete image-generation cues: subject, setting, camera distance and angle"
        ));
    }

    #[test]
    fn spoken_text_heuristic_prefers_quoted_dialogue() {
        let prompt = r#"Make her say "Hello there, traveler." in a warm calm voice."#;
        assert_eq!(
            derive_spoken_text_heuristic(prompt),
            "Hello there, traveler."
        );
    }

    #[test]
    fn speech_direction_heuristic_strips_the_spoken_line() {
        let prompt = r#"Say "Hello there, traveler." in a warm calm voice with a gentle smile."#;
        let spoken = derive_spoken_text_heuristic(prompt);
        let direction = derive_speech_direction_heuristic(prompt, Some(&spoken))
            .expect("direction should be derived");
        assert!(direction.contains("warm calm voice"));
        assert!(!direction.contains("Hello there, traveler."));
    }

    #[test]
    fn polish_compiled_prompt_strips_expository_filler_into_generator_cues() {
        let polished = polish_compiled_prompt(
            "The user said a cat in a tree. The scene shows a fluffy white cat, golden hour light, perched on a low branch.",
            &[
                "oak tree".to_string(),
                "soft sunlight".to_string(),
                "fluffy white cat".to_string(),
            ],
            GenerationStyle::Realism,
            MediaKind::Image,
        );
        assert!(!polished.contains("The user said"));
        assert!(!polished.contains("The scene shows"));
        assert!(polished.contains("fluffy white cat"));
        assert!(polished.contains("golden hour light"));
        assert!(polished.contains("oak tree"));
    }

    #[test]
    fn merge_negative_prompts_keeps_user_and_compiler_constraints() {
        let merged = merge_negative_prompts(
            Some("blurry, low quality"),
            Some("extra limbs, bad anatomy"),
        )
        .expect("merged negative prompt");
        assert!(merged.contains("blurry"));
        assert!(merged.contains("extra limbs"));
        assert!(merged.contains("bad anatomy"));
    }

    #[test]
    fn merge_negative_prompts_dedupes_punctuation_variants() {
        let merged = merge_negative_prompts(Some("blurry."), Some("blurry, low quality"))
            .expect("merged negative prompt");
        assert_eq!(merged, "blurry, low quality");
    }
}
