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
            Self::Video => "avi",
            Self::Audio => "wav",
        }
    }

    pub fn output_mime(self) -> &'static str {
        match self {
            Self::Image => "image/png",
            Self::Gif => "image/gif",
            Self::Video => "video/x-msvideo",
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

fn default_low_vram_mode() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    pub temperature: f32,
    pub steps: u32,
    pub cfg_scale: f32,
    pub resolution: ResolutionPreset,
    #[serde(default = "default_video_resolution")]
    pub video_resolution: VideoResolutionPreset,
    #[serde(default = "default_video_duration_seconds")]
    pub video_duration_seconds: u32,
    #[serde(default = "default_video_fps")]
    pub video_fps: u32,
    #[serde(default = "default_low_vram_mode")]
    pub low_vram_mode: bool,
    pub seed: Option<u64>,
}

impl GenerationSettings {
    pub fn dimensions_for(&self, kind: MediaKind) -> (u32, u32) {
        match kind {
            MediaKind::Gif | MediaKind::Video => self.video_resolution.dimensions(),
            MediaKind::Image | MediaKind::Audio => self.resolution.dimensions(),
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
            MediaKind::Image | MediaKind::Audio => self.resolution.label().to_string(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateAccepted {
    pub job_id: Uuid,
    pub used_seed: u64,
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
    pub requires_end_image_reference: bool,
    #[serde(default)]
    pub supports_end_image_reference: bool,
    #[serde(default)]
    pub supports_video_reference: bool,
    pub supports_audio_reference: bool,
    pub supports_voice_output: bool,
    pub mmproj_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAsset {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub kind: MediaKind,
    pub url: String,
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
    pub prompt_assist: PromptAssistMode,
    #[serde(default)]
    pub interpreter_model: Option<String>,
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
