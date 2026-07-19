use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use reqwest::{
    StatusCode,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
    multipart::{Form, Part},
};
use serde_json::json;
use std::{path::Path, time::Duration};

use crate::types::{CloudProviderCapabilities, CloudProviderEntry, CloudProviderKind};

pub fn default_capabilities(kind: CloudProviderKind) -> CloudProviderCapabilities {
    match kind {
        CloudProviderKind::OpenAi => CloudProviderCapabilities {
            text_assist: true,
            vision_assist: true,
            image_generation: true,
            video_generation: true,
            audio_generation: true,
        },
        CloudProviderKind::OpenAiCompatible => CloudProviderCapabilities {
            text_assist: true,
            vision_assist: true,
            image_generation: false,
            video_generation: false,
            audio_generation: false,
        },
        CloudProviderKind::Anthropic => CloudProviderCapabilities {
            text_assist: true,
            vision_assist: true,
            image_generation: false,
            video_generation: false,
            audio_generation: false,
        },
        CloudProviderKind::Gemini => CloudProviderCapabilities {
            text_assist: true,
            vision_assist: true,
            image_generation: true,
            video_generation: true,
            audio_generation: true,
        },
        CloudProviderKind::XAiGrok => CloudProviderCapabilities {
            text_assist: true,
            vision_assist: false,
            image_generation: false,
            video_generation: false,
            audio_generation: false,
        },
        CloudProviderKind::DeepSeek => CloudProviderCapabilities {
            text_assist: true,
            vision_assist: false,
            image_generation: false,
            video_generation: false,
            audio_generation: false,
        },
    }
}

pub fn default_base_url(kind: CloudProviderKind) -> &'static str {
    match kind {
        CloudProviderKind::OpenAi => "https://api.openai.com/v1",
        CloudProviderKind::OpenAiCompatible => "",
        CloudProviderKind::Anthropic => "https://api.anthropic.com/v1",
        CloudProviderKind::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
        CloudProviderKind::XAiGrok => "https://api.x.ai/v1",
        CloudProviderKind::DeepSeek => "https://api.deepseek.com/v1",
    }
}

pub async fn verify_prompt_assist(entry: &CloudProviderEntry, api_key: &str) -> Result<String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.prompt_assist_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Prompt Assist model name is missing"));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi
        | CloudProviderKind::OpenAiCompatible
        | CloudProviderKind::Gemini
        | CloudProviderKind::XAiGrok
        | CloudProviderKind::DeepSeek => {
            verify_openai_chat_like(&client, entry, &base_url, api_key, model_name).await
        }
        CloudProviderKind::Anthropic => {
            verify_anthropic(&client, &base_url, api_key, model_name).await
        }
    }
}

pub async fn generate_prompt_assist_json(
    entry: &CloudProviderEntry,
    api_key: &str,
    prompt: &str,
    schema: &serde_json::Value,
    max_tokens: usize,
) -> Result<String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.prompt_assist_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Prompt Assist model name is missing"));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi
        | CloudProviderKind::OpenAiCompatible
        | CloudProviderKind::Gemini
        | CloudProviderKind::XAiGrok
        | CloudProviderKind::DeepSeek => {
            request_openai_chat_like_json(&client, entry, &base_url, api_key, model_name, prompt, schema, max_tokens).await
        }
        CloudProviderKind::Anthropic => {
            request_anthropic_json(&client, &base_url, api_key, model_name, prompt, schema, max_tokens).await
        }
    }
}

pub async fn verify_vision_assist(entry: &CloudProviderEntry, api_key: &str) -> Result<String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.vision_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Vision Assist model name is missing"));
    }
    if !entry.capabilities.vision_assist {
        return Err(anyhow!(
            "This provider is not marked as Vision Assist capable."
        ));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi | CloudProviderKind::OpenAiCompatible | CloudProviderKind::Gemini => {
            verify_openai_chat_like_with_note(
                &client,
                entry,
                &base_url,
                api_key,
                model_name,
                "Reply with OK.",
                "Vision Assist",
            )
            .await
        }
        CloudProviderKind::Anthropic => {
            verify_anthropic_vision(&client, &base_url, api_key, model_name).await
        }
        CloudProviderKind::XAiGrok => Err(anyhow!(
            "Vision Assist is not wired for xAI Grok in Chatty-art yet."
        )),
        CloudProviderKind::DeepSeek => Err(anyhow!(
            "Vision Assist is not wired for DeepSeek in Chatty-art yet."
        )),
    }
}

pub async fn generate_vision_assist_json(
    entry: &CloudProviderEntry,
    api_key: &str,
    prompt: &str,
    schema: &serde_json::Value,
    reference_image_path: &Path,
) -> Result<String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.vision_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Vision Assist model name is missing"));
    }
    if !entry.capabilities.vision_assist {
        return Err(anyhow!(
            "This provider is not marked as Vision Assist capable."
        ));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi | CloudProviderKind::OpenAiCompatible | CloudProviderKind::Gemini => {
            request_openai_vision_json(
                &client,
                entry,
                &base_url,
                api_key,
                model_name,
                prompt,
                schema,
                reference_image_path,
            )
            .await
        }
        CloudProviderKind::Anthropic => {
            request_anthropic_vision_json(
                &client,
                &base_url,
                api_key,
                model_name,
                prompt,
                schema,
                reference_image_path,
            )
            .await
        }
        CloudProviderKind::XAiGrok => Err(anyhow!(
            "Vision Assist is not wired for xAI Grok in Chatty-art yet."
        )),
        CloudProviderKind::DeepSeek => Err(anyhow!(
            "Vision Assist is not wired for DeepSeek in Chatty-art yet."
        )),
    }
}

pub async fn verify_media_generation(entry: &CloudProviderEntry, api_key: &str) -> Result<String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let image_ready =
        entry.capabilities.image_generation && !entry.image_generation_model_name.trim().is_empty();
    let video_ready =
        entry.capabilities.video_generation && !entry.video_generation_model_name.trim().is_empty();
    let audio_ready =
        entry.capabilities.audio_generation && !entry.audio_generation_model_name.trim().is_empty();
    if !image_ready && !video_ready && !audio_ready {
        return Err(anyhow!(
            "This provider does not have a configured cloud image, video, or audio generation model."
        ));
    }
    match entry.provider_kind {
        CloudProviderKind::OpenAi => verify_openai_media_generation(entry, api_key).await,
        CloudProviderKind::OpenAiCompatible => Err(anyhow!(
            "Cloud media generation is not enabled for this generic OpenAI-compatible route yet. Right now it only covers Prompt Assist and Vision Assist. Use the dedicated OpenAI family route for the current OpenAI media adapter, or wait until a specific compatible-media adapter is wired."
        )),
        CloudProviderKind::Gemini => verify_gemini_media_generation(entry, api_key).await,
        CloudProviderKind::Anthropic => Err(anyhow!(
            "Cloud media generation is not wired for Anthropic routes yet. Anthropic is currently assist-only in Chatty-art, so this saved account or route can still serve Prompt Assist or Vision Assist, but this lane will not verify or generate until a first-party Anthropic media adapter exists."
        )),
        CloudProviderKind::XAiGrok => Err(anyhow!(
            "xAI Grok is currently Prompt Assist-only in Chatty-art. This saved account or route can help with prompt expansion, but cloud media generation is not wired yet."
        )),
        CloudProviderKind::DeepSeek => Err(anyhow!(
            "DeepSeek is currently Prompt Assist-only in Chatty-art. This saved account or route can help with prompt expansion, but cloud media generation is not wired yet."
        )),
    }
}

pub async fn generate_cloud_image(
    entry: &CloudProviderEntry,
    api_key: &str,
    prompt: &str,
    size: &str,
) -> Result<CloudImageGenerationResult> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.image_generation_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Cloud image generation model name is missing"));
    }
    if !entry.capabilities.image_generation {
        return Err(anyhow!(
            "This provider is not marked as cloud image generation capable."
        ));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi => {
            request_openai_image_generation(&client, &base_url, api_key, model_name, prompt, size)
                .await
        }
        CloudProviderKind::OpenAiCompatible => Err(anyhow!(
            "Cloud image generation is not enabled for this generic OpenAI-compatible route yet. Right now it only covers Prompt Assist and Vision Assist."
        )),
        CloudProviderKind::Gemini => {
            request_gemini_image_generation(&client, &base_url, api_key, model_name, prompt, size)
                .await
        }
        CloudProviderKind::Anthropic => Err(anyhow!(
            "Cloud image generation is not wired for Anthropic routes yet. Anthropic is currently assist-only in Chatty-art."
        )),
        CloudProviderKind::XAiGrok => Err(anyhow!(
            "Cloud image generation is not wired for xAI Grok in Chatty-art yet."
        )),
        CloudProviderKind::DeepSeek => Err(anyhow!(
            "Cloud image generation is not wired for DeepSeek in Chatty-art yet."
        )),
    }
}

