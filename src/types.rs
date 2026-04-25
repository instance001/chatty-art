use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Gif,
    Video,
    Audio,
}

impl MediaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Gif => "gif",
            Self::Video => "video",
            Self::Audio => "audio",
        }
    }

    pub fn output_extension(self) -> &'static str {
        match self {
            Self::Image => "png",
            Self::Gif => "gif",
            Self::Video => "mp4",
            Self::Audio => "wav",
        }
    }

    pub fn output_mime(self) -> &'static str {
        match self {
            Self::Image => "image/png",
            Self::Gif => "image/gif",
            Self::Video => "video/mp4",
            Self::Audio => "audio/wav",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceIntent {
    #[default]
    Guide,
    Edit,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStyle {
    #[default]
    Expressive,
    Realism,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromptAssistMode {
    #[default]
    Off,
    Gentle,
    Strong,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelBackend {
    #[default]
    LlamaCpp,
    StableDiffusionCpp,
    AudioRuntime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAcceleration {
    Vulkan,
    CpuOnly,
    BuildPending,
    IncompleteTree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendRuntimeStatus {
    pub backend: ModelBackend,
    pub label: String,
    pub acceleration: RuntimeAcceleration,
    pub note: String,
    #[serde(default)]
    pub tooling_label: Option<String>,
    #[serde(default)]
    pub tooling_note: Option<String>,
    #[serde(default)]
    pub tooling_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub expressive: BackendRuntimeStatus,
    pub realism: BackendRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub platform: String,
    pub gpu_label: String,
    pub dedicated_vram_gb: Option<f32>,
    pub shared_memory_gb: Option<f32>,
    pub note: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionPreset {
    Square512,
    Square768,
    Landscape720,
    Portrait768,
    Landscape1024,
    Poster1024,
}

impl ResolutionPreset {
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Square512 => (512, 512),
            Self::Square768 => (768, 768),
            Self::Landscape720 => (1280, 720),
            Self::Portrait768 => (768, 1024),
            Self::Landscape1024 => (1024, 768),
            Self::Poster1024 => (1024, 1280),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Square512 => "Square 512",
            Self::Square768 => "Square 768",
            Self::Landscape720 => "Landscape 1280x720",
            Self::Portrait768 => "Portrait 768x1024",
            Self::Landscape1024 => "Landscape 1024x768",
            Self::Poster1024 => "Poster 1024x1280",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoResolutionPreset {
    Square256,
    #[default]
    Square512,
    Square768,
}

impl VideoResolutionPreset {
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Square256 => (256, 256),
            Self::Square512 => (512, 512),
            Self::Square768 => (768, 768),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Square256 => "256x256",
            Self::Square512 => "512x512",
            Self::Square768 => "768x768",
        }
    }
}

fn default_video_resolution() -> VideoResolutionPreset {
    VideoResolutionPreset::Square512
}

fn default_video_duration_seconds() -> u32 {
    2
}

fn default_video_fps() -> u32 {
    12
}

fn default_audio_duration_seconds() -> u32 {
    10
}

fn default_low_vram_mode() -> bool {
    false
}

fn default_sampler() -> String {
    "euler".to_string()
}

fn default_scheduler() -> String {
    "default".to_string()
}

fn default_reference_strength() -> f32 {
    0.8
}

fn default_flow_shift() -> f32 {
    3.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    pub temperature: f32,
    pub steps: u32,
    pub cfg_scale: f32,
    #[serde(default = "default_sampler")]
    pub sampler: String,
    #[serde(default = "default_scheduler")]
    pub scheduler: String,
    #[serde(default = "default_reference_strength")]
    pub reference_strength: f32,
    #[serde(default = "default_flow_shift")]
    pub flow_shift: f32,
    pub resolution: ResolutionPreset,
    #[serde(default = "default_video_resolution")]
    pub video_resolution: VideoResolutionPreset,
    #[serde(default = "default_video_duration_seconds")]
    pub video_duration_seconds: u32,
    #[serde(default = "default_video_fps")]
    pub video_fps: u32,
    #[serde(default = "default_audio_duration_seconds")]
    pub audio_duration_seconds: u32,
    #[serde(default = "default_low_vram_mode")]
    pub low_vram_mode: bool,
    pub seed: Option<u64>,
}

impl GenerationSettings {
    pub fn dimensions_for(&self, kind: MediaKind) -> (u32, u32) {
        match kind {
            MediaKind::Gif | MediaKind::Video => self.video_resolution.dimensions(),
            MediaKind::Image => self.resolution.dimensions(),
            MediaKind::Audio => (512, 512),
        }
    }

    pub fn resolution_label_for(&self, kind: MediaKind) -> String {
        match kind {
            MediaKind::Gif | MediaKind::Video => format!(
                "{} | {}s @ {} FPS ({} frames)",
                self.video_resolution.label(),
                self.video_duration_seconds,
                self.video_fps,
                self.video_frame_count()
            ),
            MediaKind::Image => self.resolution.label().to_string(),
            MediaKind::Audio => format!(
                "{}s audio | {} steps | CFG {:.1}",
                self.audio_duration_seconds.max(1),
                self.steps,
                self.cfg_scale
            ),
        }
    }

    pub fn video_frame_count(&self) -> u32 {
        self.video_duration_seconds
            .max(1)
            .saturating_mul(self.video_fps.max(1))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: Option<String>,
    #[serde(default)]
    pub selected_lora: Option<String>,
    #[serde(default)]
    pub selected_lora_weight: Option<f32>,
    #[serde(default)]
    pub selected_loras: Vec<LoraSelection>,
    #[serde(default)]
    pub prompt_assist: PromptAssistMode,
    pub model: String,
    pub kind: MediaKind,
    #[serde(default)]
    pub style: GenerationStyle,
    pub settings: GenerationSettings,
    #[serde(default)]
    pub reference_asset: Option<String>,
    #[serde(default)]
    pub reference_intent: ReferenceIntent,
    #[serde(default)]
    pub end_reference_asset: Option<String>,
    #[serde(default)]
    pub control_reference_asset: Option<String>,
    #[serde(default)]
    pub prepared_prompt: Option<String>,
    #[serde(default)]
    pub prepared_negative_prompt: Option<String>,
    #[serde(default)]
    pub prepared_note: Option<String>,
    #[serde(default)]
    pub prepared_interpreter_model: Option<String>,
    #[serde(default)]
    pub prepared_spoken_text: Option<String>,
    #[serde(default)]
    pub audio_literal_prompt: Option<String>,
    #[serde(default)]
    pub audio_segments: Vec<AudioPromptSegment>,
    #[serde(default)]
    pub manual_focus_tags: Vec<String>,
    #[serde(default)]
    pub manual_assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraSelection {
    pub id: String,
    #[serde(default)]
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioPromptSegment {
    #[serde(default)]
    pub label: Option<String>,
    pub literal: String,
    #[serde(default)]
    pub same_time_as_previous: bool,
}

impl AudioPromptSegment {
    pub fn normalized(&self) -> Option<Self> {
        let literal = self.literal.trim();
        if literal.is_empty() {
            return None;
        }

        let label = self
            .label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        Some(Self {
            label,
            literal: literal.to_string(),
            same_time_as_previous: self.same_time_as_previous,
        })
    }
}

impl GenerateRequest {
    pub fn normalized_audio_segments(&self) -> Vec<AudioPromptSegment> {
        self.audio_segments
            .iter()
            .filter_map(AudioPromptSegment::normalized)
            .collect()
    }

    pub fn has_audio_literal_content(&self) -> bool {
        self.audio_literal_prompt
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
            || !self.normalized_audio_segments().is_empty()
    }

    pub fn combined_audio_literal_prompt(&self) -> Option<String> {
        if let Some(single) = self
            .audio_literal_prompt
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(single.to_string());
        }

        let segments = self.normalized_audio_segments();
        if segments.is_empty() {
            return None;
        }

        Some(
            segments
                .into_iter()
                .map(|segment| segment.literal)
                .collect::<Vec<_>>()
                .join(" | "),
        )
    }

    pub fn normalized_lora_weight(&self) -> Option<f32> {
        self.selected_lora_weight
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 2.0))
    }

    pub fn normalized_lora_selections(&self) -> Vec<LoraSelection> {
        use std::collections::HashSet;

        let mut normalized = Vec::new();
        let mut seen = HashSet::new();

        for selection in &self.selected_loras {
            let id = selection.id.trim();
            if id.is_empty() {
                continue;
            }

            let key = id.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }

            normalized.push(LoraSelection {
                id: id.to_string(),
                weight: selection
                    .weight
                    .filter(|value| value.is_finite())
                    .map(|value| value.clamp(0.0, 2.0)),
            });
        }

        if normalized.is_empty() {
            if let Some(id) = self
                .selected_lora
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                normalized.push(LoraSelection {
                    id: id.to_string(),
                    weight: self.normalized_lora_weight(),
                });
            }
        }

        normalized
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAccepted {
    pub job_id: Uuid,
    pub used_seed: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EstimateConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEstimate {
    pub min_seconds: u32,
    pub max_seconds: u32,
    pub confidence: EstimateConfidence,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareResponse {
    pub model: String,
    pub kind: MediaKind,
    pub style: GenerationStyle,
    pub original_prompt: String,
    pub prepared_prompt: String,
    #[serde(default)]
    pub prepared_spoken_text: Option<String>,
    pub effective_negative_prompt: Option<String>,
    pub prompt_assist: PromptAssistMode,
    pub interpreter_model: Option<String>,
    pub note: String,
    pub assumptions: Vec<String>,
    pub focus_tags: Vec<String>,
    pub used_original_prompt: bool,
    pub resolution_label: String,
    pub estimated_frames: Option<u32>,
    pub estimated_time: TimeEstimate,
    pub hardware_note: String,
    pub reference_note: Option<String>,
    #[serde(default)]
    pub selected_lora_name: Option<String>,
    #[serde(default)]
    pub selected_lora_weight: Option<f32>,
    #[serde(default)]
    pub selected_lora_labels: Vec<String>,
    #[serde(default)]
    pub supports_voice_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraInfo {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub relative_path: String,
    pub family: String,
    pub family_key: String,
    pub runtime_supported: bool,
    pub compatibility_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub file_name: String,
    pub relative_path: String,
    pub family: String,
    pub backend: ModelBackend,
    pub generation_style: GenerationStyle,
    pub runtime_supported: bool,
    pub compatibility_note: String,
    pub supported_kinds: Vec<MediaKind>,
    pub requires_reference: bool,
    pub supports_image_reference: bool,
    #[serde(default)]
    pub supports_reference_strength: bool,
    #[serde(default)]
    pub requires_end_image_reference: bool,
    #[serde(default)]
    pub supports_end_image_reference: bool,
    #[serde(default)]
    pub supports_video_reference: bool,
    pub supports_audio_reference: bool,
    pub supports_voice_output: bool,
    pub mmproj_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetSource {
    Input,
    Output,
}

impl AssetSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAsset {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub kind: MediaKind,
    pub url: String,
    pub source: AssetSource,
}

impl InputAsset {
    pub fn native_relative_path(&self) -> PathBuf {
        self.relative_path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect()
    }

    pub fn disk_path(&self, input_root: &Path, outputs_root: &Path) -> PathBuf {
        let root = match self.source {
            AssetSource::Input => input_root,
            AssetSource::Output => outputs_root,
        };
        root.join(self.native_relative_path())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSummary {
    pub name: String,
    pub relative_path: String,
    pub kind: MediaKind,
    pub palette: Vec<String>,
    pub intent: ReferenceIntent,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEntry {
    pub id: String,
    pub job_id: Uuid,
    pub kind: MediaKind,
    #[serde(default)]
    pub style: GenerationStyle,
    #[serde(default)]
    pub backend: ModelBackend,
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: Option<String>,
    #[serde(default)]
    pub compiled_prompt: Option<String>,
    #[serde(default)]
    pub spoken_text: Option<String>,
    #[serde(default)]
    pub prompt_assist: PromptAssistMode,
    #[serde(default)]
    pub interpreter_model: Option<String>,
    #[serde(default)]
    pub lora_name: Option<String>,
    #[serde(default)]
    pub lora_weight: Option<f32>,
    #[serde(default)]
    pub lora_labels: Vec<String>,
    pub file_name: String,
    pub relative_path: String,
    pub url: String,
    pub mime: String,
    pub created_at: DateTime<Utc>,
    pub settings: GenerationSettings,
    pub used_seed: u64,
    pub resolution_label: String,
    pub reference_asset: Option<String>,
    #[serde(default)]
    pub reference_intent: Option<ReferenceIntent>,
    #[serde(default)]
    pub end_reference_asset: Option<String>,
    #[serde(default)]
    pub control_reference_asset: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Progress {
        job_id: Uuid,
        percent: f32,
        phase: String,
        message: String,
    },
    Completed {
        job_id: Uuid,
        output: OutputEntry,
    },
    Error {
        job_id: Uuid,
        message: String,
    },
}
