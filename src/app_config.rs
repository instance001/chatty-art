use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::types::{CloudConfig, CloudLaneAssignments, CloudProviderEntry};

pub fn default_config_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("config").join("preferences.json")
}

pub fn load_cloud_config(path: &Path) -> Result<CloudConfig> {
    if !path.is_file() {
        return Ok(CloudConfig::default());
    }
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut config: CloudConfig =
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
    normalize_cloud_config(&mut config);
    Ok(config)
}

pub fn save_cloud_config(path: &Path, config: &CloudConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create config dir {}", parent.display()))?;
    }
    let mut cloned = config.clone();
    normalize_cloud_config(&mut cloned);
    let bytes = serde_json::to_vec_pretty(&cloned).context("serialize cloud config")?;
    std::fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn normalize_cloud_config(config: &mut CloudConfig) {
    config.cloud_providers.retain_mut(|entry| normalize_provider_entry(entry));
    config
        .cloud_providers
        .sort_by(|left, right| left.display_name.to_lowercase().cmp(&right.display_name.to_lowercase()));
    config.lane_assignments = normalize_lane_assignments_for_providers(
        config.lane_assignments.clone(),
        &config.cloud_providers,
    );
}

pub fn normalize_lane_assignments(mut assignments: CloudLaneAssignments) -> CloudLaneAssignments {
    assignments.prompt_assist = normalize_lane_value(&assignments.prompt_assist, "local_auto");
    assignments.vision_assist = normalize_lane_value(&assignments.vision_assist, "local_auto");
    assignments.media_generation = normalize_lane_value(&assignments.media_generation, "local_only");
    assignments
}

pub fn normalize_lane_assignments_for_providers(
    assignments: CloudLaneAssignments,
    providers: &[CloudProviderEntry],
) -> CloudLaneAssignments {
    let mut normalized = normalize_lane_assignments(assignments);
    normalized.prompt_assist = normalize_lane_selection(
        &normalized.prompt_assist,
        "local_auto",
        providers,
        prompt_assist_provider_ready,
    );
    normalized.vision_assist = normalize_lane_selection(
        &normalized.vision_assist,
        "local_auto",
        providers,
        vision_assist_provider_ready,
    );
    normalized.media_generation = normalize_lane_selection(
        &normalized.media_generation,
        "local_only",
        providers,
        media_generation_provider_ready,
    );
    normalized
}

fn normalize_provider_entry(entry: &mut CloudProviderEntry) -> bool {
    entry.id = entry.id.trim().to_string();
    entry.display_name = entry.display_name.trim().to_string();
    entry.base_url = entry.base_url.trim().trim_end_matches('/').to_string();
    entry.prompt_assist_model_name = entry.prompt_assist_model_name.trim().to_string();
    entry.vision_model_name = entry.vision_model_name.trim().to_string();
    entry.image_generation_model_name = entry.image_generation_model_name.trim().to_string();
    entry.video_generation_model_name = entry.video_generation_model_name.trim().to_string();
    entry.audio_generation_model_name = entry.audio_generation_model_name.trim().to_string();
    entry.audio_generation_voice = entry.audio_generation_voice.trim().to_string();
    if entry.display_name.is_empty() {
        entry.display_name = if !entry.prompt_assist_model_name.is_empty() {
            entry.prompt_assist_model_name.clone()
        } else if !entry.vision_model_name.is_empty() {
            entry.vision_model_name.clone()
        } else if !entry.image_generation_model_name.is_empty() {
            entry.image_generation_model_name.clone()
        } else if !entry.video_generation_model_name.is_empty() {
            entry.video_generation_model_name.clone()
        } else if !entry.audio_generation_model_name.is_empty() {
            entry.audio_generation_model_name.clone()
        } else {
            entry.id.clone()
        };
    }
    !entry.id.is_empty()
}

fn normalize_lane_value(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_lane_selection(
    value: &str,
    fallback: &str,
    providers: &[CloudProviderEntry],
    is_ready: fn(&CloudProviderEntry) -> bool,
) -> String {
    let trimmed = value.trim();
    let Some(provider_id) = trimmed.strip_prefix("cloud:").map(str::trim) else {
        return normalize_lane_value(trimmed, fallback);
    };
    if provider_id.is_empty() {
        return fallback.to_string();
    }
    if providers
        .iter()
        .any(|entry| entry.id == provider_id && is_ready(entry))
    {
        format!("cloud:{provider_id}")
    } else {
        fallback.to_string()
    }
}

fn prompt_assist_provider_ready(entry: &CloudProviderEntry) -> bool {
    entry.enabled
        && entry.capabilities.text_assist
        && !entry.prompt_assist_model_name.trim().is_empty()
}

fn vision_assist_provider_ready(entry: &CloudProviderEntry) -> bool {
    entry.enabled
        && entry.capabilities.vision_assist
        && !entry.vision_model_name.trim().is_empty()
}

fn media_generation_provider_ready(entry: &CloudProviderEntry) -> bool {
    entry.enabled
        && ((entry.capabilities.image_generation
            && !entry.image_generation_model_name.trim().is_empty())
            || (entry.capabilities.video_generation
                && !entry.video_generation_model_name.trim().is_empty())
            || (entry.capabilities.audio_generation
                && !entry.audio_generation_model_name.trim().is_empty()))
}