pub async fn generate_cloud_speech(
    entry: &CloudProviderEntry,
    api_key: &str,
    input: &str,
    instructions: Option<&str>,
) -> Result<CloudSpeechGenerationResult> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.audio_generation_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Cloud audio generation model name is missing"));
    }
    if !entry.capabilities.audio_generation {
        return Err(anyhow!(
            "This provider is not marked as cloud audio generation capable."
        ));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi => {
            request_openai_speech_generation(
                &client,
                &base_url,
                api_key,
                model_name,
                entry.audio_generation_voice.trim(),
                input,
                instructions,
            )
            .await
        }
        CloudProviderKind::OpenAiCompatible => Err(anyhow!(
            "Cloud audio generation is not enabled for this generic OpenAI-compatible route yet. Right now it only covers Prompt Assist and Vision Assist."
        )),
        CloudProviderKind::Gemini => {
            request_gemini_speech_generation(
                &client,
                &base_url,
                api_key,
                model_name,
                entry.audio_generation_voice.trim(),
                input,
                instructions,
            )
            .await
        }
        CloudProviderKind::Anthropic => Err(anyhow!(
            "Cloud audio generation is not wired for Anthropic routes yet. Anthropic is currently assist-only in Chatty-art."
        )),
        CloudProviderKind::XAiGrok => Err(anyhow!(
            "Cloud audio generation is not wired for xAI Grok in Chatty-art yet."
        )),
        CloudProviderKind::DeepSeek => Err(anyhow!(
            "Cloud audio generation is not wired for DeepSeek in Chatty-art yet."
        )),
    }
}

pub async fn generate_cloud_video(
    entry: &CloudProviderEntry,
    api_key: &str,
    prompt: &str,
    size: &str,
    seconds: u32,
    reference_image_path: Option<&Path>,
) -> Result<CloudVideoGenerationResult> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("API key is missing"));
    }
    let model_name = entry.video_generation_model_name.trim();
    if model_name.is_empty() {
        return Err(anyhow!("Cloud video generation model name is missing"));
    }
    if !entry.capabilities.video_generation {
        return Err(anyhow!(
            "This provider is not marked as cloud video generation capable."
        ));
    }
    let trimmed_prompt = prompt.trim();
    if trimmed_prompt.is_empty() {
        return Err(anyhow!("Cloud video generation needs a prompt."));
    }
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;
    match entry.provider_kind {
        CloudProviderKind::OpenAi => {
            request_openai_video_generation(
                &client,
                &base_url,
                api_key,
                model_name,
                trimmed_prompt,
                size,
                seconds,
                reference_image_path,
            )
            .await
        }
        CloudProviderKind::OpenAiCompatible => Err(anyhow!(
            "Cloud video generation is not enabled for this generic OpenAI-compatible route yet. Right now it only covers Prompt Assist and Vision Assist."
        )),
        CloudProviderKind::Gemini => {
            request_gemini_video_generation(
                &client,
                &base_url,
                api_key,
                model_name,
                trimmed_prompt,
                size,
                seconds,
                reference_image_path,
            )
            .await
        }
        CloudProviderKind::Anthropic => Err(anyhow!(
            "Cloud video generation is not wired for Anthropic routes yet. Anthropic is currently assist-only in Chatty-art."
        )),
        CloudProviderKind::XAiGrok => Err(anyhow!(
            "Cloud video generation is not wired for xAI Grok in Chatty-art yet."
        )),
        CloudProviderKind::DeepSeek => Err(anyhow!(
            "Cloud video generation is not wired for DeepSeek in Chatty-art yet."
        )),
    }
}

pub struct CloudImageGenerationResult {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub revised_prompt: Option<String>,
}

pub struct CloudSpeechGenerationResult {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub voice: String,
}

pub struct CloudVideoGenerationResult {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub model: String,
    pub size: String,
    pub seconds: u32,
}

async fn verify_openai_media_generation(
    entry: &CloudProviderEntry,
    api_key: &str,
) -> Result<String> {
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;
    let mut notes = Vec::new();

    if entry.capabilities.image_generation && !entry.image_generation_model_name.trim().is_empty() {
        request_openai_image_generation(
            &client,
            &base_url,
            api_key,
            entry.image_generation_model_name.trim(),
            "flat neutral gray square, no text, no objects",
            "256x256",
        )
        .await?;
        notes.push(format!(
            "image ok ({})",
            entry.image_generation_model_name.trim()
        ));
    }

    if entry.capabilities.audio_generation && !entry.audio_generation_model_name.trim().is_empty() {
        let voice_name = entry.audio_generation_voice.trim();
        let voice_label = if voice_name.is_empty() {
            "default voice"
        } else {
            voice_name
        };
        request_openai_speech_generation(
            &client,
            &base_url,
            api_key,
            entry.audio_generation_model_name.trim(),
            voice_name,
            "verification",
            Some("Short neutral verification clip."),
        )
        .await?;
        notes.push(format!(
            "speech ok ({} via {})",
            entry.audio_generation_model_name.trim(),
            voice_label
        ));
    }

    if entry.capabilities.video_generation && !entry.video_generation_model_name.trim().is_empty() {
        request_openai_video_generation(
            &client,
            &base_url,
            api_key,
            entry.video_generation_model_name.trim(),
            "single calm abstract gradient, no text, no people",
            "1280x720",
            4,
            None,
        )
        .await?;
        notes.push(format!(
            "video ok ({} via the current deprecated OpenAI Videos API path)",
            entry.video_generation_model_name.trim()
        ));
    }

    if notes.is_empty() {
        return Err(anyhow!(
            "This provider does not have a configured cloud image, video, or audio generation model."
        ));
    }

    Ok(format!(
        "Live media verification passed for {}: {}.",
        entry.display_name.trim(),
        notes.join("; ")
    ))
}

async fn verify_gemini_media_generation(
    entry: &CloudProviderEntry,
    api_key: &str,
) -> Result<String> {
    let base_url = effective_base_url(entry);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;
    let mut notes = Vec::new();

    if entry.capabilities.image_generation && !entry.image_generation_model_name.trim().is_empty() {
        request_gemini_image_generation(
            &client,
            &base_url,
            api_key,
            entry.image_generation_model_name.trim(),
            "flat neutral gray square, no text, no objects",
            "256x256",
        )
        .await?;
        notes.push(format!(
            "image ok ({})",
            entry.image_generation_model_name.trim()
        ));
    }

    if entry.capabilities.video_generation && !entry.video_generation_model_name.trim().is_empty() {
        request_gemini_video_generation(
            &client,
            &base_url,
            api_key,
            entry.video_generation_model_name.trim(),
            "single calm abstract gradient, no text, no people",
            "1280x720",
            4,
            None,
        )
        .await?;
        notes.push(format!(
            "video ok ({})",
            entry.video_generation_model_name.trim()
        ));
    }

    if notes.is_empty() {
        return Err(anyhow!(
            "This provider does not have a configured cloud image or video generation model."
        ));
    }

    Ok(format!(
        "Live Gemini media verification passed for {}: {}.",
        entry.display_name.trim(),
        notes.join("; ")
    ))
}

pub fn validate_openai_video_request(model_name: &str, size: &str, seconds: u32) -> Result<()> {
    let normalized_model = model_name.trim().to_ascii_lowercase();
    let normalized_size = size.trim();
    let is_pro = normalized_model.contains("sora-2-pro");
    let size_ok = if is_pro {
        matches!(
            normalized_size,
            "1280x720" | "720x1280" | "1792x1024" | "1024x1792" | "1920x1080" | "1080x1920"
        )
    } else {
        matches!(
            normalized_size,
            "1280x720" | "720x1280" | "1792x1024" | "1024x1792"
        )
    };
    if !size_ok {
        if is_pro {
            return Err(anyhow!(
                "OpenAI cloud video currently expects one of these sizes for sora-2-pro: 1280x720, 720x1280, 1792x1024, 1024x1792, 1920x1080, or 1080x1920."
            ));
        }
        return Err(anyhow!(
            "OpenAI cloud video currently expects one of these sizes for sora-2: 1280x720, 720x1280, 1792x1024, or 1024x1792. Use sora-2-pro if you want 1920x1080 or 1080x1920."
        ));
    }
    if !matches!(seconds, 4 | 8 | 12 | 16 | 20) {
        return Err(anyhow!(
            "OpenAI cloud video currently expects a duration of 4, 8, 12, 16, or 20 seconds."
        ));
    }
    Ok(())
}

fn effective_base_url(entry: &CloudProviderEntry) -> String {
    let trimmed = entry.base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        default_base_url(entry.provider_kind).to_string()
    } else {
        trimmed.to_string()
    }
}

