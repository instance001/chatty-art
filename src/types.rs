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
    Cloud,
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloudProviderKind {
    OpenAi,
    #[default]
    OpenAiCompatible,
    Anthropic,
    Gemini,
    XAiGrok,
    DeepSeek,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderCapabilities {
    #[serde(default)]
    pub text_assist: bool,
    #[serde(default)]
    pub vision_assist: bool,
    #[serde(default)]
    pub image_generation: bool,
    #[serde(default)]
    pub video_generation: bool,
    #[serde(default)]
    pub audio_generation: bool,
}

impl Default for CloudProviderCapabilities {
    fn default() -> Self {
        Self {
            text_assist: true,
            vision_assist: false,
            image_generation: false,
            video_generation: false,
            audio_generation: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudVerificationStatus {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checked_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub provider_kind: CloudProviderKind,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub prompt_assist_model_name: String,
    #[serde(default)]
    pub vision_model_name: String,
    #[serde(default)]
    pub image_generation_model_name: String,
    #[serde(default)]
    pub video_generation_model_name: String,
    #[serde(default)]
    pub audio_generation_model_name: String,
    #[serde(default)]
    pub audio_generation_voice: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub capabilities: CloudProviderCapabilities,
    #[serde(default)]
    pub prompt_assist_verification: CloudVerificationStatus,
    #[serde(default)]
    pub vision_assist_verification: CloudVerificationStatus,
    #[serde(default)]
    pub media_generation_verification: CloudVerificationStatus,
}

impl Default for CloudProviderEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            display_name: String::new(),
            provider_kind: CloudProviderKind::default(),
            base_url: String::new(),
            prompt_assist_model_name: String::new(),
            vision_model_name: String::new(),
            image_generation_model_name: String::new(),
            video_generation_model_name: String::new(),
            audio_generation_model_name: String::new(),
            audio_generation_voice: String::new(),
            enabled: true,
            capabilities: CloudProviderCapabilities::default(),
            prompt_assist_verification: CloudVerificationStatus::default(),
            vision_assist_verification: CloudVerificationStatus::default(),
            media_generation_verification: CloudVerificationStatus::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudLaneAssignments {
    #[serde(default = "default_prompt_assist_lane")]
    pub prompt_assist: String,
    #[serde(default = "default_vision_assist_lane")]
    pub vision_assist: String,
    #[serde(default = "default_media_generation_lane")]
    pub media_generation: String,
}

impl Default for CloudLaneAssignments {
    fn default() -> Self {
        Self {
            prompt_assist: default_prompt_assist_lane(),
            vision_assist: default_vision_assist_lane(),
            media_generation: default_media_generation_lane(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudConfig {
    #[serde(default)]
    pub api_key_lanes_enabled: bool,
    #[serde(default)]
    pub cloud_providers: Vec<CloudProviderEntry>,
    #[serde(default)]
    pub lane_assignments: CloudLaneAssignments,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            api_key_lanes_enabled: false,
            cloud_providers: Vec::new(),
            lane_assignments: CloudLaneAssignments::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderSummary {
    pub id: String,
    pub display_name: String,
    pub provider_kind: CloudProviderKind,
    pub base_url: String,
    pub prompt_assist_model_name: String,
    pub vision_model_name: String,
    pub image_generation_model_name: String,
    pub video_generation_model_name: String,
    pub audio_generation_model_name: String,
    pub audio_generation_voice: String,
    pub enabled: bool,
    pub capabilities: CloudProviderCapabilities,
    pub prompt_assist_verification: CloudVerificationStatus,
    pub vision_assist_verification: CloudVerificationStatus,
    pub media_generation_verification: CloudVerificationStatus,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderUpsertRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub display_name: String,
    #[serde(default)]
    pub provider_kind: CloudProviderKind,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub prompt_assist_model_name: String,
    #[serde(default)]
    pub vision_model_name: String,
    #[serde(default)]
    pub image_generation_model_name: String,
    #[serde(default)]
    pub video_generation_model_name: String,
    #[serde(default)]
    pub audio_generation_model_name: String,
    #[serde(default)]
    pub audio_generation_voice: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderDeleteRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderVerifyRequest {
    pub id: String,
    #[serde(default)]
    pub lane: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderUpsertResponse {
    pub provider: CloudProviderSummary,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderDeleteResponse {
    pub id: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProviderVerifyResponse {
    pub provider: CloudProviderSummary,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudProvidersResponse {
    pub api_key_lanes_enabled: bool,
    pub providers: Vec<CloudProviderSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudLaneAssignmentsResponse {
    pub lane_assignments: CloudLaneAssignments,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudLaneAssignmentsUpdateRequest {
    pub lane_assignments: CloudLaneAssignments,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudLaneAssignmentsUpdateResponse {
    pub lane_assignments: CloudLaneAssignments,
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
    VideoLandscape720,
    VideoPortrait720,
    VideoLandscape1024,
    VideoPortrait1024,
    VideoLandscape1080,
    VideoPortrait1080,
}

impl VideoResolutionPreset {
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Square256 => (256, 256),
            Self::Square512 => (512, 512),
            Self::Square768 => (768, 768),
            Self::VideoLandscape720 => (1280, 720),
            Self::VideoPortrait720 => (720, 1280),
            Self::VideoLandscape1024 => (1792, 1024),
            Self::VideoPortrait1024 => (1024, 1792),
            Self::VideoLandscape1080 => (1920, 1080),
            Self::VideoPortrait1080 => (1080, 1920),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Square256 => "256x256",
            Self::Square512 => "512x512",
            Self::Square768 => "768x768",
            Self::VideoLandscape720 => "1280x720",
            Self::VideoPortrait720 => "720x1280",
            Self::VideoLandscape1024 => "1792x1024",
            Self::VideoPortrait1024 => "1024x1792",
            Self::VideoLandscape1080 => "1920x1080",
            Self::VideoPortrait1080 => "1080x1920",
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

fn default_true() -> bool {
    true
}

fn default_prompt_assist_lane() -> String {
    "local_auto".to_string()
}

fn default_vision_assist_lane() -> String {
    "local_auto".to_string()
}

fn default_media_generation_lane() -> String {
    "local_only".to_string()
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
    #[serde(default = "default_batch_count")]
    pub batch_count: u32,
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
    pub selected_prompt_model: Option<String>,
    #[serde(default)]
    pub selected_vision_model: Option<String>,
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
    #[serde(default)]
    pub manual_preserve_items: Vec<String>,
    #[serde(default)]
    pub manual_change_targets: Vec<String>,
    #[serde(default)]
    pub manual_avoid_items: Vec<String>,
}

fn default_batch_count() -> u32 {
    1
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
    pub fn normalized_batch_count(&self) -> u32 {
        self.batch_count.clamp(1, 64)
    }

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
    #[serde(default)]
    pub used_seed: Option<u64>,
    #[serde(default = "default_batch_count")]
    pub batch_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    pub job_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAccepted {
    pub job_id: Uuid,
    pub accepted: bool,
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
    #[serde(default)]
    pub vision_model: Option<String>,
    #[serde(default)]
    pub vision_summary: Option<String>,
    #[serde(default)]
    pub vision_error: Option<String>,
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
    pub source: AssetSource,
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
    pub prompt_assist_route: Option<String>,
    #[serde(default)]
    pub vision_model: Option<String>,
    #[serde(default)]
    pub vision_assist_route: Option<String>,
    #[serde(default)]
    pub output_route: Option<String>,
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
    Canceled {
        job_id: Uuid,
        message: String,
    },
    Error {
        job_id: Uuid,
        message: String,
    },
}