async fn request_openai_image_generation(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    size: &str,
) -> Result<CloudImageGenerationResult> {
    let url = format!("{base_url}/images/generations");
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .json(&json!({
            "model": model_name,
            "prompt": prompt,
            "size": size,
            "output_format": "png",
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    parse_openai_image_generation_response(client, &body).await
}

async fn request_gemini_image_generation(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    size: &str,
) -> Result<CloudImageGenerationResult> {
    let url = format!("{base_url}/images/generations");
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .json(&json!({
            "model": model_name,
            "prompt": prompt,
            "size": size,
            "response_format": "b64_json",
            "n": 1,
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    parse_openai_image_generation_response(client, &body).await
}

async fn parse_openai_image_generation_response(
    client: &reqwest::Client,
    body: &str,
) -> Result<CloudImageGenerationResult> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| anyhow!("Provider returned invalid JSON while reading image generation output"))?;
    let image = value
        .get("data")
        .and_then(|data| data.as_array())
        .and_then(|data| data.first())
        .ok_or_else(|| anyhow!("Provider returned an unexpected image generation response shape"))?;
    let revised_prompt = image
        .get("revised_prompt")
        .and_then(|value| value.as_str())
        .map(str::to_string);

    if let Some(b64_json) = image.get("b64_json").and_then(|value| value.as_str()) {
        let bytes = BASE64_STANDARD
            .decode(b64_json)
            .map_err(|_| anyhow!("Provider returned invalid base64 image data"))?;
        return Ok(CloudImageGenerationResult {
            bytes,
            mime: "image/png".to_string(),
            revised_prompt,
        });
    }

    if let Some(url) = image.get("url").and_then(|value| value.as_str()) {
        let download = client.get(url).send().await?;
        let status = download.status();
        let mime = download
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("image/png")
            .to_string();
        let bytes = download.bytes().await?.to_vec();
        if !status.is_success() {
            return Err(anyhow!(
                "Cloud image generation returned a download URL, but fetching it failed ({status})."
            ));
        }
        return Ok(CloudImageGenerationResult {
            bytes,
            mime,
            revised_prompt,
        });
    }

    Err(anyhow!(
        "Provider did not return image bytes or a downloadable image URL."
    ))
}

async fn request_openai_speech_generation(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    voice_name: &str,
    input: &str,
    instructions: Option<&str>,
) -> Result<CloudSpeechGenerationResult> {
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err(anyhow!("Cloud audio generation needs spoken text input."));
    }
    let voice = if voice_name.trim().is_empty() {
        "marin"
    } else {
        voice_name.trim()
    };
    let url = format!("{base_url}/audio/speech");
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "audio/wav")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .json(&json!({
            "model": model_name,
            "voice": voice,
            "input": trimmed_input,
            "instructions": instructions.unwrap_or("").trim(),
            "response_format": "wav",
        }))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("audio/wav")
        .to_string();
    let bytes = response.bytes().await?.to_vec();
    Ok(CloudSpeechGenerationResult {
        bytes,
        mime,
        voice: voice.to_string(),
    })
}

async fn request_gemini_speech_generation(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    voice_name: &str,
    input: &str,
    instructions: Option<&str>,
) -> Result<CloudSpeechGenerationResult> {
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err(anyhow!("Cloud audio generation needs spoken text input."));
    }
    let voice = gemini_voice_name(voice_name);
    let prompt = build_gemini_speech_prompt(trimmed_input, instructions);
    let native_base_url = gemini_native_base_url(base_url);
    let url = format!("{native_base_url}/models/{model_name}:generateContent");
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("x-goog-api-key", api_key)
        .json(&json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": voice
                        }
                    }
                }
            },
            "model": model_name,
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    let pcm = extract_gemini_tts_pcm(&body)?;
    let bytes = pcm_to_wav(&pcm, 24_000, 1, 16);
    Ok(CloudSpeechGenerationResult {
        bytes,
        mime: "audio/wav".to_string(),
        voice: voice.to_string(),
    })
}

async fn request_openai_video_generation(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    size: &str,
    seconds: u32,
    reference_image_path: Option<&Path>,
) -> Result<CloudVideoGenerationResult> {
    validate_openai_video_request(model_name, size, seconds)?;
    let create_url = format!("{base_url}/videos");
    let mut form = Form::new()
        .text("model", model_name.to_string())
        .text("prompt", prompt.to_string())
        .text("size", size.to_string())
        .text("seconds", seconds.to_string());

    if let Some(reference_image_path) = reference_image_path {
        let mime = image_mime_type(reference_image_path)?;
        let file_name = reference_image_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("reference.png")
            .to_string();
        let bytes = std::fs::read(reference_image_path)
            .map_err(|error| anyhow!("Could not read reference image: {error}"))?;
        let part = Part::bytes(bytes)
            .file_name(file_name)
            .mime_str(mime)
            .map_err(|error| anyhow!("Could not attach the reference image: {error}"))?;
        form = form.part("input_reference", part);
    }

    let response = client
        .post(create_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }

    let started = parse_openai_video_job(&body)?;
    let final_job = poll_openai_video_job(client, base_url, api_key, &started.id).await?;
    if final_job.status == "failed" {
        let reason = final_job
            .error_message
            .unwrap_or_else(|| "Video generation failed on the provider.".to_string());
        return Err(anyhow!(reason));
    }
    if final_job.status != "completed" {
        return Err(anyhow!(
            "Video generation ended in unexpected provider status '{}'.",
            final_job.status
        ));
    }

    let content_url = format!("{base_url}/videos/{}/content", final_job.id);
    let download = client
        .get(content_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .send()
        .await?;
    let status = download.status();
    if !status.is_success() {
        let body = download.text().await.unwrap_or_default();
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    let mime = download
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("video/mp4")
        .to_string();
    let bytes = download.bytes().await?.to_vec();
    Ok(CloudVideoGenerationResult {
        bytes,
        mime,
        model: final_job.model.unwrap_or_else(|| model_name.to_string()),
        size: final_job.size.unwrap_or_else(|| size.to_string()),
        seconds: final_job.seconds.unwrap_or(seconds),
    })
}

async fn request_gemini_video_generation(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    size: &str,
    seconds: u32,
    reference_image_path: Option<&Path>,
) -> Result<CloudVideoGenerationResult> {
    let (aspect_ratio, resolution) = gemini_video_shape(size)?;
    let duration_seconds = gemini_video_duration(seconds, reference_image_path.is_some())?;
    let create_url = format!("{base_url}/videos");
    let mut extra_body = json!({
        "aspect_ratio": aspect_ratio,
        "resolution": resolution,
        "duration_seconds": duration_seconds,
    });
    if let Some(reference_image_path) = reference_image_path {
        let bytes = std::fs::read(reference_image_path)
            .map_err(|error| anyhow!("Could not read reference image: {error}"))?;
        let encoded = BASE64_STANDARD.encode(bytes);
        extra_body["image"] = serde_json::Value::String(encoded);
    }

    let response = client
        .post(create_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(&json!({
            "model": model_name,
            "prompt": prompt,
            "extra_body": extra_body,
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }

    let started = parse_openai_video_job(&body)?;
    let final_job = poll_openai_video_job(client, base_url, api_key, &started.id).await?;
    if final_job.status == "failed" {
        let reason = final_job
            .error_message
            .unwrap_or_else(|| "Video generation failed on the provider.".to_string());
        return Err(anyhow!(reason));
    }
    if final_job.status != "completed" {
        return Err(anyhow!(
            "Video generation ended in unexpected provider status '{}'.",
            final_job.status
        ));
    }

    let content_url = format!("{base_url}/videos/{}/content", final_job.id);
    let download = client
        .get(content_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .send()
        .await?;
    let status = download.status();
    if !status.is_success() {
        let body = download.text().await.unwrap_or_default();
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    let mime = download
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("video/mp4")
        .to_string();
    let bytes = download.bytes().await?.to_vec();
    Ok(CloudVideoGenerationResult {
        bytes,
        mime,
        model: final_job.model.unwrap_or_else(|| model_name.to_string()),
        size: final_job.size.unwrap_or_else(|| size.to_string()),
        seconds: final_job.seconds.unwrap_or(duration_seconds),
    })
}

async fn poll_openai_video_job(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    video_id: &str,
) -> Result<OpenAiVideoJob> {
    let url = format!("{base_url}/videos/{video_id}");
    let mut delay = Duration::from_secs(4);
    for _ in 0..120 {
        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {api_key}"))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
        }
        let job = parse_openai_video_job(&body)?;
        if matches!(job.status.as_str(), "completed" | "failed" | "cancelled" | "canceled") {
            return Ok(job);
        }
        tokio::time::sleep(delay).await;
        delay = (delay + Duration::from_secs(2)).min(Duration::from_secs(12));
    }
    Err(anyhow!(
        "Timed out while waiting for the cloud video job to finish."
    ))
}

fn parse_openai_video_job(body: &str) -> Result<OpenAiVideoJob> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| anyhow!("Provider returned invalid JSON while reading video generation output"))?;
    let id = value
        .get("id")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Provider returned an unexpected video generation response shape"))?;
    let status = value
        .get("status")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Provider returned an unexpected video generation response shape"))?;
    let model = value.get("model").and_then(|value| value.as_str()).map(str::to_string);
    let size = value.get("size").and_then(|value| value.as_str()).map(str::to_string);
    let seconds = value
        .get("seconds")
        .and_then(|value| value.as_str().and_then(|raw| raw.parse::<u32>().ok()).or_else(|| value.as_u64().map(|raw| raw as u32)));
    let error_message = value.get("error").and_then(parse_error_value);
    Ok(OpenAiVideoJob {
        id,
        status,
        model,
        size,
        seconds,
        error_message,
    })
}

fn gemini_video_shape(size: &str) -> Result<(&'static str, &'static str)> {
    match size.trim() {
        "1280x720" => Ok(("16:9", "720p")),
        "720x1280" => Ok(("9:16", "720p")),
        "1792x1024" | "1920x1080" => Ok(("16:9", "1080p")),
        "1024x1792" | "1080x1920" => Ok(("9:16", "1080p")),
        other => Err(anyhow!(
            "Gemini cloud video currently expects one of these sizes: 1280x720, 720x1280, 1792x1024, 1024x1792, 1920x1080, or 1080x1920. Received '{other}'."
        )),
    }
}

fn gemini_video_duration(seconds: u32, uses_reference_image: bool) -> Result<u32> {
    let clamped = seconds.clamp(4, 8);
    let normalized = if clamped <= 4 {
        4
    } else if clamped <= 6 {
        6
    } else {
        8
    };
    if uses_reference_image && normalized != 8 {
        return Err(anyhow!(
            "Gemini cloud video requires an 8-second duration when using a still-image reference."
        ));
    }
    Ok(normalized)
}

fn parse_error_value(value: &serde_json::Value) -> Option<String> {
    value
        .get("message")
        .and_then(|message| message.as_str())
        .or_else(|| value.as_str())
        .map(str::to_string)
}

struct OpenAiVideoJob {
    id: String,
    status: String,
    model: Option<String>,
    size: Option<String>,
    seconds: Option<u32>,
    error_message: Option<String>,
}

async fn verify_openai_chat_like(
    client: &reqwest::Client,
    entry: &CloudProviderEntry,
    base_url: &str,
    api_key: &str,
    model_name: &str,
) -> Result<String> {
    verify_openai_chat_like_with_note(
        client,
        entry,
        base_url,
        api_key,
        model_name,
        "Reply with OK.",
        "Prompt Assist",
    )
    .await
}

async fn verify_openai_chat_like_with_note(
    client: &reqwest::Client,
    entry: &CloudProviderEntry,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    lane_label: &str,
) -> Result<String> {
    let url = format!("{base_url}/chat/completions");
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let auth_value = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .map_err(|_| anyhow!("API key contains invalid header characters"))?;
    headers.insert(AUTHORIZATION, auth_value);
    let response = client
        .post(url)
        .headers(headers)
        .json(&json!({
            "model": model_name,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 8,
            "temperature": 0
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    Ok(format!(
        "Health check passed for {lane_label} via {}.",
        entry.display_name.trim(),
    ))
}

async fn request_openai_chat_like_json(
    client: &reqwest::Client,
    _entry: &CloudProviderEntry,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    schema: &serde_json::Value,
    max_tokens: usize,
) -> Result<String> {
    let url = format!("{base_url}/chat/completions");
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let auth_value = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .map_err(|_| anyhow!("API key contains invalid header characters"))?;
    headers.insert(AUTHORIZATION, auth_value);
    let system = format!(
        "You are Prompt Assist for a local creative tool. Return only JSON that matches this schema exactly. Do not wrap it in markdown.\nSchema:\n{}",
        schema
    );
    let response = client
        .post(url)
        .headers(headers)
        .json(&json!({
            "model": model_name,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": max_tokens.max(64),
            "temperature": 0.2
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    extract_openai_chat_text(&body)
}

async fn verify_anthropic(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
) -> Result<String> {
    let url = format!("{base_url}/messages");
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": model_name,
            "max_tokens": 8,
            "temperature": 0,
            "messages": [
                {"role": "user", "content": "Reply with OK."}
            ]
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    Ok("Health check passed for Prompt Assist via Anthropic.".to_string())
}

async fn verify_anthropic_vision(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
) -> Result<String> {
    let url = format!("{base_url}/messages");
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": model_name,
            "max_tokens": 8,
            "temperature": 0,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/png",
                                "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC"
                            }
                        },
                        {
                            "type": "text",
                            "text": "Reply with OK."
                        }
                    ]
                }
            ]
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    Ok("Health check passed for Vision Assist via Anthropic.".to_string())
}

async fn request_anthropic_json(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    schema: &serde_json::Value,
    max_tokens: usize,
) -> Result<String> {
    let url = format!("{base_url}/messages");
    let system = format!(
        "You are Prompt Assist for a local creative tool. Return only JSON that matches this schema exactly. Do not wrap it in markdown.\nSchema:\n{}",
        schema
    );
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": model_name,
            "system": system,
            "max_tokens": max_tokens.max(64),
            "temperature": 0.2,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    extract_anthropic_text(&body)
}

async fn request_anthropic_vision_json(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    schema: &serde_json::Value,
    reference_image_path: &Path,
) -> Result<String> {
    let (media_type, image_data) = image_base64_parts(reference_image_path)?;
    let url = format!("{base_url}/messages");
    let system = format!(
        "You are Vision Assist for a local creative tool. Study the attached reference image and return only JSON that matches this schema exactly. Do not wrap it in markdown.\nSchema:\n{}",
        schema
    );
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": model_name,
            "system": system,
            "max_tokens": 320,
            "temperature": 0.1,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": media_type,
                                "data": image_data
                            }
                        },
                        {
                            "type": "text",
                            "text": prompt
                        }
                    ]
                }
            ]
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    extract_anthropic_text(&body)
}

async fn request_openai_vision_json(
    client: &reqwest::Client,
    _entry: &CloudProviderEntry,
    base_url: &str,
    api_key: &str,
    model_name: &str,
    prompt: &str,
    schema: &serde_json::Value,
    reference_image_path: &Path,
) -> Result<String> {
    let image_url = image_data_url(reference_image_path)?;
    let system = format!(
        "You are Vision Assist for a local creative tool. Study the attached reference image and return only JSON that matches this schema exactly. Do not wrap it in markdown.\nSchema:\n{}",
        schema
    );
    let url = format!("{base_url}/chat/completions");
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let auth_value = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .map_err(|_| anyhow!("API key contains invalid header characters"))?;
    headers.insert(AUTHORIZATION, auth_value);

    let response = client
        .post(url)
        .headers(headers)
        .json(&json!({
            "model": model_name,
            "messages": [
                {"role": "system", "content": system},
                {
                    "role": "user",
                    "content": [
                        {"type": "text", "text": prompt},
                        {"type": "image_url", "image_url": {"url": image_url}}
                    ]
                }
            ],
            "max_tokens": 320,
            "temperature": 0.1
        }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(extract_error_message(status.as_u16(), &body)));
    }
    extract_openai_chat_text(&body)
}

fn extract_openai_chat_text(body: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| anyhow!("Provider returned invalid JSON while reading Prompt Assist output"))?;
    let content = value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .ok_or_else(|| anyhow!("Provider returned an unexpected chat response shape"))?;
    if let Some(text) = content.as_str() {
        return Ok(text.to_string());
    }
    let text = content
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| anyhow!("Provider returned an unexpected chat response shape"))?;
    Ok(text)
}

fn extract_anthropic_text(body: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| anyhow!("Provider returned invalid JSON while reading Prompt Assist output"))?;
    let content = value
        .get("content")
        .and_then(|content| content.as_array())
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(|text| text.as_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Provider returned an unexpected Anthropic response shape"))?;
    Ok(content)
}

fn extract_error_message(status: u16, body: &str) -> String {
    if status == StatusCode::TOO_MANY_REQUESTS.as_u16() {
        return "Provider request failed (429): rate limited by the provider. Wait a moment and try again.".to_string();
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(message) = extract_structured_error_message(&value) {
            return format_structured_error_message(status, message);
        }
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        format!("Provider request failed ({status}).")
    } else {
        let summary = summarize_error_body(trimmed);
        format!("Provider request failed ({status}): {summary}")
    }
}

fn extract_structured_error_message<'a>(value: &'a serde_json::Value) -> Option<&'a str> {
    if let Some(message) = value
        .get("error")
        .and_then(|error| {
            error
                .get("message")
                .and_then(|value| value.as_str())
                .or_else(|| error.as_str())
        })
    {
        return Some(message);
    }
    if let Some(message) = value.get("message").and_then(|value| value.as_str()) {
        return Some(message);
    }
    value
        .as_array()
        .and_then(|items| items.iter().find_map(extract_structured_error_message))
}

fn format_structured_error_message(status: u16, message: &str) -> String {
    let normalized = normalize_error_whitespace(message);
    if let Some(summary) = summarize_auth_or_permission_error(status, &normalized) {
        return format!("Provider request failed ({status}): {summary}");
    }
    format!(
        "Provider request failed ({status}): {}",
        truncate_error_summary(&normalized, 240)
    )
}

fn summarize_auth_or_permission_error(status: u16, message: &str) -> Option<String> {
    let lower = message.to_ascii_lowercase();
    if status == StatusCode::UNAUTHORIZED.as_u16()
        || lower.contains("incorrect api key")
        || lower.contains("invalid api key")
        || lower.contains("valid api key")
        || lower.contains("unauthorized")
        || lower.contains("authentication")
    {
        return Some(
            "authentication failed. Check the saved API key and base URL for this route."
                .to_string(),
        );
    }
    if status == StatusCode::FORBIDDEN.as_u16()
        || lower.contains("forbidden")
        || lower.contains("permission")
        || lower.contains("not allowed")
        || lower.contains("access denied")
    {
        return Some(
            "permission denied by the provider. Check the saved API key, account/project access, and model entitlement for this route."
                .to_string(),
        );
    }
    None
}

fn summarize_error_body(body: &str) -> String {
    if looks_like_html_error(body) {
        let title = extract_html_title(body)
            .or_else(|| first_nonempty_line(&strip_html_tags(body)))
            .unwrap_or_else(|| "HTML error page".to_string());
        return format!(
            "{}. Verify the base URL points at the API endpoint, not a website.",
            truncate_error_summary(&normalize_error_whitespace(&title), 120)
        );
    }
    truncate_error_summary(&normalize_error_whitespace(body), 240)
}

fn looks_like_html_error(body: &str) -> bool {
    let lower = body.trim().to_ascii_lowercase();
    lower.starts_with("<!doctype html")
        || lower.starts_with("<html")
        || lower.contains("<head")
        || lower.contains("<body")
        || lower.contains("<title")
}

fn extract_html_title(body: &str) -> Option<String> {
    let lower = body.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after_open = lower[start..].find('>')? + start + 1;
    let end = lower[after_open..].find("</title>")? + after_open;
    let title = body[after_open..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn strip_html_tags(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let mut in_tag = false;
    for ch in body.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

fn first_nonempty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn normalize_error_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_error_summary(text: &str, limit: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= limit {
        return text.to_string();
    }
    let clipped = text.chars().take(limit).collect::<String>();
    format!("{}...", clipped.trim_end())
}

fn image_data_url(path: &Path) -> Result<String> {
    let mime = image_mime_type(path)?;
    let encoded = read_image_base64(path)?;
    Ok(format!("data:{mime};base64,{encoded}"))
}

fn gemini_native_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    if let Some(stripped) = trimmed.strip_suffix("/openai") {
        stripped.to_string()
    } else {
        trimmed.to_string()
    }
}

fn gemini_voice_name(voice_name: &str) -> &str {
    let trimmed = voice_name.trim();
    if trimmed.is_empty() {
        "Kore"
    } else {
        trimmed
    }
}

fn build_gemini_speech_prompt(input: &str, instructions: Option<&str>) -> String {
    let direction = instructions.unwrap_or("").trim();
    if direction.is_empty() {
        format!(
            "Read the following text exactly as written.\n\nSpoken text:\n{}",
            input
        )
    } else {
        format!(
            "Read the following text exactly as written.\nUse these delivery directions but do not change the words.\n\nDelivery directions:\n{}\n\nSpoken text:\n{}",
            direction,
            input
        )
    }
}

fn extract_gemini_tts_pcm(body: &str) -> Result<Vec<u8>> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| anyhow!("Provider returned invalid JSON while reading Gemini speech output"))?;
    let audio_b64 = value
        .get("candidates")
        .and_then(|candidates| candidates.as_array())
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.as_array())
        .and_then(|parts| {
            parts.iter().find_map(|part| {
                part.get("inlineData")
                    .or_else(|| part.get("inline_data"))
                    .and_then(|inline| inline.get("data"))
                    .and_then(|data| data.as_str())
            })
        })
        .ok_or_else(|| anyhow!("Provider returned an unexpected Gemini speech response shape"))?;
    BASE64_STANDARD
        .decode(audio_b64)
        .map_err(|_| anyhow!("Provider returned invalid base64 audio for Gemini speech output"))
}

fn pcm_to_wav(pcm: &[u8], sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8> {
    let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample) / 8;
    let block_align = channels * (bits_per_sample / 8);
    let data_len = pcm.len() as u32;
    let riff_len = 36 + data_len;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

fn image_base64_parts(path: &Path) -> Result<(&'static str, String)> {
    let mime = image_mime_type(path)?;
    let encoded = read_image_base64(path)?;
    Ok((mime, encoded))
}

fn read_image_base64(path: &Path) -> Result<String> {
    let bytes =
        std::fs::read(path).map_err(|error| anyhow!("Could not read reference image: {error}"))?;
    Ok(BASE64_STANDARD.encode(bytes))
}

fn image_mime_type(path: &Path) -> Result<&'static str> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| anyhow!("Reference image is missing a file extension"))?;
    match extension.as_str() {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "webp" => Ok("image/webp"),
        "gif" => Ok("image/gif"),
        other => Err(anyhow!(
            "Unsupported reference image format for cloud Vision Assist: {other}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_gemini_speech_prompt, default_capabilities, extract_error_message,
        extract_gemini_tts_pcm, gemini_native_base_url, generate_cloud_image,
        generate_cloud_speech, generate_prompt_assist_json, generate_vision_assist_json,
        pcm_to_wav, validate_openai_video_request, verify_media_generation,
        verify_prompt_assist, verify_vision_assist,
    };
    use crate::types::{CloudProviderEntry, CloudProviderKind};
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
    use serde_json::json;
    use std::{fs, path::PathBuf};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        sync::{mpsc, oneshot},
    };

    async fn spawn_single_request_server(
        response_body: &'static str,
    ) -> (String, oneshot::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, request_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = Vec::new();
            let mut header_end = None;
            let mut content_length = 0usize;
            loop {
                let mut chunk = [0u8; 1024];
                let read = stream.read(&mut chunk).await.unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if header_end.is_none() {
                    if let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                        let body_start = end + 4;
                        let headers = String::from_utf8_lossy(&buffer[..end]);
                        for line in headers.lines() {
                            let lower = line.to_ascii_lowercase();
                            if let Some(length) = lower.strip_prefix("content-length:") {
                                content_length = length.trim().parse::<usize>().unwrap();
                            }
                        }
                        header_end = Some(body_start);
                    }
                }
                if let Some(body_start) = header_end {
                    if buffer.len() >= body_start + content_length {
                        break;
                    }
                }
            }
            let request = String::from_utf8(buffer).unwrap();
            let _ = request_tx.send(request);
            let reply = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(reply.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        });
        (format!("http://{address}"), request_rx)
    }

    async fn spawn_sequence_server(
        responses: Vec<(&'static str, &'static str, &'static str)>,
    ) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, request_rx) = mpsc::channel(8);
        tokio::spawn(async move {
            for (status_line, content_type, response_body) in responses {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = Vec::new();
                let mut header_end = None;
                let mut content_length = 0usize;
                loop {
                    let mut chunk = [0u8; 1024];
                    let read = stream.read(&mut chunk).await.unwrap();
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                    if header_end.is_none() {
                        if let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                            let body_start = end + 4;
                            let headers = String::from_utf8_lossy(&buffer[..end]);
                            for line in headers.lines() {
                                let lower = line.to_ascii_lowercase();
                                if let Some(length) = lower.strip_prefix("content-length:") {
                                    content_length = length.trim().parse::<usize>().unwrap();
                                }
                            }
                            header_end = Some(body_start);
                        }
                    }
                    if let Some(body_start) = header_end {
                        if buffer.len() >= body_start + content_length {
                            break;
                        }
                    }
                }
                let request = String::from_utf8(buffer).unwrap();
                request_tx.send(request).await.unwrap();
                let reply = format!(
                    "HTTP/1.1 {status_line}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                stream.write_all(reply.as_bytes()).await.unwrap();
                stream.shutdown().await.unwrap();
            }
        });
        (format!("http://{address}"), request_rx)
    }

    fn write_test_png(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("chatty-art-{name}-{}.png", std::process::id()));
        let png = BASE64_STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC")
            .unwrap();
        fs::write(&path, png).unwrap();
        path
    }

    #[test]
    fn openai_video_validation_accepts_sora2_standard_shapes_and_durations() {
        validate_openai_video_request("sora-2", "1280x720", 4).unwrap();
        validate_openai_video_request("sora-2", "1792x1024", 20).unwrap();
    }

    #[test]
    fn openai_video_validation_rejects_1080p_on_non_pro_model() {
        let error = validate_openai_video_request("sora-2", "1920x1080", 8).unwrap_err();
        assert!(error
            .to_string()
            .contains("Use sora-2-pro if you want 1920x1080 or 1080x1920"));
    }

    #[test]
    fn openai_video_validation_accepts_1080p_on_pro_model() {
        validate_openai_video_request("sora-2-pro", "1920x1080", 8).unwrap();
        validate_openai_video_request("sora-2-pro", "1080x1920", 16).unwrap();
    }

    #[test]
    fn openai_video_validation_rejects_unsupported_duration_bucket() {
        let error = validate_openai_video_request("sora-2-pro", "1280x720", 6).unwrap_err();
        assert!(error
            .to_string()
            .contains("duration of 4, 8, 12, 16, or 20 seconds"));
    }

    #[test]
    fn gemini_native_base_url_strips_openai_suffix_only() {
        assert_eq!(
            gemini_native_base_url("https://generativelanguage.googleapis.com/v1beta/openai"),
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(
            gemini_native_base_url("https://generativelanguage.googleapis.com/v1beta/"),
            "https://generativelanguage.googleapis.com/v1beta"
        );
    }

    #[test]
    fn gemini_speech_prompt_keeps_words_and_separates_directions() {
        let prompt = build_gemini_speech_prompt(
            "Hello there, traveler.",
            Some("Warm, calm, slightly smiling narration."),
        );
        assert!(prompt.contains("Read the following text exactly as written."));
        assert!(prompt.contains("Delivery directions:\nWarm, calm, slightly smiling narration."));
        assert!(prompt.contains("Spoken text:\nHello there, traveler."));
    }

    #[test]
    fn gemini_tts_pcm_parser_accepts_inline_data_shape() {
        let raw = [1u8, 2, 3, 4];
        let encoded = BASE64_STANDARD.encode(raw);
        let body = format!(
            r#"{{"candidates":[{{"content":{{"parts":[{{"inlineData":{{"data":"{encoded}"}}}}]}}}}]}}"#
        );
        let parsed = extract_gemini_tts_pcm(&body).unwrap();
        assert_eq!(parsed, raw);
    }

    #[test]
    fn gemini_tts_pcm_parser_accepts_inline_data_snake_case_shape() {
        let raw = [9u8, 8, 7, 6];
        let encoded = BASE64_STANDARD.encode(raw);
        let body = format!(
            r#"{{"candidates":[{{"content":{{"parts":[{{"inline_data":{{"data":"{encoded}"}}}}]}}}}]}}"#
        );
        let parsed = extract_gemini_tts_pcm(&body).unwrap();
        assert_eq!(parsed, raw);
    }

    #[test]
    fn pcm_to_wav_wraps_pcm_with_expected_header() {
        let pcm = [0u8, 1, 2, 3];
        let wav = pcm_to_wav(&pcm, 24_000, 1, 16);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(&wav[40..44], &(pcm.len() as u32).to_le_bytes());
        assert_eq!(&wav[44..], &pcm);
    }

    #[test]
    fn extract_error_message_summarizes_html_error_pages() {
        let body = r#"<!doctype html><html lang="en"><head><title>Example Domain</title></head><body><h1>Example Domain</h1></body></html>"#;
        let message = extract_error_message(405, body);
        assert!(message.contains("Provider request failed (405): Example Domain."));
        assert!(message.contains("Verify the base URL points at the API endpoint, not a website."));
        assert!(!message.contains("<html"));
    }

    #[test]
    fn extract_error_message_normalizes_and_truncates_plain_text() {
        let body = "bad endpoint\n\nreturned way too much text      with odd spacing ".repeat(20);
        let message = extract_error_message(502, &body);
        assert!(message.starts_with("Provider request failed (502): bad endpoint returned way too much text with odd spacing"));
        assert!(message.ends_with("..."));
        assert!(!message.contains("\n"));
    }

    #[test]
    fn extract_error_message_summarizes_unauthorized_key_errors() {
        let body = r#"{"error":{"message":"Incorrect API key provided: fake-key. You can find your API key at https://platform.openai.com/account/api-keys."}}"#;
        let message = extract_error_message(401, body);
        assert_eq!(
            message,
            "Provider request failed (401): authentication failed. Check the saved API key and base URL for this route."
        );
        assert!(!message.contains("platform.openai.com"));
    }

    #[test]
    fn extract_error_message_summarizes_forbidden_permission_errors() {
        let body = r#"{"error":{"message":"Forbidden: this project does not have access to that model."}}"#;
        let message = extract_error_message(403, body);
        assert_eq!(
            message,
            "Provider request failed (403): permission denied by the provider. Check the saved API key, account/project access, and model entitlement for this route."
        );
    }

    #[test]
    fn extract_error_message_summarizes_array_wrapped_auth_errors() {
        let body = r#"[{ "error": { "code": 400, "message": "Please pass a valid API key", "status": "INVALID_ARGUMENT" } }]"#;
        let message = extract_error_message(400, body);
        assert_eq!(
            message,
            "Provider request failed (400): authentication failed. Check the saved API key and base URL for this route."
        );
    }

    #[test]
    fn anthropic_capabilities_stay_assist_only_with_vision_enabled() {
        let caps = default_capabilities(CloudProviderKind::Anthropic);
        assert!(caps.text_assist);
        assert!(caps.vision_assist);
        assert!(!caps.image_generation);
        assert!(!caps.video_generation);
        assert!(!caps.audio_generation);
    }

    #[test]
    fn xai_grok_capabilities_stay_prompt_assist_only() {
        let caps = default_capabilities(CloudProviderKind::XAiGrok);
        assert!(caps.text_assist);
        assert!(!caps.vision_assist);
        assert!(!caps.image_generation);
        assert!(!caps.video_generation);
        assert!(!caps.audio_generation);
    }

    #[test]
    fn deepseek_capabilities_stay_prompt_assist_only() {
        let caps = default_capabilities(CloudProviderKind::DeepSeek);
        assert!(caps.text_assist);
        assert!(!caps.vision_assist);
        assert!(!caps.image_generation);
        assert!(!caps.video_generation);
        assert!(!caps.audio_generation);
    }

    #[tokio::test]
    async fn anthropic_prompt_verify_uses_messages_endpoint() {
        let (base_url, request_rx) =
            spawn_single_request_server(r#"{"content":[{"type":"text","text":"OK"}]}"#).await;
        let entry = CloudProviderEntry {
            display_name: "Anthropic Prompt".to_string(),
            provider_kind: CloudProviderKind::Anthropic,
            base_url,
            prompt_assist_model_name: "claude-sonnet-5".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Anthropic),
            ..CloudProviderEntry::default()
        };

        let status = verify_prompt_assist(&entry, "test-anthropic-key")
            .await
            .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Prompt Assist via Anthropic.");
        assert!(request.starts_with("POST /messages HTTP/1.1"));
        assert!(request.contains("x-api-key: test-anthropic-key"));
        assert!(request.contains("anthropic-version: 2023-06-01"));
        assert!(request.contains(r#""model":"claude-sonnet-5""#));
        assert!(request.contains(r#""content":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn anthropic_prompt_generation_returns_text_only_json() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"content":[{"type":"text","text":"{\"prompt\":\"lush mossy ruin at dawn\"}"}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "Anthropic Prompt".to_string(),
            provider_kind: CloudProviderKind::Anthropic,
            base_url,
            prompt_assist_model_name: "claude-sonnet-5".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Anthropic),
            ..CloudProviderEntry::default()
        };

        let output = generate_prompt_assist_json(
            &entry,
            "test-anthropic-key",
            "Expand this into generator-ready JSON only.",
            &json!({"type":"object","properties":{"prompt":{"type":"string"}}}),
            160,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(output, r#"{"prompt":"lush mossy ruin at dawn"}"#);
        assert!(request.starts_with("POST /messages HTTP/1.1"));
        assert!(request.contains(r#""model":"claude-sonnet-5""#));
        assert!(request.contains("Expand this into generator-ready JSON only."));
        assert!(request.contains("Return only JSON that matches this schema exactly."));
        assert!(request.contains(r#""max_tokens":160"#));
    }

    #[tokio::test]
    async fn xai_grok_prompt_verify_uses_openai_chat_shape() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"OK"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "xAI Grok Prompt".to_string(),
            provider_kind: CloudProviderKind::XAiGrok,
            base_url,
            prompt_assist_model_name: "grok-4-fast-reasoning".to_string(),
            capabilities: default_capabilities(CloudProviderKind::XAiGrok),
            ..CloudProviderEntry::default()
        };

        let status = verify_prompt_assist(&entry, "grok-test-key").await.unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Prompt Assist via xAI Grok Prompt.");
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains("authorization: Bearer grok-test-key"));
        assert!(request.contains(r#""model":"grok-4-fast-reasoning""#));
        assert!(request.contains(r#""content":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn xai_grok_prompt_generation_returns_text_only_json() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"{\"prompt\":\"storm-lit canyon shrine\"}"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "xAI Grok Prompt".to_string(),
            provider_kind: CloudProviderKind::XAiGrok,
            base_url,
            prompt_assist_model_name: "grok-4-fast-reasoning".to_string(),
            capabilities: default_capabilities(CloudProviderKind::XAiGrok),
            ..CloudProviderEntry::default()
        };

        let output = generate_prompt_assist_json(
            &entry,
            "grok-test-key",
            "Expand this into generator-ready JSON only.",
            &json!({"type":"object","properties":{"prompt":{"type":"string"}}}),
            160,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(output, r#"{"prompt":"storm-lit canyon shrine"}"#);
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains(r#""model":"grok-4-fast-reasoning""#));
        assert!(request.contains("Expand this into generator-ready JSON only."));
        assert!(request.contains("Return only JSON that matches this schema exactly."));
        assert!(request.contains(r#""max_tokens":160"#));
    }

    #[tokio::test]
    async fn deepseek_prompt_verify_uses_openai_chat_shape() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"OK"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "DeepSeek Prompt".to_string(),
            provider_kind: CloudProviderKind::DeepSeek,
            base_url,
            prompt_assist_model_name: "deepseek-chat".to_string(),
            capabilities: default_capabilities(CloudProviderKind::DeepSeek),
            ..CloudProviderEntry::default()
        };

        let status = verify_prompt_assist(&entry, "deepseek-test-key").await.unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Prompt Assist via DeepSeek Prompt.");
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains("authorization: Bearer deepseek-test-key"));
        assert!(request.contains(r#""model":"deepseek-chat""#));
        assert!(request.contains(r#""content":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn deepseek_prompt_generation_returns_text_only_json() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"{\"prompt\":\"paper lantern alley after rain\"}"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "DeepSeek Prompt".to_string(),
            provider_kind: CloudProviderKind::DeepSeek,
            base_url,
            prompt_assist_model_name: "deepseek-chat".to_string(),
            capabilities: default_capabilities(CloudProviderKind::DeepSeek),
            ..CloudProviderEntry::default()
        };

        let output = generate_prompt_assist_json(
            &entry,
            "deepseek-test-key",
            "Expand this into generator-ready JSON only.",
            &json!({"type":"object","properties":{"prompt":{"type":"string"}}}),
            160,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(output, r#"{"prompt":"paper lantern alley after rain"}"#);
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains(r#""model":"deepseek-chat""#));
        assert!(request.contains("Expand this into generator-ready JSON only."));
        assert!(request.contains("Return only JSON that matches this schema exactly."));
        assert!(request.contains(r#""max_tokens":160"#));
    }

    #[tokio::test]
    async fn anthropic_vision_verify_uses_messages_endpoint_with_image_payload() {
        let (base_url, request_rx) =
            spawn_single_request_server(r#"{"content":[{"type":"text","text":"OK"}]}"#).await;
        let entry = CloudProviderEntry {
            display_name: "Anthropic Vision".to_string(),
            provider_kind: CloudProviderKind::Anthropic,
            base_url,
            vision_model_name: "claude-sonnet-5".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Anthropic),
            ..CloudProviderEntry::default()
        };

        let status = verify_vision_assist(&entry, "test-anthropic-key")
            .await
            .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Vision Assist via Anthropic.");
        assert!(request.starts_with("POST /messages HTTP/1.1"));
        assert!(request.contains("x-api-key: test-anthropic-key"));
        assert!(request.contains("anthropic-version: 2023-06-01"));
        assert!(request.contains(r#""model":"claude-sonnet-5""#));
        assert!(request.contains(r#""type":"image""#));
        assert!(request.contains(r#""text":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn anthropic_vision_generation_sends_reference_image_and_returns_text() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"content":[{"type":"text","text":"{\"scene\":\"frog on branch\"}"}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "Anthropic Vision".to_string(),
            provider_kind: CloudProviderKind::Anthropic,
            base_url,
            vision_model_name: "claude-sonnet-5".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Anthropic),
            ..CloudProviderEntry::default()
        };
        let image_path = write_test_png("anthropic-vision");

        let output = generate_vision_assist_json(
            &entry,
            "test-anthropic-key",
            "Describe the scene as JSON only.",
            &json!({"type":"object","properties":{"scene":{"type":"string"}}}),
            &image_path,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();
        let _ = fs::remove_file(&image_path);

        assert_eq!(output, r#"{"scene":"frog on branch"}"#);
        assert!(request.starts_with("POST /messages HTTP/1.1"));
        assert!(request.contains(r#""model":"claude-sonnet-5""#));
        assert!(request.contains(r#""media_type":"image/png""#));
        assert!(request.contains("Describe the scene as JSON only."));
        assert!(request.contains("scene"));
    }

    #[tokio::test]
    async fn openai_vision_verify_uses_openai_chat_shape() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"OK"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "OpenAI Vision".to_string(),
            provider_kind: CloudProviderKind::OpenAi,
            base_url,
            vision_model_name: "gpt-4.1-mini".to_string(),
            capabilities: default_capabilities(CloudProviderKind::OpenAi),
            ..CloudProviderEntry::default()
        };

        let status = verify_vision_assist(&entry, "openai-test-key").await.unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Vision Assist via OpenAI Vision.");
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains("authorization: Bearer openai-test-key"));
        assert!(request.contains(r#""model":"gpt-4.1-mini""#));
        assert!(request.contains(r#""content":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn openai_vision_generation_sends_image_data_url_and_returns_text() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"{\"subject\":\"tree frog\"}"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "OpenAI Vision".to_string(),
            provider_kind: CloudProviderKind::OpenAi,
            base_url,
            vision_model_name: "gpt-4.1-mini".to_string(),
            capabilities: default_capabilities(CloudProviderKind::OpenAi),
            ..CloudProviderEntry::default()
        };
        let image_path = write_test_png("openai-vision");

        let output = generate_vision_assist_json(
            &entry,
            "openai-test-key",
            "Return only a JSON subject summary.",
            &json!({"type":"object","properties":{"subject":{"type":"string"}}}),
            &image_path,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();
        let _ = fs::remove_file(&image_path);

        assert_eq!(output, r#"{"subject":"tree frog"}"#);
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains(r#""type":"image_url""#));
        assert!(request.contains(r#""url":"data:image/png;base64,"#));
        assert!(request.contains("Return only a JSON subject summary."));
    }

    #[tokio::test]
    async fn gemini_vision_verify_uses_openai_chat_shape() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"OK"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "Gemini Vision".to_string(),
            provider_kind: CloudProviderKind::Gemini,
            base_url,
            vision_model_name: "gemini-3.5-flash".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Gemini),
            ..CloudProviderEntry::default()
        };

        let status = verify_vision_assist(&entry, "gemini-test-key").await.unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Vision Assist via Gemini Vision.");
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains("authorization: Bearer gemini-test-key"));
        assert!(request.contains(r#""model":"gemini-3.5-flash""#));
        assert!(request.contains(r#""content":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn gemini_vision_generation_sends_image_data_url_and_returns_text() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"{\"mood\":\"lush canopy\"}"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "Gemini Vision".to_string(),
            provider_kind: CloudProviderKind::Gemini,
            base_url,
            vision_model_name: "gemini-3.5-flash".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Gemini),
            ..CloudProviderEntry::default()
        };
        let image_path = write_test_png("gemini-vision");

        let output = generate_vision_assist_json(
            &entry,
            "gemini-test-key",
            "Return only a JSON mood summary.",
            &json!({"type":"object","properties":{"mood":{"type":"string"}}}),
            &image_path,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();
        let _ = fs::remove_file(&image_path);

        assert_eq!(output, r#"{"mood":"lush canopy"}"#);
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains(r#""type":"image_url""#));
        assert!(request.contains(r#""url":"data:image/png;base64,"#));
        assert!(request.contains("Return only a JSON mood summary."));
    }

    #[tokio::test]
    async fn gemini_image_generation_uses_b64_response_format() {
        let png = BASE64_STANDARD
            .encode(BASE64_STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC").unwrap());
        let body = format!(r#"{{"data":[{{"b64_json":"{png}"}}]}}"#);
        let (base_url, request_rx) = spawn_single_request_server(Box::leak(body.into_boxed_str())).await;
        let entry = CloudProviderEntry {
            display_name: "Gemini Media".to_string(),
            provider_kind: CloudProviderKind::Gemini,
            base_url,
            image_generation_model_name: "gemini-2.5-flash-image".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Gemini),
            ..CloudProviderEntry::default()
        };

        let result = generate_cloud_image(
            &entry,
            "gemini-test-key",
            "flat neutral gray square, no text, no objects",
            "256x256",
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(result.mime, "image/png");
        assert!(!result.bytes.is_empty());
        assert!(request.starts_with("POST /images/generations HTTP/1.1"));
        assert!(request.contains(r#""response_format":"b64_json""#));
        assert!(request.contains(r#""n":1"#));
        assert!(request.contains(r#""model":"gemini-2.5-flash-image""#));
    }

    #[tokio::test]
    async fn gemini_speech_generation_uses_native_tts_route_and_default_voice() {
        let audio = BASE64_STANDARD.encode([1u8, 2, 3, 4]);
        let body = format!(
            r#"{{"candidates":[{{"content":{{"parts":[{{"inlineData":{{"data":"{audio}"}}}}]}}}}]}}"#
        );
        let (base_url, request_rx) = spawn_single_request_server(Box::leak(body.into_boxed_str())).await;
        let entry = CloudProviderEntry {
            display_name: "Gemini Speech".to_string(),
            provider_kind: CloudProviderKind::Gemini,
            base_url: format!("{base_url}/openai"),
            audio_generation_model_name: "gemini-3.1-flash-tts-preview".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Gemini),
            ..CloudProviderEntry::default()
        };

        let result = generate_cloud_speech(
            &entry,
            "gemini-test-key",
            "Hello there, traveler.",
            Some("Warm, calm narration."),
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(result.mime, "audio/wav");
        assert_eq!(result.voice, "Kore");
        assert!(request.starts_with("POST /models/gemini-3.1-flash-tts-preview:generateContent HTTP/1.1"));
        assert!(request.contains("x-goog-api-key: gemini-test-key"));
        assert!(request.contains(r#""voiceName":"Kore""#));
        assert!(request.contains("Hello there, traveler."));
        assert!(request.contains("Warm, calm narration."));
    }

    #[tokio::test]
    async fn openai_image_generation_uses_png_output_format() {
        let png = BASE64_STANDARD
            .encode(BASE64_STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC").unwrap());
        let body = format!(r#"{{"data":[{{"b64_json":"{png}"}}]}}"#);
        let (base_url, request_rx) = spawn_single_request_server(Box::leak(body.into_boxed_str())).await;
        let entry = CloudProviderEntry {
            display_name: "OpenAI Media".to_string(),
            provider_kind: CloudProviderKind::OpenAi,
            base_url,
            image_generation_model_name: "gpt-image-2".to_string(),
            capabilities: default_capabilities(CloudProviderKind::OpenAi),
            ..CloudProviderEntry::default()
        };

        let result = generate_cloud_image(
            &entry,
            "openai-image-key",
            "flat neutral gray square, no text, no objects",
            "256x256",
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(result.mime, "image/png");
        assert!(!result.bytes.is_empty());
        assert!(request.starts_with("POST /images/generations HTTP/1.1"));
        assert!(request.contains(r#""output_format":"png""#));
        assert!(request.contains(r#""model":"gpt-image-2""#));
    }

    #[tokio::test]
    async fn openai_speech_generation_uses_default_marin_voice() {
        let wav_body = "RIFFdemoWAVEfmt ";
        let (base_url, mut requests) = spawn_sequence_server(vec![(
            "200 OK",
            "audio/wav",
            wav_body,
        )])
        .await;
        let entry = CloudProviderEntry {
            display_name: "OpenAI Speech".to_string(),
            provider_kind: CloudProviderKind::OpenAi,
            base_url,
            audio_generation_model_name: "gpt-4o-mini-tts".to_string(),
            capabilities: default_capabilities(CloudProviderKind::OpenAi),
            ..CloudProviderEntry::default()
        };

        let result = generate_cloud_speech(
            &entry,
            "openai-speech-key",
            "Hello there, traveler.",
            Some("Warm, calm narration."),
        )
        .await
        .unwrap();
        let request = requests.recv().await.unwrap();

        assert_eq!(result.mime, "audio/wav");
        assert_eq!(result.voice, "marin");
        assert!(request.starts_with("POST /audio/speech HTTP/1.1"));
        assert!(request.contains(r#""voice":"marin""#));
        assert!(request.contains("Hello there, traveler."));
        assert!(request.contains("Warm, calm narration."));
    }

    #[tokio::test]
    async fn gemini_video_generation_uses_extra_body_shape_and_normalized_duration() {
        let create_body = r#"{"id":"vid_123","status":"queued","model":"veo-3.1-generate-preview","size":"1280x720","seconds":6}"#;
        let poll_body = r#"{"id":"vid_123","status":"completed","model":"veo-3.1-generate-preview","size":"1280x720","seconds":6}"#;
        let content_body = "FAKE_MP4_BYTES";
        let (base_url, mut requests) = spawn_sequence_server(vec![
            ("200 OK", "application/json", create_body),
            ("200 OK", "application/json", poll_body),
            ("200 OK", "video/mp4", content_body),
        ])
        .await;
        let entry = CloudProviderEntry {
            display_name: "Gemini Video".to_string(),
            provider_kind: CloudProviderKind::Gemini,
            base_url,
            video_generation_model_name: "veo-3.1-generate-preview".to_string(),
            capabilities: default_capabilities(CloudProviderKind::Gemini),
            ..CloudProviderEntry::default()
        };

        let result = super::generate_cloud_video(
            &entry,
            "gemini-video-key",
            "single calm abstract gradient, no text, no people",
            "1280x720",
            5,
            None,
        )
        .await
        .unwrap();
        let create_request = requests.recv().await.unwrap();
        let poll_request = requests.recv().await.unwrap();
        let content_request = requests.recv().await.unwrap();

        assert_eq!(result.mime, "video/mp4");
        assert_eq!(result.model, "veo-3.1-generate-preview");
        assert_eq!(result.size, "1280x720");
        assert_eq!(result.seconds, 6);
        assert_eq!(result.bytes, content_body.as_bytes());
        assert!(create_request.starts_with("POST /videos HTTP/1.1"));
        assert!(create_request.contains(r#""aspect_ratio":"16:9""#));
        assert!(create_request.contains(r#""resolution":"720p""#));
        assert!(create_request.contains(r#""duration_seconds":6"#));
        assert!(create_request.contains(r#""model":"veo-3.1-generate-preview""#));
        assert!(poll_request.starts_with("GET /videos/vid_123 HTTP/1.1"));
        assert!(content_request.starts_with("GET /videos/vid_123/content HTTP/1.1"));
    }

    #[tokio::test]
    async fn openai_compatible_vision_verify_uses_openai_chat_shape() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"OK"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "Compatible Vision".to_string(),
            provider_kind: CloudProviderKind::OpenAiCompatible,
            base_url,
            vision_model_name: "vision-compatible-model".to_string(),
            capabilities: default_capabilities(CloudProviderKind::OpenAiCompatible),
            ..CloudProviderEntry::default()
        };

        let status = verify_vision_assist(&entry, "compatible-key").await.unwrap();
        let request = request_rx.await.unwrap();

        assert_eq!(status, "Health check passed for Vision Assist via Compatible Vision.");
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains("authorization: Bearer compatible-key"));
        assert!(request.contains(r#""model":"vision-compatible-model""#));
        assert!(request.contains(r#""content":"Reply with OK.""#));
    }

    #[tokio::test]
    async fn openai_compatible_vision_generation_sends_image_data_url() {
        let (base_url, request_rx) = spawn_single_request_server(
            r#"{"choices":[{"message":{"content":"{\"palette\":\"moss green\"}"}}]}"#,
        )
        .await;
        let entry = CloudProviderEntry {
            display_name: "Compatible Vision".to_string(),
            provider_kind: CloudProviderKind::OpenAiCompatible,
            base_url,
            vision_model_name: "vision-compatible-model".to_string(),
            capabilities: default_capabilities(CloudProviderKind::OpenAiCompatible),
            ..CloudProviderEntry::default()
        };
        let image_path = write_test_png("openai-compatible-vision");

        let output = generate_vision_assist_json(
            &entry,
            "compatible-key",
            "Return only a JSON palette summary.",
            &json!({"type":"object","properties":{"palette":{"type":"string"}}}),
            &image_path,
        )
        .await
        .unwrap();
        let request = request_rx.await.unwrap();
        let _ = fs::remove_file(&image_path);

        assert_eq!(output, r#"{"palette":"moss green"}"#);
        assert!(request.starts_with("POST /chat/completions HTTP/1.1"));
        assert!(request.contains(r#""type":"image_url""#));
        assert!(request.contains(r#""url":"data:image/png;base64,"#));
        assert!(request.contains("Return only a JSON palette summary."));
    }

    #[tokio::test]
    async fn openai_compatible_media_generation_stays_blocked() {
        let mut capabilities = default_capabilities(CloudProviderKind::OpenAiCompatible);
        capabilities.image_generation = true;
        let entry = CloudProviderEntry {
            display_name: "Compatible Vision".to_string(),
            provider_kind: CloudProviderKind::OpenAiCompatible,
            base_url: "https://compatible.example/v1".to_string(),
            image_generation_model_name: "compatible-image".to_string(),
            capabilities,
            ..CloudProviderEntry::default()
        };

        let error = verify_media_generation(&entry, "compatible-key")
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("Cloud media generation is not enabled for this generic OpenAI-compatible route yet."));
    }
}
