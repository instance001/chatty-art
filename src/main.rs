mod app_config;
mod audio_runtime;
mod cloud_ai;
mod gguf;
mod render;
mod runtime;
mod secrets;
mod sdcpp;
mod types;

use std::{
    collections::{HashMap, VecDeque},
    io::ErrorKind,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    http::header::CONTENT_TYPE,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use rand::random;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::{
    net::TcpListener,
    process::Command,
    sync::{Semaphore, broadcast, oneshot},
    time::timeout,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{error, info};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    app_config::load_cloud_config,
    audio_runtime::{
        detect_audio_runtime_package_support, detect_audio_runtime_support,
        generate_with_audio_runtime,
    },
    cloud_ai::{
        default_base_url, default_capabilities, generate_cloud_image, generate_cloud_speech,
        generate_cloud_video, validate_openai_video_request, verify_media_generation,
        verify_prompt_assist, verify_vision_assist,
    },
    gguf::inspect_gguf,
    render::{render_audio, render_image, render_video},
    runtime::{
        build_audio_plan, build_image_plan, build_reference_summary, build_video_plan,
        compile_prompt, compile_prompt_cloud, derive_speech_direction_heuristic,
        derive_spoken_text_heuristic,
    },
    sdcpp::{detect_sdcpp_support, generate_with_sdcpp, realism_runtime_status},
    types::{
        AssetSource, BackendRuntimeStatus, CancelAccepted, CancelRequest, EstimateConfidence,
        CloudConfig, CloudLaneAssignmentsResponse, CloudLaneAssignmentsUpdateRequest,
        CloudLaneAssignmentsUpdateResponse, CloudProviderDeleteRequest, CloudProviderDeleteResponse,
        CloudProviderEntry, CloudProviderKind, CloudProviderSummary, CloudProviderUpsertRequest,
        CloudProviderUpsertResponse, CloudProviderVerifyRequest, CloudProviderVerifyResponse,
        CloudProvidersResponse, GenerateAccepted, GenerateRequest, GenerationSettings,
        GenerationStyle, HardwareProfile, InputAsset, LoraInfo, MediaKind, ModelBackend,
        ModelInfo, OutputEntry, PrepareResponse, PromptAssistMode, ReferenceSummary,
        ResolutionPreset, RuntimeAcceleration, RuntimeStatus, ServerEvent, TimeEstimate,
        VideoResolutionPreset,
    },
};

const MAX_RUNTIME_SEED: u64 = u32::MAX as u64;
const GENERATION_CANCELED_MESSAGE: &str = "Generation canceled.";

#[derive(Debug)]
struct AppPaths {
    models_dir: PathBuf,
    input_dir: PathBuf,
    outputs_dir: PathBuf,
    runtime_dir: PathBuf,
    config_dir: PathBuf,
    config_path: PathBuf,
    secrets_path: PathBuf,
    diffuse_runtime_dir: PathBuf,
    audio_runtime_dir: PathBuf,
}

impl AppPaths {
    fn discover() -> Result<Self> {
        let root = discover_app_root()?;
        Ok(Self {
            models_dir: root.join("models"),
            input_dir: root.join("input"),
            outputs_dir: root.join("outputs"),
            runtime_dir: root.join("runtime"),
            config_dir: root.join("runtime").join("config"),
            config_path: app_config::default_config_path(&root.join("runtime")),
            secrets_path: secrets::default_secrets_path(&root.join("runtime")),
            diffuse_runtime_dir: root.join("diffuse_runtime"),
            audio_runtime_dir: root.join("audio_runtime"),
        })
    }

    fn ensure_layout(&self) -> Result<()> {
        std::fs::create_dir_all(&self.models_dir)?;
        std::fs::create_dir_all(self.input_dir.join("images"))?;
        std::fs::create_dir_all(self.input_dir.join("audio"))?;
        std::fs::create_dir_all(self.input_dir.join("video"))?;
        std::fs::create_dir_all(self.outputs_dir.join("image"))?;
        std::fs::create_dir_all(self.outputs_dir.join("gif"))?;
        std::fs::create_dir_all(self.outputs_dir.join("video"))?;
        std::fs::create_dir_all(self.outputs_dir.join("audio"))?;
        std::fs::create_dir_all(&self.audio_runtime_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        Ok(())
    }
}

fn discover_app_root() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    candidates.push(std::env::current_dir().context("failed to get current directory")?);
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for candidate in candidates {
        if let Some(root) = find_chatty_art_root(&candidate) {
            return Ok(root);
        }
    }

    bail!(
        "Could not find the Chatty-art project root. Start the app from inside the project folder, or make sure the folder contains Cargo.toml, src/main.rs, and static/index.html."
    )
}

fn find_chatty_art_root(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        if is_chatty_art_root(ancestor) {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn is_chatty_art_root(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        && path.join("src").join("main.rs").exists()
        && path.join("static").join("index.html").exists()
}

#[derive(Clone)]
struct AppState {
    paths: Arc<AppPaths>,
    cloud_config: Arc<RwLock<CloudConfig>>,
    events: broadcast::Sender<ServerEvent>,
    generation_gate: Arc<Semaphore>,
    gpu_telemetry: Arc<RwLock<GpuTelemetrySnapshot>>,
    hardware_profile: Arc<HardwareProfile>,
    active_jobs: Arc<Mutex<HashMap<Uuid, Arc<JobControl>>>>,
}

#[derive(Debug)]
struct JobControl {
    canceled: AtomicBool,
}

#[derive(Debug, Deserialize)]
struct BridgeLoraImportRequest {
    lane_id: String,
    asset_ids: Vec<String>,
    family_key: String,
}

#[derive(Debug, Deserialize)]
struct DeleteOutputsRequest {
    relative_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DeleteOutputsResponse {
    deleted_paths: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BridgeIncomingAssetRecord {
    #[serde(default)]
    payload_file_name: String,
    #[serde(default)]
    file_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct GpuTelemetrySnapshot {
    supported: bool,
    label: String,
    note: String,
    current_percent: f32,
    history: Vec<f32>,
}

#[derive(Serialize)]
struct PlannerTraceSidecar {
    job_id: Uuid,
    kind: MediaKind,
    style: GenerationStyle,
    backend: ModelBackend,
    model: String,
    prompt_assist_route: Option<String>,
    vision_assist_route: Option<String>,
    output_route: String,
    prompt: String,
    note: String,
    used_fallback: bool,
    extracted_json: Option<String>,
    raw_output: String,
    stderr: String,
}

#[derive(Serialize)]
struct PromptAssistTraceSidecar {
    job_id: Uuid,
    kind: MediaKind,
    style: GenerationStyle,
    assist_mode: PromptAssistMode,
    generation_model: String,
    prompt_assist_route: Option<String>,
    vision_assist_route: Option<String>,
    output_route: String,
    interpreter_model: String,
    original_prompt: String,
    compiled_prompt: String,
    spoken_text: Option<String>,
    negative_prompt: Option<String>,
    note: String,
    used_original_prompt: bool,
    assumptions: Vec<String>,
    focus_tags: Vec<String>,
    extracted_json: Option<String>,
    raw_output: String,
    stderr: String,
}

type ApiResult<T> = std::result::Result<Json<T>, (StatusCode, String)>;

fn register_job(state: &AppState, job_id: Uuid) -> Arc<JobControl> {
    let control = Arc::new(JobControl {
        canceled: AtomicBool::new(false),
    });
    if let Ok(mut jobs) = state.active_jobs.lock() {
        jobs.insert(job_id, control.clone());
    }
    control
}

fn find_job_control(state: &AppState, job_id: Uuid) -> Option<Arc<JobControl>> {
    state
        .active_jobs
        .lock()
        .ok()
        .and_then(|jobs| jobs.get(&job_id).cloned())
}

fn unregister_job(state: &AppState, job_id: Uuid) {
    if let Ok(mut jobs) = state.active_jobs.lock() {
        jobs.remove(&job_id);
    }
}

fn job_is_canceled(control: &Arc<JobControl>) -> bool {
    control.canceled.load(Ordering::SeqCst)
}

fn ensure_not_canceled(control: &Arc<JobControl>) -> Result<()> {
    if job_is_canceled(control) {
        anyhow::bail!(GENERATION_CANCELED_MESSAGE);
    }
    Ok(())
}

async fn await_cancelable<F, T>(control: &Arc<JobControl>, future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    tokio::pin!(future);
    let mut interval = tokio::time::interval(Duration::from_millis(250));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            result = &mut future => return result,
            _ = interval.tick() => ensure_not_canceled(control)?,
        }
    }
}

fn is_cancellation_error(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .to_ascii_lowercase()
        .contains("generation canceled")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "chatty_art=info,tower_http=info".into()),
        )
        .init();

    let paths = Arc::new(AppPaths::discover()?);
    paths.ensure_layout()?;

    let (events, _) = broadcast::channel(256);
    let gpu_telemetry = Arc::new(RwLock::new(initial_gpu_telemetry()));
    let hardware_profile = Arc::new(detect_hardware_profile().await);
    let cloud_config = Arc::new(RwLock::new(load_cloud_config(&paths.config_path)?));
    let state = AppState {
        paths: paths.clone(),
        cloud_config,
        events,
        generation_gate: Arc::new(Semaphore::new(1)),
        gpu_telemetry: gpu_telemetry.clone(),
        hardware_profile,
        active_jobs: Arc::new(Mutex::new(HashMap::new())),
    };
    spawn_gpu_sampler(gpu_telemetry);

    let app = Router::new()
        .route("/", get(index_page))
        .route("/app.js", get(app_js))
        .route("/styles.css", get(styles_css))
        .route("/api/models", get(list_models))
        .route("/api/loras", get(list_loras))
        .route("/api/loras/import-bridge", post(import_bridge_loras))
        .route("/api/runtime", get(runtime_status))
        .route("/api/hardware", get(hardware_profile_status))
        .route("/api/telemetry/gpu", get(gpu_telemetry_status))
        .route("/api/cloud/providers", get(list_cloud_providers))
        .route("/api/cloud/providers", post(save_cloud_provider))
        .route("/api/cloud/providers/delete", post(delete_cloud_provider))
        .route("/api/cloud/providers/verify", post(verify_cloud_provider))
        .route("/api/cloud/lanes", get(get_cloud_lane_assignments))
        .route("/api/cloud/lanes", post(update_cloud_lane_assignments))
        .route("/api/assets", get(list_assets))
        .route("/api/outputs", get(list_outputs))
        .route("/api/outputs/delete", post(delete_outputs))
        .route("/api/prepare", post(prepare_generate))
        .route("/api/generate", post(start_generate))
        .route("/api/cancel", post(cancel_generate))
        .route("/ws", get(websocket_endpoint))
        .nest_service("/outputs", ServeDir::new(paths.outputs_dir.clone()))
        .nest_service("/input", ServeDir::new(paths.input_dir.clone()))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    match claim_server_address(7878).await? {
        ServerBinding::Existing(url) => {
            info!("Chatty-art is already running at {url}");
            println!("Chatty-art is already running at {url}");
            if std::env::var("CHATTY_ART_NO_BROWSER").ok().as_deref() != Some("1") {
                let _ = webbrowser::open(&url);
            }
        }
        ServerBinding::Fresh(listener, url) => {
            info!("Chatty-art is running at {url}");
            println!("Chatty-art is running at {url}");
            if std::env::var("CHATTY_ART_NO_BROWSER").ok().as_deref() != Some("1") {
                let _ = webbrowser::open(&url);
            }
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

enum ServerBinding {
    Existing(String),
    Fresh(TcpListener, String),
}

async fn claim_server_address(preferred_port: u16) -> Result<ServerBinding> {
    let preferred = SocketAddr::from(([127, 0, 0, 1], preferred_port));
    match TcpListener::bind(preferred).await {
        Ok(listener) => {
            let url = format!("http://{}", listener.local_addr()?);
            Ok(ServerBinding::Fresh(listener, url))
        }
        Err(error) if error.kind() == ErrorKind::AddrInUse => {
            let existing_url = format!("http://127.0.0.1:{preferred_port}");
            if is_chatty_art_instance(&existing_url).await {
                return Ok(ServerBinding::Existing(existing_url));
            }

            for offset in 1..20 {
                let candidate = SocketAddr::from(([127, 0, 0, 1], preferred_port + offset));
                if let Ok(listener) = TcpListener::bind(candidate).await {
                    let url = format!("http://{}", listener.local_addr()?);
                    return Ok(ServerBinding::Fresh(listener, url));
                }
            }

            Err(error.into())
        }
        Err(error) => Err(error.into()),
    }
}

async fn is_chatty_art_instance(base_url: &str) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    let response = match client.get(base_url).send().await {
        Ok(response) => response,
        Err(_) => return false,
    };
    let text = match response.text().await {
        Ok(text) => text,
        Err(_) => return false,
    };

    text.contains("Chatty-art")
}

async fn list_models(State(state): State<AppState>) -> ApiResult<Vec<ModelInfo>> {
    scan_models(
        &state.paths.models_dir,
        &state.paths.diffuse_runtime_dir,
        &state.paths.audio_runtime_dir,
    )
    .map(Json)
    .map_err(internal_error)
}

async fn list_loras(State(state): State<AppState>) -> ApiResult<Vec<LoraInfo>> {
    scan_loras(&state.paths.models_dir)
        .map(Json)
        .map_err(internal_error)
}

async fn import_bridge_loras(
    State(state): State<AppState>,
    Json(request): Json<BridgeLoraImportRequest>,
) -> impl IntoResponse {
    match import_bridge_lora_files(&state.paths, request) {
        Ok(notes) => Json(serde_json::json!({ "notes": notes })).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn runtime_status(State(state): State<AppState>) -> ApiResult<RuntimeStatus> {
    Ok(Json(RuntimeStatus {
        expressive: expressive_runtime_status(&state.paths.runtime_dir),
        realism: realism_runtime_status(&state.paths.diffuse_runtime_dir),
    }))
}

async fn gpu_telemetry_status(State(state): State<AppState>) -> ApiResult<GpuTelemetrySnapshot> {
    Ok(Json(state.gpu_telemetry.read().await.clone()))
}

async fn hardware_profile_status(State(state): State<AppState>) -> ApiResult<HardwareProfile> {
    Ok(Json((*state.hardware_profile).clone()))
}

async fn list_cloud_providers(State(state): State<AppState>) -> ApiResult<CloudProvidersResponse> {
    let config = state.cloud_config.read().await.clone();
    Ok(Json(CloudProvidersResponse {
        api_key_lanes_enabled: config.api_key_lanes_enabled,
        providers: config
            .cloud_providers
            .iter()
            .map(|entry| summarize_provider(&state.paths, entry))
            .collect::<Result<Vec<_>, _>>()
            .map_err(internal_error)?,
    }))
}

async fn save_cloud_provider(
    State(state): State<AppState>,
    Json(request): Json<CloudProviderUpsertRequest>,
) -> ApiResult<CloudProviderUpsertResponse> {
    let mut config = state.cloud_config.write().await;
    config.api_key_lanes_enabled = true;

    let provider_id = request
        .id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let provider_kind = request.provider_kind;
    let enabled = request.enabled;
    let mut entry = CloudProviderEntry {
        id: provider_id.clone(),
        display_name: request.display_name.trim().to_string(),
        provider_kind,
        base_url: request.base_url.trim().trim_end_matches('/').to_string(),
        prompt_assist_model_name: request.prompt_assist_model_name.trim().to_string(),
        vision_model_name: request.vision_model_name.trim().to_string(),
        image_generation_model_name: request.image_generation_model_name.trim().to_string(),
        video_generation_model_name: request.video_generation_model_name.trim().to_string(),
        audio_generation_model_name: request.audio_generation_model_name.trim().to_string(),
        audio_generation_voice: request.audio_generation_voice.trim().to_string(),
        enabled,
        capabilities: default_capabilities(provider_kind),
        prompt_assist_verification: Default::default(),
        vision_assist_verification: Default::default(),
        media_generation_verification: Default::default(),
    };

    if entry.base_url.is_empty() {
        entry.base_url = default_base_url(provider_kind).to_string();
    }
    let parsed_base_url = reqwest::Url::parse(&entry.base_url).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Enter a full cloud base URL, including http:// or https://.".to_string(),
        )
    })?;
    if !matches!(parsed_base_url.scheme(), "http" | "https") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cloud base URLs must start with http:// or https://.".to_string(),
        ));
    }
    validate_provider_entry_against_capabilities(&entry)?;
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
            provider_id.clone()
        };
    }
    if entry.prompt_assist_model_name.is_empty()
        && entry.vision_model_name.is_empty()
        && entry.image_generation_model_name.is_empty()
        && entry.video_generation_model_name.is_empty()
        && entry.audio_generation_model_name.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Set at least one cloud model name before saving this cloud account or route."
                .to_string(),
        ));
    }

    if let Some(api_key) = request.api_key.as_deref().map(str::trim) {
        if !api_key.is_empty() {
            secrets::save_api_key(&state.paths.secrets_path, &provider_id, api_key)
                .map_err(internal_error)?;
        }
    }

    if !secrets::has_api_key(&state.paths.secrets_path, &provider_id).map_err(internal_error)? {
        return Err((
            StatusCode::BAD_REQUEST,
            "Enter an API key before saving this cloud account or route.".to_string(),
        ));
    }

    if let Some(existing) = config
        .cloud_providers
        .iter_mut()
        .find(|existing| existing.id == provider_id)
    {
        entry.prompt_assist_verification = existing.prompt_assist_verification.clone();
        entry.vision_assist_verification = existing.vision_assist_verification.clone();
        entry.media_generation_verification = existing.media_generation_verification.clone();
        existing.clone_from(&entry);
    } else {
        config.cloud_providers.push(entry.clone());
    }

    app_config::save_cloud_config(&state.paths.config_path, &config).map_err(internal_error)?;
    let summary = summarize_provider(&state.paths, &entry).map_err(internal_error)?;
    Ok(Json(CloudProviderUpsertResponse {
        provider: summary,
        note: "Saved cloud account or route.".to_string(),
    }))
}

fn validate_provider_entry_against_capabilities(
    entry: &CloudProviderEntry,
) -> Result<(), (StatusCode, String)> {
    if !entry.capabilities.text_assist && !entry.prompt_assist_model_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "This saved account or route is not wired for Prompt Assist yet.".to_string(),
        ));
    }
    if !entry.capabilities.vision_assist && !entry.vision_model_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "This saved account or route is not wired for Vision Assist yet.".to_string(),
        ));
    }
    if !entry.capabilities.image_generation && !entry.image_generation_model_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "This saved account or route is not wired for cloud image generation yet.".to_string(),
        ));
    }
    if !entry.capabilities.video_generation && !entry.video_generation_model_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "This saved account or route is not wired for cloud video generation yet.".to_string(),
        ));
    }
    if !entry.capabilities.audio_generation && !entry.audio_generation_model_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "This saved account or route is not wired for cloud speech generation yet.".to_string(),
        ));
    }
    if !entry.capabilities.audio_generation && !entry.audio_generation_voice.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Save a voice or speaker id only on routes that already support cloud speech generation.".to_string(),
        ));
    }
    Ok(())
}

async fn delete_cloud_provider(
    State(state): State<AppState>,
    Json(request): Json<CloudProviderDeleteRequest>,
) -> ApiResult<CloudProviderDeleteResponse> {
    let mut config = state.cloud_config.write().await;
    let provider_id = request.id.trim().to_string();
    if provider_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Provider id is required.".to_string()));
    }
    config.cloud_providers.retain(|entry| entry.id != provider_id);
    if config.lane_assignments.prompt_assist == format!("cloud:{provider_id}") {
        config.lane_assignments.prompt_assist = "local_auto".to_string();
    }
    if config.lane_assignments.vision_assist == format!("cloud:{provider_id}") {
        config.lane_assignments.vision_assist = "local_auto".to_string();
    }
    if config.lane_assignments.media_generation == format!("cloud:{provider_id}") {
        config.lane_assignments.media_generation = "local_only".to_string();
    }
    secrets::delete_api_key(&state.paths.secrets_path, &provider_id).map_err(internal_error)?;
    app_config::save_cloud_config(&state.paths.config_path, &config).map_err(internal_error)?;
    Ok(Json(CloudProviderDeleteResponse {
        id: provider_id,
        note: "Deleted cloud account or route.".to_string(),
    }))
}

async fn verify_cloud_provider(
    State(state): State<AppState>,
    Json(request): Json<CloudProviderVerifyRequest>,
) -> ApiResult<CloudProviderVerifyResponse> {
    let provider_id = request.id.trim().to_string();
    if provider_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Provider id is required.".to_string()));
    }
    let maybe_entry = {
        let config = state.cloud_config.read().await;
        config
            .cloud_providers
            .iter()
            .find(|entry| entry.id == provider_id)
            .cloned()
    };
    let entry = if let Some(entry) = maybe_entry {
        entry
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            "Cloud account or route was not found.".to_string(),
        ));
    };
    let api_key = secrets::read_api_key(&state.paths.secrets_path, &provider_id)
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "No API key is stored for that cloud account or route.".to_string(),
            )
        })?;

    let lane = request
        .lane
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("prompt_assist");
    let verification_note = match lane {
        "vision_assist" => match verify_vision_assist(&entry, &api_key).await {
            Ok(note) => note,
            Err(error) => error.to_string(),
        },
        "media_generation" => match verify_media_generation(&entry, &api_key).await {
            Ok(note) => note,
            Err(error) => error.to_string(),
        },
        _ => match verify_prompt_assist(&entry, &api_key).await {
            Ok(note) => note,
            Err(error) => error.to_string(),
        },
    };
    let checked_at_unix_ms = now_unix_ms();

    let summary = {
        let mut config = state.cloud_config.write().await;
        let updated = config
            .cloud_providers
            .iter_mut()
            .find(|existing| existing.id == provider_id)
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    "Cloud account or route disappeared during verification.".to_string(),
                )
            })?;
        if lane == "vision_assist" {
            updated.vision_assist_verification.status = verification_note.clone();
            updated.vision_assist_verification.checked_at_unix_ms = checked_at_unix_ms;
        } else if lane == "media_generation" {
            updated.media_generation_verification.status = verification_note.clone();
            updated.media_generation_verification.checked_at_unix_ms = checked_at_unix_ms;
        } else {
            updated.prompt_assist_verification.status = verification_note.clone();
            updated.prompt_assist_verification.checked_at_unix_ms = checked_at_unix_ms;
        }
        let snapshot = updated.clone();
        app_config::save_cloud_config(&state.paths.config_path, &config).map_err(internal_error)?;
        summarize_provider(&state.paths, &snapshot).map_err(internal_error)?
    };

    Ok(Json(CloudProviderVerifyResponse {
        provider: summary,
        note: verification_note,
    }))
}

async fn get_cloud_lane_assignments(
    State(state): State<AppState>,
) -> ApiResult<CloudLaneAssignmentsResponse> {
    let config = state.cloud_config.read().await;
    Ok(Json(CloudLaneAssignmentsResponse {
        lane_assignments: config.lane_assignments.clone(),
    }))
}

async fn update_cloud_lane_assignments(
    State(state): State<AppState>,
    Json(request): Json<CloudLaneAssignmentsUpdateRequest>,
) -> ApiResult<CloudLaneAssignmentsUpdateResponse> {
    let mut config = state.cloud_config.write().await;
    let normalized = app_config::normalize_lane_assignments_for_providers(
        request.lane_assignments,
        &config.cloud_providers,
    );
    config.lane_assignments = normalized.clone();
    config.api_key_lanes_enabled = true;
    app_config::save_cloud_config(&state.paths.config_path, &config).map_err(internal_error)?;
    Ok(Json(CloudLaneAssignmentsUpdateResponse {
        lane_assignments: normalized,
        note: "Saved cloud lane assignments.".to_string(),
    }))
}

async fn resolve_prompt_assist_cloud_target(
    state: &AppState,
) -> Result<Option<PromptAssistCloudTarget>> {
    let config = state.cloud_config.read().await.clone();
    let selection = config.lane_assignments.prompt_assist.trim().to_string();
    let Some(provider_id) = selection.strip_prefix("cloud:").map(str::trim) else {
        return Ok(None);
    };
    if provider_id.is_empty() {
        return Ok(None);
    }
    let entry = config
        .cloud_providers
        .iter()
        .find(|entry| entry.id == provider_id && entry.enabled)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("The selected Prompt Assist cloud route is missing or disabled."))?;
    let api_key = secrets::read_api_key(&state.paths.secrets_path, provider_id)?
        .ok_or_else(|| anyhow::anyhow!("The selected Prompt Assist cloud route does not have a saved API key."))?;
    Ok(Some(PromptAssistCloudTarget { entry, api_key }))
}

async fn resolve_vision_assist_cloud_target(
    state: &AppState,
) -> Result<Option<VisionAssistCloudTarget>> {
    let config = state.cloud_config.read().await.clone();
    let selection = config.lane_assignments.vision_assist.trim().to_string();
    let Some(provider_id) = selection.strip_prefix("cloud:").map(str::trim) else {
        return Ok(None);
    };
    if provider_id.is_empty() {
        return Ok(None);
    }
    let entry = config
        .cloud_providers
        .iter()
        .find(|entry| entry.id == provider_id && entry.enabled)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("The selected Vision Assist cloud route is missing or disabled."))?;
    if !entry.capabilities.vision_assist {
        bail!("The selected Vision Assist cloud route is not marked as vision capable.");
    }
    if entry.vision_model_name.trim().is_empty() {
        bail!("The selected Vision Assist cloud route does not have a Vision Assist model configured.");
    }
    let api_key = secrets::read_api_key(&state.paths.secrets_path, provider_id)?
        .ok_or_else(|| anyhow::anyhow!("The selected Vision Assist cloud route does not have a saved API key."))?;
    Ok(Some(VisionAssistCloudTarget { entry, api_key }))
}

async fn resolve_media_generation_cloud_target(
    state: &AppState,
) -> Result<Option<MediaGenerationCloudTarget>> {
    let config = state.cloud_config.read().await.clone();
    let selection = config.lane_assignments.media_generation.trim().to_string();
    let Some(provider_id) = selection.strip_prefix("cloud:").map(str::trim) else {
        return Ok(None);
    };
    if provider_id.is_empty() {
        return Ok(None);
    }
    let entry = config
        .cloud_providers
        .iter()
        .find(|entry| entry.id == provider_id && entry.enabled)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("The selected cloud media route is missing or disabled."))?;
    let image_ready =
        entry.capabilities.image_generation && !entry.image_generation_model_name.trim().is_empty();
    let video_ready =
        entry.capabilities.video_generation && !entry.video_generation_model_name.trim().is_empty();
    let audio_ready =
        entry.capabilities.audio_generation && !entry.audio_generation_model_name.trim().is_empty();
    if !image_ready && !video_ready && !audio_ready {
        bail!("The selected cloud media route does not have a usable image, video, or audio generation model configured.");
    }
    let api_key = secrets::read_api_key(&state.paths.secrets_path, provider_id)?
        .ok_or_else(|| anyhow::anyhow!("The selected cloud media route does not have a saved API key."))?;
    Ok(Some(MediaGenerationCloudTarget { entry, api_key }))
}

fn is_speech_generation_model(model: &ModelInfo) -> bool {
    model.supports_voice_output
        && matches!(model.backend, ModelBackend::AudioRuntime | ModelBackend::Cloud)
}

async fn index_page() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn app_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/app.js"),
    )
}

async fn styles_css() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/styles.css"),
    )
}

fn expressive_runtime_status(runtime_dir: &Path) -> BackendRuntimeStatus {
    let has_llama_cli = runtime_dir.join("llama-cli.exe").exists();
    let has_vulkan = runtime_dir.join("ggml-vulkan.dll").exists();
    BackendRuntimeStatus {
        backend: ModelBackend::LlamaCpp,
        label: if !has_llama_cli {
            "Runtime missing".to_string()
        } else if has_vulkan {
            "Vulkan-ready".to_string()
        } else {
            "Bundled runtime".to_string()
        },
        acceleration: if !has_llama_cli {
            RuntimeAcceleration::IncompleteTree
        } else if has_vulkan {
            RuntimeAcceleration::Vulkan
        } else {
            RuntimeAcceleration::CpuOnly
        },
        note: if !has_llama_cli {
            "The bundled llama.cpp runtime is missing llama-cli.exe. Restore the expressive runtime files into runtime/ to re-enable Prompt Assist and expressive generation."
                .to_string()
        } else if has_vulkan {
            "The bundled llama.cpp runtime includes Vulkan support for expressive planning."
                .to_string()
        } else {
            "The bundled llama.cpp runtime is present, but Vulkan support was not detected in the local runtime folder.".to_string()
        },
        tooling_label: None,
        tooling_note: None,
        tooling_ready: false,
    }
}

fn initial_gpu_telemetry() -> GpuTelemetrySnapshot {
    GpuTelemetrySnapshot {
        supported: cfg!(target_os = "windows"),
        label: "ECG Window".to_string(),
        note: if cfg!(target_os = "windows") {
            "ECG-style view of the busiest Windows GPU engine, similar to Task Manager.".to_string()
        } else {
            "ECG Window is currently available on Windows only.".to_string()
        },
        current_percent: 0.0,
        history: Vec::new(),
    }
}

async fn detect_hardware_profile() -> HardwareProfile {
    if cfg!(target_os = "windows") {
        match query_windows_hardware_profile().await {
            Ok(profile) => profile,
            Err(error) => HardwareProfile {
                platform: "windows".to_string(),
                gpu_label: "Windows GPU".to_string(),
                dedicated_vram_gb: None,
                shared_memory_gb: None,
                note: format!(
                    "Hardware recommendations are using a generic Windows fallback because GPU memory detection failed: {error}"
                ),
            },
        }
    } else {
        HardwareProfile {
            platform: std::env::consts::OS.to_string(),
            gpu_label: "Local GPU".to_string(),
            dedicated_vram_gb: None,
            shared_memory_gb: None,
            note: "Hardware-aware recommendations are currently most accurate on Windows. Other platforms use a generic fallback.".to_string(),
        }
    }
}

#[derive(serde::Deserialize)]
struct WindowsHardwareProbe {
    #[serde(default)]
    gpu_label: String,
    #[serde(default)]
    dedicated_vram_bytes: Option<u64>,
    #[serde(default)]
    shared_memory_bytes: Option<u64>,
}

async fn query_windows_hardware_profile() -> Result<HardwareProfile> {
    let script = r#"
$gpu = Get-CimInstance Win32_VideoController | Select-Object -First 1 Name, AdapterRAM
$registry = Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Video\*\0000' -ErrorAction SilentlyContinue |
  Where-Object { $_.'HardwareInformation.qwMemorySize' } |
  Select-Object -First 1
$totalRam = Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty TotalPhysicalMemory
$dedicated = $null
if ($registry -and $registry.'HardwareInformation.qwMemorySize') {
  $dedicated = [uint64]$registry.'HardwareInformation.qwMemorySize'
} elseif ($gpu -and $gpu.AdapterRAM) {
  $dedicated = [uint64]$gpu.AdapterRAM
}
$shared = $null
if ($totalRam) {
  $shared = [uint64]($totalRam / 2)
}
[pscustomobject]@{
  gpu_label = if ($gpu -and $gpu.Name) { $gpu.Name.Trim() } else { 'Windows GPU' }
  dedicated_vram_bytes = $dedicated
  shared_memory_bytes = $shared
} | ConvertTo-Json -Compress
"#;

    let output = timeout(
        Duration::from_secs(6),
        Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", script])
            .output(),
    )
    .await
    .context("timed out querying Windows hardware profile")?
    .context("failed to launch PowerShell for Windows hardware profile query")?;

    if !output.status.success() {
        anyhow::bail!("PowerShell hardware profile query failed");
    }

    let probe: WindowsHardwareProbe = serde_json::from_slice(&output.stdout)
        .context("failed to parse Windows hardware profile JSON")?;

    Ok(HardwareProfile {
        platform: "windows".to_string(),
        gpu_label: if probe.gpu_label.trim().is_empty() {
            "Windows GPU".to_string()
        } else {
            probe.gpu_label.trim().to_string()
        },
        dedicated_vram_gb: probe
            .dedicated_vram_bytes
            .map(|bytes| ((bytes as f64) / 1024f64 / 1024f64 / 1024f64 * 10.0).round() as f32 / 10.0),
        shared_memory_gb: probe
            .shared_memory_bytes
            .map(|bytes| ((bytes as f64) / 1024f64 / 1024f64 / 1024f64 * 10.0).round() as f32 / 10.0),
        note: "Recommendations are based mainly on dedicated VRAM. Windows shared GPU memory can help, but Vulkan image/video jobs may still fail when they need a large contiguous GPU allocation.".to_string(),
    })
}

fn spawn_gpu_sampler(target: Arc<RwLock<GpuTelemetrySnapshot>>) {
    tokio::spawn(async move {
        if !cfg!(target_os = "windows") {
            return;
        }

        if let Ok(label) = query_windows_gpu_label().await {
            let mut guard = target.write().await;
            guard.label = label;
        }

        let mut history = VecDeque::with_capacity(60);
        let mut last_error_note = None::<String>;

        loop {
            match sample_windows_gpu_activity_percent().await {
                Ok(percent) => {
                    if history.len() >= 60 {
                        history.pop_front();
                    }
                    history.push_back(percent);

                    let mut guard = target.write().await;
                    guard.supported = true;
                    guard.current_percent = percent;
                    guard.history = history.iter().copied().collect();
                    if last_error_note.take().is_some() {
                        guard.note = "ECG-style view of the busiest Windows GPU engine, similar to Task Manager."
                            .to_string();
                    }
                }
                Err(error) => {
                    let note = format!(
                        "ECG Window uses Windows performance counters. Sampling is temporarily unavailable: {error}"
                    );
                    if last_error_note.as_deref() != Some(note.as_str()) {
                        let mut guard = target.write().await;
                        guard.note = note.clone();
                        last_error_note = Some(note);
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(1200)).await;
        }
    });
}

async fn query_windows_gpu_label() -> Result<String> {
    let script = r#"
$adapter = Get-CimInstance Win32_VideoController | Select-Object -First 1 -ExpandProperty Name
if ([string]::IsNullOrWhiteSpace($adapter)) { 'ECG Window' } else { 'ECG Window - ' + $adapter.Trim() }
"#;
    let output = timeout(
        Duration::from_secs(4),
        Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", script])
            .output(),
    )
    .await
    .context("timed out querying Windows GPU adapter label")?
    .context("failed to launch PowerShell for GPU adapter query")?;

    if !output.status.success() {
        anyhow::bail!("PowerShell GPU adapter query failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn sample_windows_gpu_activity_percent() -> Result<f32> {
    let script = r#"
$values = @(
  Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine |
    Where-Object { $_.Name -match 'engtype_(3D|Compute|Video|Copy)' } |
    ForEach-Object { [double]$_.UtilizationPercentage }
)
if ($values.Count -eq 0) {
  '0'
} else {
  [math]::Round((($values | Measure-Object -Maximum).Maximum), 1)
}
"#;

    let output = timeout(
        Duration::from_secs(5),
        Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", script])
            .output(),
    )
    .await
    .context("timed out querying Windows ECG Window counters")?
    .context("failed to launch PowerShell for ECG Window query")?;

    if !output.status.success() {
        anyhow::bail!("PowerShell ECG Window query failed");
    }

    parse_gpu_percent_output(&String::from_utf8_lossy(&output.stdout))
        .ok_or_else(|| anyhow::anyhow!("could not parse ECG Window percentage"))
}

fn parse_gpu_percent_output(output: &str) -> Option<f32> {
    output
        .trim()
        .lines()
        .rev()
        .find_map(|line| line.trim().parse::<f32>().ok())
}

async fn list_assets(State(state): State<AppState>) -> ApiResult<Vec<InputAsset>> {
    scan_assets(&state.paths.input_dir, &state.paths.outputs_dir)
        .map(Json)
        .map_err(internal_error)
}

async fn list_outputs(State(state): State<AppState>) -> ApiResult<Vec<OutputEntry>> {
    scan_outputs(&state.paths.outputs_dir)
        .map(Json)
        .map_err(internal_error)
}

async fn delete_outputs(
    State(state): State<AppState>,
    Json(request): Json<DeleteOutputsRequest>,
) -> impl IntoResponse {
    match delete_output_files(&state.paths, request) {
        Ok(response) => Json(response).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Clone)]
struct ResolvedGenerateContext {
    request: GenerateRequest,
    model: ModelInfo,
    media_generation_cloud_target: Option<MediaGenerationCloudTarget>,
    selected_loras: Vec<LoraInfo>,
    prompt_interpreter_model: Option<ModelInfo>,
    vision_interpreter_model: Option<ModelInfo>,
    vision_model_was_auto_selected: bool,
    reference_asset: Option<InputAsset>,
    end_reference_asset: Option<InputAsset>,
    control_reference_asset: Option<InputAsset>,
    used_seed: u32,
}

struct PreparedPromptState {
    effective_request: GenerateRequest,
    prompt_assist_note: String,
    compiled_prompt: Option<String>,
    prepared_spoken_text: Option<String>,
    interpreter_model_name: Option<String>,
    vision_model_name: Option<String>,
    vision_summary: Option<String>,
    vision_error: Option<String>,
    assumptions: Vec<String>,
    focus_tags: Vec<String>,
    used_original_prompt: bool,
    prompt_assist_sidecar: Option<PromptAssistTraceSidecar>,
}

#[derive(Clone)]
struct PromptAssistCloudTarget {
    entry: CloudProviderEntry,
    api_key: String,
}

#[derive(Clone)]
struct VisionAssistCloudTarget {
    entry: CloudProviderEntry,
    api_key: String,
}

#[derive(Clone)]
struct MediaGenerationCloudTarget {
    entry: CloudProviderEntry,
    api_key: String,
}

async fn prepare_generate(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> ApiResult<PrepareResponse> {
    let context = resolve_generate_context(&state, request).await?;
    let prompt_assist_cloud = resolve_prompt_assist_cloud_target(&state).await.map_err(internal_error)?;
    let vision_assist_cloud = resolve_vision_assist_cloud_target(&state).await.map_err(internal_error)?;
    let reference_image_path = context
        .reference_asset
        .as_ref()
        .filter(|asset| asset.kind == MediaKind::Image)
        .map(|asset| asset.disk_path(&state.paths.input_dir, &state.paths.outputs_dir));
    let reference_summary = match context.reference_asset.as_ref() {
        Some(asset) => Some(
            build_reference_summary(
                &asset.disk_path(&state.paths.input_dir, &state.paths.outputs_dir),
                asset,
                context.request.reference_intent,
            )
            .await
            .map_err(internal_error)?,
        ),
        None => None,
    };

    let prepared = build_prompt_handoff(
        &state.paths,
        &context.request,
        &context.model,
        context.prompt_interpreter_model.as_ref(),
        context.vision_interpreter_model.as_ref(),
        context.vision_model_was_auto_selected,
        reference_summary.as_ref(),
        context.used_seed,
        prompt_assist_cloud.as_ref(),
        vision_assist_cloud.as_ref(),
        reference_image_path.as_deref(),
    )
    .await
    .map_err(internal_error)?;

    let estimated_frames = match context.request.kind {
        MediaKind::Gif | MediaKind::Video => Some(context.request.settings.video_frame_count()),
        MediaKind::Image | MediaKind::Audio => None,
    };
    let estimated_time = estimate_generation_time(
        &context.model,
        &prepared.effective_request,
        &state.hardware_profile,
    );

    Ok(Json(PrepareResponse {
        model: context.model.name.clone(),
        kind: context.request.kind,
        style: context.request.style,
        original_prompt: context.request.prompt.trim().to_string(),
        prepared_prompt: prepared.effective_request.prompt.trim().to_string(),
        prepared_spoken_text: prepared.prepared_spoken_text,
        effective_negative_prompt: prepared.effective_request.negative_prompt.clone(),
        prompt_assist: context.request.prompt_assist,
        interpreter_model: prepared.interpreter_model_name,
        vision_model: prepared.vision_model_name,
        vision_summary: prepared.vision_summary,
        vision_error: prepared.vision_error,
        note: prepared.prompt_assist_note,
        assumptions: prepared.assumptions,
        focus_tags: prepared.focus_tags,
        used_original_prompt: prepared.used_original_prompt,
        resolution_label: context
            .request
            .settings
            .resolution_label_for(context.request.kind),
        estimated_frames,
        estimated_time,
        hardware_note: state.hardware_profile.note.clone(),
        reference_note: reference_summary.map(|summary| summary.note),
        selected_lora_name: context.selected_loras.first().map(|lora| lora.name.clone()),
        selected_lora_weight: context
            .request
            .normalized_lora_selections()
            .first()
            .and_then(|selection| selection.weight)
            .or(Some(1.0))
            .filter(|_| !context.selected_loras.is_empty()),
        selected_lora_labels: context
            .selected_loras
            .iter()
            .zip(context.request.normalized_lora_selections().iter())
            .map(|(lora, selection)| {
                format!(
                    "{} @ {:.2}",
                    lora.name,
                    selection.weight.unwrap_or(1.0)
                )
            })
            .collect(),
        supports_voice_output: context.model.supports_voice_output,
    }))
}

async fn start_generate(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> ApiResult<GenerateAccepted> {
    let context = resolve_generate_context(&state, request).await?;
    let job_id = Uuid::new_v4();
    let job_control = register_job(&state, job_id);
    let batch_total = context.request.normalized_batch_count();
    let accepted_seed = if batch_total == 1 {
        Some(u64::from(context.used_seed))
    } else {
        None
    };

    let task_state = state.clone();
    tokio::spawn(async move {
        if let Err(error) = run_generation_batch(
            task_state.clone(),
            job_id,
            job_control.clone(),
            context.model,
            context.media_generation_cloud_target,
            context.selected_loras,
            context.prompt_interpreter_model,
            context.vision_interpreter_model,
            context.request,
            context.reference_asset,
            context.end_reference_asset,
            context.control_reference_asset,
            context.used_seed,
        )
        .await {
            if is_cancellation_error(&error) {
                let _ = task_state.events.send(ServerEvent::Canceled {
                    job_id,
                    message: GENERATION_CANCELED_MESSAGE.to_string(),
                });
            } else {
                let _ = task_state.events.send(ServerEvent::Error {
                    job_id,
                    message: error.to_string(),
                });
                error!("{error:#}");
            }
        }
        unregister_job(&task_state, job_id);
    });

    Ok(Json(GenerateAccepted {
        job_id,
        used_seed: accepted_seed,
        batch_total,
    }))
}

async fn cancel_generate(
    State(state): State<AppState>,
    Json(request): Json<CancelRequest>,
) -> ApiResult<CancelAccepted> {
    let Some(control) = find_job_control(&state, request.job_id) else {
        return Err((
            StatusCode::NOT_FOUND,
            "That job is no longer active, so there is nothing to cancel.".to_string(),
        ));
    };

    control.canceled.store(true, Ordering::SeqCst);
    let _ = state.events.send(ServerEvent::Progress {
        job_id: request.job_id,
        percent: 0.97,
        phase: "Canceling".to_string(),
        message: "Cancel requested. Chatty-art is stopping the current run and aborting any queued batch items.".to_string(),
    });
    Ok(Json(CancelAccepted {
        job_id: request.job_id,
        accepted: true,
    }))
}

async fn websocket_endpoint(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(mut socket: WebSocket, state: AppState) {
    let mut receiver = state.events.subscribe();

    loop {
        tokio::select! {
            event = receiver.recv() => {
                match event {
                    Ok(event) => {
                        if let Ok(payload) = serde_json::to_string(&event) {
                            if socket.send(Message::Text(payload.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

async fn resolve_generate_context(
    state: &AppState,
    mut request: GenerateRequest,
) -> std::result::Result<ResolvedGenerateContext, (StatusCode, String)> {
    let has_audio_literal_prompt =
        request.kind == MediaKind::Audio && request.has_audio_literal_content();
    if request.prompt.trim().is_empty() && !has_audio_literal_prompt {
        return Err((
            StatusCode::BAD_REQUEST,
            "Prompt cannot be empty unless the audio Words / Script / Sounds area is filled in."
                .to_string(),
        ));
    }

    let media_generation_cloud_target =
        resolve_media_generation_cloud_target(state).await.map_err(internal_error)?;

    let models = scan_models(
        &state.paths.models_dir,
        &state.paths.diffuse_runtime_dir,
        &state.paths.audio_runtime_dir,
    )
    .map_err(internal_error)?;
    let model = if let Some(target) = media_generation_cloud_target.as_ref() {
        if !request.normalized_lora_selections().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "LoRA stacking is not available on the cloud media-generation lane."
                    .to_string(),
            ));
        }
        match request.kind {
            MediaKind::Image => {
                if !target.entry.capabilities.image_generation
                    || target.entry.image_generation_model_name.trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "The selected cloud media route does not have a still-image generation model configured."
                            .to_string(),
                    ));
                }
                if request.end_reference_asset.is_some() || request.control_reference_asset.is_some() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Cloud image generation does not support end-frame or control-video references yet."
                            .to_string(),
                    ));
                }
                ModelInfo {
                    id: format!("cloud:{}", target.entry.id),
                    name: format!(
                        "{} via {}",
                        target.entry.image_generation_model_name, target.entry.display_name
                    ),
                    slug: format!("cloud-{}", target.entry.id),
                    file_name: String::new(),
                    relative_path: String::new(),
                    family: "cloud-image".to_string(),
                    backend: ModelBackend::Cloud,
                    generation_style: request.style,
                    runtime_supported: true,
                    compatibility_note: "Cloud image generation route.".to_string(),
                    supported_kinds: vec![MediaKind::Image],
                    requires_reference: false,
                    supports_image_reference: false,
                    supports_reference_strength: false,
                    requires_end_image_reference: false,
                    supports_end_image_reference: false,
                    supports_video_reference: false,
                    supports_audio_reference: false,
                    supports_voice_output: false,
                    mmproj_path: None,
                }
            }
            MediaKind::Audio => {
                if !target.entry.capabilities.audio_generation
                    || target.entry.audio_generation_model_name.trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "The selected cloud media route does not have a speech-audio generation model configured."
                            .to_string(),
                    ));
                }
                if request.reference_asset.is_some()
                    || request.end_reference_asset.is_some()
                    || request.control_reference_asset.is_some()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Cloud speech generation does not support guide, end-frame, control-video, or voice-reference assets yet."
                            .to_string(),
                    ));
                }
                ModelInfo {
                    id: format!("cloud:{}", target.entry.id),
                    name: format!(
                        "{} via {}",
                        target.entry.audio_generation_model_name, target.entry.display_name
                    ),
                    slug: format!("cloud-{}", target.entry.id),
                    file_name: String::new(),
                    relative_path: String::new(),
                    family: "cloud-audio".to_string(),
                    backend: ModelBackend::Cloud,
                    generation_style: request.style,
                    runtime_supported: true,
                    compatibility_note: "Cloud speech-audio generation route.".to_string(),
                    supported_kinds: vec![MediaKind::Audio],
                    requires_reference: false,
                    supports_image_reference: false,
                    supports_reference_strength: false,
                    requires_end_image_reference: false,
                    supports_end_image_reference: false,
                    supports_video_reference: false,
                    supports_audio_reference: false,
                    supports_voice_output: true,
                    mmproj_path: None,
                }
            }
            MediaKind::Gif => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Cloud GIF generation is not enabled in Chatty-art yet. The current cloud video lane returns MP4 video only."
                        .to_string(),
                ));
            }
            MediaKind::Video => {
                if !target.entry.capabilities.video_generation
                    || target.entry.video_generation_model_name.trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "The selected cloud media route does not have a video generation model configured."
                            .to_string(),
                    ));
                }
                let (width, height) = request.settings.dimensions_for(request.kind);
                if !matches!(
                    (width, height),
                    (1280, 720)
                        | (720, 1280)
                        | (1792, 1024)
                        | (1024, 1792)
                        | (1920, 1080)
                        | (1080, 1920)
                ) {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Cloud video generation currently expects one of these supported sizes: 1280x720, 720x1280, 1792x1024, 1024x1792, 1920x1080, or 1080x1920."
                            .to_string(),
                    ));
                }
                if request.end_reference_asset.is_some() || request.control_reference_asset.is_some() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Cloud video generation currently supports only the main still-image guide reference. End-frame and control-video references stay local-only."
                            .to_string(),
                    ));
                }
                if matches!(target.entry.provider_kind, CloudProviderKind::Gemini) {
                    let requested_seconds = request.settings.video_duration_seconds;
                    if !matches!(requested_seconds, 4 | 6 | 8) {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            "Gemini cloud video requires a duration of 4, 6, or 8 seconds."
                                .to_string(),
                        ));
                    }
                    if request.reference_asset.is_some() && requested_seconds != 8 {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            "Gemini cloud video requires an 8-second duration when using a still-image guide reference."
                                .to_string(),
                        ));
                    }
                } else if matches!(target.entry.provider_kind, CloudProviderKind::OpenAi) {
                    validate_openai_video_request(
                        target.entry.video_generation_model_name.trim(),
                        &format!("{width}x{height}"),
                        request.settings.video_duration_seconds,
                    )
                    .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
                }
                ModelInfo {
                    id: format!("cloud:{}", target.entry.id),
                    name: format!(
                        "{} via {}",
                        target.entry.video_generation_model_name, target.entry.display_name
                    ),
                    slug: format!("cloud-{}", target.entry.id),
                    file_name: String::new(),
                    relative_path: String::new(),
                    family: "cloud-video".to_string(),
                    backend: ModelBackend::Cloud,
                    generation_style: request.style,
                    runtime_supported: true,
                    compatibility_note: if matches!(target.entry.provider_kind, CloudProviderKind::Gemini) {
                        "Cloud video generation route via Gemini's OpenAI-compatible Veo adapter.".to_string()
                    } else {
                        "Cloud video generation route via the current deprecated OpenAI Videos API path.".to_string()
                    },
                    supported_kinds: vec![MediaKind::Video],
                    requires_reference: false,
                    supports_image_reference: true,
                    supports_reference_strength: false,
                    requires_end_image_reference: false,
                    supports_end_image_reference: false,
                    supports_video_reference: false,
                    supports_audio_reference: false,
                    supports_voice_output: false,
                    mmproj_path: None,
                }
            }
        }
    } else {
        let model = models
            .iter()
            .find(|candidate| candidate.id == request.model || candidate.relative_path == request.model)
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Model '{}' was not found in models/.", request.model),
                )
            })?;

        if !model.runtime_supported {
            return Err((StatusCode::BAD_REQUEST, model.compatibility_note.clone()));
        }

        if model.generation_style != request.style {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "'{}' is a {:?} mode model, but the request asked for {:?} mode.",
                    model.name, model.generation_style, request.style
                ),
            ));
        }

        if !model.supported_kinds.contains(&request.kind) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "'{}' currently supports {} generation in Chatty-art.",
                    model.name,
                    model
                        .supported_kinds
                        .iter()
                        .map(|kind| format!("{:?}", kind).to_lowercase())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }

        model
    };

    let selected_loras = if !request.normalized_lora_selections().is_empty() {
        let loras = scan_loras(&state.paths.models_dir).map_err(internal_error)?;
        let mut resolved = Vec::new();

        if model.backend != ModelBackend::StableDiffusionCpp {
            return Err((
                StatusCode::BAD_REQUEST,
                "LoRA is currently only available for Realism models that use the stable-diffusion.cpp backend."
                    .to_string(),
            ));
        }

        for requested in request.normalized_lora_selections() {
            let lora = loras
                .iter()
                .find(|candidate| {
                    candidate.id == requested.id || candidate.relative_path == requested.id
                })
                .cloned()
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("LoRA '{}' was not found in models/loras/ or models/lora/.", requested.id),
                    )
                })?;

            if !lora.runtime_supported {
                return Err((StatusCode::BAD_REQUEST, lora.compatibility_note.clone()));
            }

            if !lora_matches_model(&lora, &model) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "'{}' is a {} LoRA, but '{}' expects {}-family LoRAs.",
                        lora.name,
                        lora.family,
                        model.name,
                        model_lora_family_label(&model)
                    ),
                ));
            }

            resolved.push(lora);
        }

        resolved
    } else {
        Vec::new()
    };

    let assets = scan_assets(&state.paths.input_dir, &state.paths.outputs_dir)
        .map_err(internal_error)?;
    let reference_asset = request.reference_asset.as_ref().and_then(|requested| {
        assets
            .iter()
            .find(|asset| asset.id == *requested || asset.relative_path == *requested)
            .cloned()
    });
    let end_reference_asset = request.end_reference_asset.as_ref().and_then(|requested| {
        assets
            .iter()
            .find(|asset| asset.id == *requested || asset.relative_path == *requested)
            .cloned()
    });
    let control_reference_asset = request
        .control_reference_asset
        .as_ref()
        .and_then(|requested| {
            assets
                .iter()
                .find(|asset| asset.id == *requested || asset.relative_path == *requested)
                .cloned()
        });

    if model.backend == ModelBackend::StableDiffusionCpp {
        if model.requires_reference && reference_asset.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "'{}' needs an image in the Input Tray before it can generate.",
                    model.name
                ),
            ));
        }

        if let Some(reference_asset) = reference_asset.as_ref() {
            if reference_asset.kind != MediaKind::Image {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Realism mode currently only accepts still-image references from the tray."
                        .to_string(),
                ));
            }

            if !model.supports_image_reference && !model.requires_reference {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "'{}' does not use reference images in Chatty-art yet.",
                        model.name
                    ),
                ));
            }
        }

        if model.requires_end_image_reference && end_reference_asset.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "'{}' needs an end image in the Input Tray before it can generate.",
                    model.name
                ),
            ));
        }

        if let Some(end_reference_asset) = end_reference_asset.as_ref() {
            if end_reference_asset.kind != MediaKind::Image {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Realism end-frame input must be a still image from the tray.".to_string(),
                ));
            }

            if !model.supports_end_image_reference && !model.requires_end_image_reference {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "'{}' does not use an end image in Chatty-art yet.",
                        model.name
                    ),
                ));
            }
        }

        if let Some(control_reference_asset) = control_reference_asset.as_ref() {
            if control_reference_asset.kind != MediaKind::Video
                && control_reference_asset.kind != MediaKind::Gif
            {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Realism control-video input must be a video or GIF from the tray."
                        .to_string(),
                ));
            }

            if !model.supports_video_reference {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "'{}' does not use control-video guidance in Chatty-art yet.",
                        model.name
                    ),
                ));
            }
        }
    }

    let used_seed = resolve_runtime_seed(request.settings.seed).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            format!("{message} Chatty-art currently supports seeds from 0 to {MAX_RUNTIME_SEED}."),
        )
    })?;
    request.settings.seed = Some(u64::from(used_seed));
    let prompt_interpreter_model =
        if request.prompt_assist != PromptAssistMode::Off && request.prepared_prompt.is_none() {
            if let Some(selected_prompt_model) = request
                .selected_prompt_model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                Some(
                    models
                        .iter()
                        .find(|candidate| {
                            candidate.id == selected_prompt_model
                                || candidate.relative_path == selected_prompt_model
                        })
                        .cloned()
                        .ok_or_else(|| {
                            (
                                StatusCode::BAD_REQUEST,
                                format!(
                                    "Prompt Assist model '{}' was not found in models/.",
                                    selected_prompt_model
                                ),
                            )
                        })
                        .and_then(|candidate| {
                            if supports_manual_prompt_interpreter(&candidate) {
                                Ok(candidate)
                            } else {
                                Err((
                                    StatusCode::BAD_REQUEST,
                                    format!(
                                        "'{}' is not ready to act as a Prompt Assist interpreter. Pick a local expressive llama.cpp model.",
                                        candidate.name
                                    ),
                                ))
                            }
                        })?,
                )
            } else {
                Some(
                    choose_prompt_interpreter_model(&models, &model).ok_or_else(|| {
                        (
                            StatusCode::BAD_REQUEST,
                            "Prompt Assist needs at least one local expressive llama.cpp model in models/."
                                .to_string(),
                        )
                    })?,
                )
            }
        } else {
            None
        };
    let vision_interpreter_model = if request.prompt_assist != PromptAssistMode::Off
        && request.prepared_prompt.is_none()
        && reference_asset.as_ref().is_some_and(|asset| asset.kind == MediaKind::Image)
    {
        if let Some(selected_vision_model) = request
            .selected_vision_model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(
                models
                    .iter()
                    .find(|candidate| {
                        candidate.id == selected_vision_model
                            || candidate.relative_path == selected_vision_model
                    })
                    .cloned()
                    .ok_or_else(|| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!(
                                "Vision Assist model '{}' was not found in models/.",
                                selected_vision_model
                            ),
                        )
                    })
                    .and_then(|candidate| {
                        if supports_vision_interpreter(&candidate) {
                            Ok(candidate)
                        } else {
                            Err((
                                StatusCode::BAD_REQUEST,
                                format!(
                                    "'{}' is not ready to act as a Vision Assist helper. Pick a local expressive multimodal model with an mmproj companion.",
                                    candidate.name
                                ),
                            ))
                        }
                    })?,
            )
        } else {
            choose_vision_interpreter_model(&models, &model, prompt_interpreter_model.as_ref())
        }
    } else {
        None
    };
    let vision_model_was_auto_selected = request
        .selected_vision_model
        .as_deref()
        .map(str::trim)
        .is_none_or(|value| value.is_empty())
        && vision_interpreter_model.is_some();

    Ok(ResolvedGenerateContext {
        request,
        model,
        media_generation_cloud_target,
        selected_loras,
        prompt_interpreter_model,
        vision_interpreter_model,
        vision_model_was_auto_selected,
        reference_asset,
        end_reference_asset,
        control_reference_asset,
        used_seed,
    })
}

fn estimate_audio_sequence_units(request: &GenerateRequest) -> f32 {
    let segments = request.normalized_audio_segments();
    if segments.is_empty() {
        return 1.0;
    }

    let mut previous_start = 0.0f32;
    let mut previous_end = 1.0f32;
    let mut total_end = 1.0f32;

    for (index, segment) in segments.iter().enumerate() {
        let start = if index == 0 {
            0.0
        } else if segment.same_time_as_previous {
            previous_start
        } else {
            previous_end
        };
        let end = start + 1.0;
        previous_start = start;
        previous_end = end;
        total_end = total_end.max(end);
    }

    total_end.max(1.0)
}

fn normalize_manual_prompt_items(values: &[String], limit: usize) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .iter()
        .map(|value| value.trim().trim_matches(',').to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.to_ascii_lowercase()))
        .take(limit)
        .collect()
}

fn merge_manual_prompt_items(base: &mut Vec<String>, extra: &[String], limit: usize) {
    let mut seen = base
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();

    for value in extra {
        if base.len() >= limit {
            break;
        }

        let key = value.to_ascii_lowercase();
        if seen.insert(key) {
            base.push(value.clone());
        }
    }
}

fn append_manual_prompt_note(
    base: &str,
    assumptions: &[String],
    focus_tags: &[String],
    preserve_items: &[String],
    change_targets: &[String],
    avoid_items: &[String],
) -> String {
    if assumptions.is_empty()
        && focus_tags.is_empty()
        && preserve_items.is_empty()
        && change_targets.is_empty()
        && avoid_items.is_empty()
    {
        return base.to_string();
    }

    let mut additions = Vec::new();
    if !focus_tags.is_empty() {
        additions.push(format!("Manual focus cues: {}", focus_tags.join(", ")));
    }
    if !assumptions.is_empty() {
        additions.push(format!(
            "Manual assumptions: {}",
            assumptions.join(", ")
        ));
    }
    if !preserve_items.is_empty() {
        additions.push(format!(
            "Manual keep / preserve: {}",
            preserve_items.join(", ")
        ));
    }
    if !change_targets.is_empty() {
        additions.push(format!(
            "Manual change targets: {}",
            change_targets.join(", ")
        ));
    }
    if !avoid_items.is_empty() {
        additions.push(format!("Manual avoid: {}", avoid_items.join(", ")));
    }

    let addition = format!("Added to the handoff. {}", additions.join(" "));
    if base.trim().is_empty() {
        addition
    } else {
        format!("{} {}", base.trim(), addition)
    }
}

async fn build_prompt_handoff(
    paths: &AppPaths,
    request: &GenerateRequest,
    model: &ModelInfo,
    prompt_interpreter_model: Option<&ModelInfo>,
    vision_interpreter_model: Option<&ModelInfo>,
    vision_model_was_auto_selected: bool,
    reference_summary: Option<&ReferenceSummary>,
    used_seed: u32,
    prompt_assist_cloud: Option<&PromptAssistCloudTarget>,
    vision_assist_cloud: Option<&VisionAssistCloudTarget>,
    reference_image_path: Option<&Path>,
) -> Result<PreparedPromptState> {
    let mut effective_request = request.clone();
    let manual_focus_tags = normalize_manual_prompt_items(&request.manual_focus_tags, 10);
    let manual_assumptions = normalize_manual_prompt_items(&request.manual_assumptions, 6);
    let manual_preserve_items = normalize_manual_prompt_items(&request.manual_preserve_items, 8);
    let manual_change_targets = normalize_manual_prompt_items(&request.manual_change_targets, 8);
    let manual_avoid_items = normalize_manual_prompt_items(&request.manual_avoid_items, 8);
    let is_speech_audio = request.kind == MediaKind::Audio && is_speech_generation_model(model);
    let is_sound_audio = request.kind == MediaKind::Audio
        && model.backend == ModelBackend::AudioRuntime
        && !model.supports_voice_output;
    let prepared_prompt = request
        .prepared_prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let prepared_spoken_text = request
        .prepared_spoken_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let literal_audio_prompt_value = request.combined_audio_literal_prompt();
    let literal_audio_prompt = literal_audio_prompt_value.as_deref();

    if prepared_prompt.is_some() || (is_speech_audio && prepared_spoken_text.is_some()) {
        if let Some(prepared_prompt) = prepared_prompt {
            effective_request.prompt = prepared_prompt.to_string();
        }
        effective_request.negative_prompt = request
            .prepared_negative_prompt
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let spoken_text = if is_speech_audio {
            prepared_spoken_text
                .map(str::to_string)
                .or_else(|| literal_audio_prompt.map(str::to_string))
                .or_else(|| Some(derive_spoken_text_heuristic(&request.prompt)))
        } else {
            None
        };
        effective_request.prepared_spoken_text = spoken_text.clone();
        if is_speech_audio && effective_request.prompt.trim().is_empty() {
            if let Some(direction) = derive_speech_direction_heuristic(
                &request.prompt,
                spoken_text.as_deref().or(literal_audio_prompt),
            ) {
                effective_request.prompt = direction;
            }
        }
        return Ok(PreparedPromptState {
            effective_request,
            prompt_assist_note: append_manual_prompt_note(
                &request.prepared_note.clone().unwrap_or_else(|| {
                    if is_speech_audio {
                        "Preview Handoff was reviewed before generation. Only the Spoken Text field will be voiced."
                            .to_string()
                    } else {
                        "Preview Handoff was reviewed before generation.".to_string()
                    }
                }),
                &manual_assumptions,
                &manual_focus_tags,
                &manual_preserve_items,
                &manual_change_targets,
                &manual_avoid_items,
            ),
            compiled_prompt: request
                .prepared_prompt
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            prepared_spoken_text: spoken_text,
            interpreter_model_name: request.prepared_interpreter_model.clone(),
            vision_model_name: None,
            vision_summary: None,
            vision_error: None,
            assumptions: manual_assumptions,
            focus_tags: manual_focus_tags,
            used_original_prompt: false,
            prompt_assist_sidecar: None,
        });
    }

    let use_literal_audio_directly =
        request.prompt.trim().is_empty() && literal_audio_prompt.is_some();

    if request.prompt_assist == PromptAssistMode::Off || use_literal_audio_directly {
        if is_speech_audio {
            let spoken_text = literal_audio_prompt
                .map(str::to_string)
                .unwrap_or_else(|| derive_spoken_text_heuristic(&request.prompt));
            let direction = derive_speech_direction_heuristic(&request.prompt, Some(&spoken_text));
            effective_request.prepared_spoken_text = Some(spoken_text.clone());
            if let Some(direction_text) = direction.as_deref() {
                effective_request.prompt = direction_text.to_string();
            }
            if !manual_focus_tags.is_empty() {
                effective_request.prompt = crate::runtime::polish_manual_prompt_handoff(
                    &effective_request.prompt,
                    &manual_focus_tags,
                    request.style,
                    request.kind,
                );
            }
            return Ok(PreparedPromptState {
                effective_request,
                prompt_assist_note: append_manual_prompt_note(if use_literal_audio_directly {
                    "Using the Words / Script field as the verbatim spoken line. Add prompt text above if you want extra delivery direction."
                        .to_string()
                } else {
                    "Speech handoff separated the words to be spoken from the delivery description. Only the Spoken Text field will be voiced.".to_string()
                }
                .as_str(),
                &manual_assumptions,
                &manual_focus_tags,
                &manual_preserve_items,
                &manual_change_targets,
                &manual_avoid_items,
                ),
                compiled_prompt: direction,
                prepared_spoken_text: Some(spoken_text),
                interpreter_model_name: None,
                vision_model_name: None,
                vision_summary: None,
                vision_error: None,
                assumptions: manual_assumptions,
                focus_tags: manual_focus_tags,
                used_original_prompt: true,
                prompt_assist_sidecar: None,
            });
        }
        if !manual_focus_tags.is_empty() {
            effective_request.prompt = crate::runtime::polish_manual_prompt_handoff(
                &effective_request.prompt,
                &manual_focus_tags,
                request.style,
                request.kind,
            );
        }
        return Ok(PreparedPromptState {
            effective_request,
            prompt_assist_note: append_manual_prompt_note(&(if is_sound_audio && literal_audio_prompt.is_some() {
                if use_literal_audio_directly {
                    "Using the Words / Sounds field as the verbatim sound lane. Add prompt text above if you want extra texture or scene description."
                        .to_string()
                } else {
                    "Prompt Assist stayed out of the Words / Sounds lane. Literal sound cues will be passed through verbatim while the main prompt remains descriptive."
                        .to_string()
                }
            } else {
                String::new()
            }),
            &manual_assumptions,
            &manual_focus_tags,
            &manual_preserve_items,
            &manual_change_targets,
            &manual_avoid_items,
            ),
            compiled_prompt: None,
            prepared_spoken_text: None,
            interpreter_model_name: None,
            vision_model_name: None,
            vision_summary: None,
            vision_error: None,
            assumptions: manual_assumptions,
            focus_tags: manual_focus_tags,
            used_original_prompt: true,
            prompt_assist_sidecar: None,
        });
    }

    let (mut compiled, interpreter_label) = if let Some(cloud_target) = prompt_assist_cloud {
        (
            compile_prompt_cloud(
                &cloud_target.entry,
                &cloud_target.api_key,
                &paths.runtime_dir,
                &paths.models_dir,
                vision_interpreter_model,
                vision_assist_cloud.map(|target| &target.entry),
                vision_assist_cloud.map(|target| target.api_key.as_str()),
                vision_model_was_auto_selected,
                &request.prompt,
                request.negative_prompt.as_deref(),
                &manual_preserve_items,
                &manual_change_targets,
                &manual_avoid_items,
                request.style,
                request.kind,
                request.prompt_assist,
                reference_summary,
                reference_image_path,
                model.supports_voice_output,
                used_seed,
            )
            .await?,
            format!(
                "{} (cloud:{})",
                cloud_target.entry.prompt_assist_model_name,
                cloud_target.entry.display_name
            ),
        )
    } else {
        let interpreter_model = prompt_interpreter_model.ok_or_else(|| {
            anyhow::anyhow!(
                "Prompt Assist needs at least one local expressive llama.cpp model in models/."
            )
        })?;
        (
            compile_prompt(
                &paths.runtime_dir,
                &paths.models_dir,
                interpreter_model,
                vision_interpreter_model,
                vision_model_was_auto_selected,
                &request.prompt,
                request.negative_prompt.as_deref(),
                &manual_preserve_items,
                &manual_change_targets,
                &manual_avoid_items,
                request.style,
                request.kind,
                request.prompt_assist,
                reference_summary,
                model.supports_voice_output,
                used_seed,
            )
            .await?,
            interpreter_model.name.clone(),
        )
    };

    merge_manual_prompt_items(&mut compiled.brief.assumptions, &manual_assumptions, 6);
    merge_manual_prompt_items(&mut compiled.brief.focus_tags, &manual_focus_tags, 10);
    let assumptions = compiled.brief.assumptions.clone();
    let focus_tags = compiled.brief.focus_tags.clone();
    let spoken_text = if is_speech_audio {
        compiled
            .spoken_text
            .clone()
            .or_else(|| literal_audio_prompt.map(str::to_string))
            .or_else(|| Some(derive_spoken_text_heuristic(&request.prompt)))
    } else {
        None
    };
    let compiled_prompt = if is_speech_audio {
        if compiled.prompt.trim().is_empty() {
            derive_speech_direction_heuristic(&request.prompt, spoken_text.as_deref())
                .unwrap_or_default()
        } else {
            compiled.prompt.clone()
        }
    } else {
        compiled.prompt.clone()
    };
    let compiled_prompt = crate::runtime::polish_manual_prompt_handoff(
        &compiled_prompt,
        &focus_tags,
        request.style,
        request.kind,
    );
    effective_request.prepared_spoken_text = spoken_text.clone();
    effective_request.prompt = if compiled_prompt.trim().is_empty() {
        request.prompt.clone()
    } else {
        compiled_prompt.clone()
    };
    if effective_request.style == GenerationStyle::Realism {
        effective_request.negative_prompt = compiled.negative_prompt.clone();
    } else if effective_request.negative_prompt.is_none() {
        effective_request.negative_prompt = compiled.negative_prompt.clone();
    }

    Ok(PreparedPromptState {
        effective_request,
        prompt_assist_note: append_manual_prompt_note(
            &compiled.note,
            &manual_assumptions,
            &manual_focus_tags,
            &manual_preserve_items,
            &manual_change_targets,
            &manual_avoid_items,
        ),
        compiled_prompt: (!compiled_prompt.trim().is_empty()).then_some(compiled_prompt.clone()),
        prepared_spoken_text: spoken_text.clone(),
        interpreter_model_name: Some(interpreter_label.clone()),
        vision_model_name: compiled.vision_model_name.clone(),
        vision_summary: compiled.vision_summary.clone(),
        vision_error: compiled.vision_error.clone(),
        assumptions: assumptions.clone(),
        focus_tags: focus_tags.clone(),
        used_original_prompt: compiled.used_original_prompt,
        prompt_assist_sidecar: Some(PromptAssistTraceSidecar {
            job_id: Uuid::nil(),
            kind: request.kind,
            style: request.style,
            assist_mode: request.prompt_assist,
            generation_model: model.name.clone(),
            prompt_assist_route: Some(if interpreter_label.contains("(cloud:") {
                "cloud".to_string()
            } else {
                "local".to_string()
            }),
            vision_assist_route: compiled.vision_model_name.as_deref().map(|value| {
                if value.contains("(cloud:") {
                    "cloud".to_string()
                } else {
                    "local".to_string()
                }
            }),
            output_route: if model.backend == ModelBackend::Cloud {
                "cloud".to_string()
            } else {
                "local".to_string()
            },
            interpreter_model: interpreter_label,
            original_prompt: request.prompt.clone(),
            compiled_prompt,
            spoken_text,
            negative_prompt: compiled.negative_prompt,
            note: compiled.note,
            used_original_prompt: compiled.used_original_prompt,
            assumptions: assumptions.clone(),
            focus_tags: focus_tags.clone(),
            extracted_json: compiled.trace.extracted_json,
            raw_output: compiled.trace.raw_output,
            stderr: compiled.trace.stderr,
        }),
    })
}

fn estimate_generation_time(
    model: &ModelInfo,
    request: &GenerateRequest,
    hardware: &HardwareProfile,
) -> TimeEstimate {
    let settings = &request.settings;
    let (width, height) = settings.dimensions_for(request.kind);
    let pixel_scale = ((width * height) as f32 / (512.0 * 512.0)).max(0.35);
    let step_scale = (settings.steps as f32 / 24.0).clamp(0.35, 4.5);
    let frame_count = settings.video_frame_count().max(1) as f32;
    let audio_duration = settings.audio_duration_seconds.max(1) as f32;
    let audio_sequence_units = estimate_audio_sequence_units(request);
    let audio_segment_count = request.normalized_audio_segments().len().max(1) as f32;
    let combined_audio_literal_prompt = request.combined_audio_literal_prompt();
    let speech_source = request
        .prepared_spoken_text
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| combined_audio_literal_prompt.as_deref())
        .unwrap_or(&request.prompt);
    let prompt_word_count = speech_source.split_whitespace().count().max(1) as f32;
    let low_vram_scale = if settings.low_vram_mode { 1.35 } else { 1.0 };
    let dedicated_vram = hardware.dedicated_vram_gb.unwrap_or(8.0).max(1.0);
    let vram_pressure_scale = match request.kind {
        MediaKind::Image => {
            if dedicated_vram >= 10.0 {
                0.9
            } else {
                (10.0 / dedicated_vram).min(1.8)
            }
        }
        MediaKind::Audio => 1.0,
        MediaKind::Gif | MediaKind::Video => {
            if dedicated_vram >= 12.0 {
                0.9
            } else {
                (12.0 / dedicated_vram).min(2.4)
            }
        }
    };

    let family = model.family.to_ascii_lowercase();
    let backend_scale = match model.backend {
        ModelBackend::LlamaCpp => match request.kind {
            MediaKind::Image => 1.0,
            MediaKind::Gif => 1.1,
            MediaKind::Video => 1.4,
            MediaKind::Audio => 1.2,
        },
        ModelBackend::StableDiffusionCpp => {
            if family.contains("wan") {
                match request.kind {
                    MediaKind::Image => 1.2,
                    MediaKind::Gif => 1.8,
                    MediaKind::Video => 2.2,
                    MediaKind::Audio => 1.0,
                }
            } else if family.contains("flux") {
                1.5
            } else {
                1.0
            }
        }
        ModelBackend::AudioRuntime => 1.3,
        ModelBackend::Cloud => 1.0,
    };

    let model_size_scale = match model.backend {
        ModelBackend::LlamaCpp => {
            let hinted = parameter_hint(&model.name) as f32;
            (hinted / 80.0).clamp(0.8, 4.0)
        }
        ModelBackend::StableDiffusionCpp | ModelBackend::AudioRuntime | ModelBackend::Cloud => 1.0,
    };

    let base_seconds = match (model.backend, request.kind) {
        (ModelBackend::LlamaCpp, MediaKind::Image) => {
            8.0 * step_scale * pixel_scale * model_size_scale
        }
        (ModelBackend::LlamaCpp, MediaKind::Gif) => {
            10.0 + frame_count * 0.22 * pixel_scale * step_scale * model_size_scale
        }
        (ModelBackend::LlamaCpp, MediaKind::Video) => {
            18.0 + frame_count * 0.35 * pixel_scale * step_scale * model_size_scale
        }
        (ModelBackend::LlamaCpp, MediaKind::Audio) => {
            (8.0 + prompt_word_count * 0.12) * step_scale * model_size_scale
        }
        (ModelBackend::StableDiffusionCpp, MediaKind::Image) => {
            16.0 * pixel_scale * step_scale * backend_scale
        }
        (ModelBackend::StableDiffusionCpp, MediaKind::Gif) => {
            12.0 + frame_count * 0.95 * pixel_scale * step_scale * backend_scale
        }
        (ModelBackend::StableDiffusionCpp, MediaKind::Video) => {
            18.0 + frame_count * 1.15 * pixel_scale * step_scale * backend_scale
        }
        (ModelBackend::StableDiffusionCpp, MediaKind::Audio) => 1.0,
        (ModelBackend::AudioRuntime, MediaKind::Audio) => {
            if model.supports_voice_output {
                let reference_scale = if request.reference_asset.is_some() {
                    1.4
                } else {
                    1.0
                };
                (10.0 + prompt_word_count * 0.22)
                    * reference_scale
                    * backend_scale
                    * audio_segment_count.max(1.0)
            } else {
                (12.0 + (audio_duration * audio_sequence_units) * 4.5)
                    * step_scale
                    * backend_scale
            }
        }
        (ModelBackend::AudioRuntime, MediaKind::Image | MediaKind::Gif | MediaKind::Video) => 1.0,
        (ModelBackend::Cloud, MediaKind::Image) => 28.0,
        (ModelBackend::Cloud, MediaKind::Audio) => 10.0 + prompt_word_count * 0.18,
        (ModelBackend::Cloud, MediaKind::Gif) => 1.0,
        (ModelBackend::Cloud, MediaKind::Video) => {
            let duration_scale = (settings.video_duration_seconds.max(2) as f32 / 8.0).clamp(0.6, 2.5);
            let size_scale = match settings.video_resolution.label() {
                "1280x720" | "720x1280" => 1.0,
                "1792x1024" | "1024x1792" => 1.35,
                "1920x1080" | "1080x1920" => 1.7,
                _ => 1.2,
            };
            95.0 * duration_scale * size_scale
        }
    } * vram_pressure_scale
        * low_vram_scale;

    let spread = match (model.backend, request.kind) {
        (ModelBackend::LlamaCpp, MediaKind::Image | MediaKind::Audio) => 0.20,
        (ModelBackend::LlamaCpp, _) => 0.30,
        (ModelBackend::StableDiffusionCpp, MediaKind::Image) => 0.30,
        (ModelBackend::StableDiffusionCpp, MediaKind::Gif) => 0.45,
        (ModelBackend::StableDiffusionCpp, MediaKind::Video) => 0.60,
        (ModelBackend::StableDiffusionCpp, MediaKind::Audio) => 0.20,
        (ModelBackend::AudioRuntime, MediaKind::Audio) => 0.45,
        (ModelBackend::AudioRuntime, _) => 0.20,
        (ModelBackend::Cloud, MediaKind::Image) => 0.55,
        (ModelBackend::Cloud, _) => 0.25,
    };

    let min_seconds = base_seconds.max(3.0).round() as u32;
    let max_seconds = (base_seconds * (1.0 + spread))
        .max(min_seconds as f32 + 2.0)
        .round() as u32;
    let confidence = match (model.backend, request.kind) {
        (ModelBackend::LlamaCpp, MediaKind::Image | MediaKind::Audio) => EstimateConfidence::High,
        (ModelBackend::LlamaCpp, _) => EstimateConfidence::Medium,
        (ModelBackend::StableDiffusionCpp, MediaKind::Image) => EstimateConfidence::Medium,
        (ModelBackend::StableDiffusionCpp, MediaKind::Gif | MediaKind::Video) => {
            EstimateConfidence::Low
        }
        (ModelBackend::StableDiffusionCpp, MediaKind::Audio) => EstimateConfidence::Low,
        (ModelBackend::AudioRuntime, MediaKind::Audio) => {
            if model.supports_voice_output {
                EstimateConfidence::Medium
            } else {
                EstimateConfidence::Low
            }
        }
        (ModelBackend::AudioRuntime, _) => EstimateConfidence::Low,
        (ModelBackend::Cloud, _) => EstimateConfidence::Low,
    };
    let note = match request.kind {
        MediaKind::Gif | MediaKind::Video => {
            if model.backend == ModelBackend::Cloud {
                if model.compatibility_note.contains("Gemini") {
                    format!(
                        "{} at {} via {}. Cloud video timings vary heavily by provider queue depth and policy checks on the Gemini Veo route.",
                        settings.video_duration_seconds,
                        settings.video_resolution.label(),
                        model.name
                    )
                } else {
                    format!(
                        "{} at {} via {}. Cloud video timings vary heavily by provider queue depth, policy checks, and the deprecated OpenAI Videos API path.",
                        settings.video_duration_seconds,
                        settings.video_resolution.label(),
                        model.name
                    )
                }
            } else {
                format!(
                    "{} frames at {} with {}. Video estimates vary most on local hardware.",
                    settings.video_frame_count(),
                    settings.video_resolution.label(),
                    model.name
                )
            }
        }
        MediaKind::Image => {
            if model.backend == ModelBackend::Cloud {
                format!(
                    "{} routed to {}. Cloud image timings vary by provider load and policy checks.",
                    settings.resolution.label(),
                    model.name
                )
            } else {
                format!(
                    "{} at {} steps on {}.",
                    settings.resolution.label(),
                    settings.steps,
                    model.name
                )
            }
        }
        MediaKind::Audio => {
            if model.backend == ModelBackend::AudioRuntime && model.supports_voice_output {
                format!(
                    "Speech estimate based on the spoken text length for {} across {} segment(s). Voice-reference cloning, if used, will slow it down.",
                    model.name,
                    audio_segment_count as usize
                )
            } else if model.backend == ModelBackend::AudioRuntime {
                format!(
                    "Soundscape estimate based on a {}s target clip at {} steps for {} across roughly {:.1} timing unit(s).",
                    settings.audio_duration_seconds.max(1),
                    settings.steps,
                    model.name,
                    audio_sequence_units
                )
            } else if model.backend == ModelBackend::Cloud {
                format!(
                    "Cloud speech estimate is based mainly on spoken text length for {}. Provider load and moderation checks can move this around.",
                    model.name
                )
            } else {
                format!(
                    "Audio estimate based on the planned expressive audio path for {}.",
                    model.name
                )
            }
        }
    };

    TimeEstimate {
        min_seconds,
        max_seconds,
        confidence,
        note,
    }
}

async fn run_generation_job(
    state: AppState,
    job_id: Uuid,
    job_control: Arc<JobControl>,
    model: ModelInfo,
    media_generation_cloud_target: Option<MediaGenerationCloudTarget>,
    selected_loras: Vec<LoraInfo>,
    prompt_interpreter_model: Option<ModelInfo>,
    vision_interpreter_model: Option<ModelInfo>,
    request: GenerateRequest,
    reference_asset: Option<InputAsset>,
    end_reference_asset: Option<InputAsset>,
    control_reference_asset: Option<InputAsset>,
) -> Result<()> {
    ensure_not_canceled(&job_control)?;
    emit_progress(
        &state,
        job_id,
        0.03,
        "Queued",
        if media_generation_cloud_target.is_some() {
            "Waiting for the cloud generation slot."
        } else {
            "Waiting for the local generator slot."
        },
    );

    let _permit = state
        .generation_gate
        .clone()
        .acquire_owned()
        .await
        .context("generation gate was closed")?;
    ensure_not_canceled(&job_control)?;

    emit_progress(
        &state,
        job_id,
        0.12,
        "Preparing",
        if media_generation_cloud_target.is_some() {
            "Checking folders, reference media, and the selected cloud output lane."
        } else {
            "Checking folders, reference media, and selected model."
        },
    );

    let used_seed =
        resolve_runtime_seed(request.settings.seed).map_err(|message| anyhow::anyhow!(message))?;
    let reference_summary = match reference_asset.as_ref() {
        Some(asset) => Some(
            build_reference_summary(
                &asset.disk_path(&state.paths.input_dir, &state.paths.outputs_dir),
                asset,
                request.reference_intent,
            )
            .await?,
        ),
        None => None,
    };
    let mut effective_request = request.clone();
    let mut prompt_assist_note = String::new();
    let mut compiled_prompt: Option<String> = None;
    let mut interpreter_model_name: Option<String> = None;
    let mut vision_model_name: Option<String> = None;
    let mut prompt_assist_sidecar: Option<PromptAssistTraceSidecar> = None;
    let reference_image_path = reference_asset
        .as_ref()
        .filter(|asset| asset.kind == MediaKind::Image)
        .map(|asset| asset.disk_path(&state.paths.input_dir, &state.paths.outputs_dir));

    if request.prompt_assist != PromptAssistMode::Off
        || request.prepared_prompt.is_some()
        || request.prepared_spoken_text.is_some()
        || request.has_audio_literal_content()
        || (request.kind == MediaKind::Audio && is_speech_generation_model(&model))
    {
        if request.prepared_prompt.is_some() {
            emit_progress(
                &state,
                job_id,
                0.2,
                "Locked In",
                "Using the reviewed handoff preview for this generation run.",
            );
        } else {
            emit_progress(
                &state,
                job_id,
                0.2,
                "Interpreting",
                "Prompt Assist is expanding the request into a richer local brief.",
            );
        }

        let heartbeat = if request.prepared_prompt.is_some() {
            None
        } else {
            Some(spawn_progress_heartbeat(
                state.clone(),
                job_id,
                0.22,
                0.32,
                "Interpreting",
                "Still compiling the prompt locally. This stage is filling in sensible missing details.",
            ))
        };

        let prepared_result = build_prompt_handoff(
            &state.paths,
            &request,
            &model,
            prompt_interpreter_model.as_ref(),
            vision_interpreter_model.as_ref(),
            request
                .selected_vision_model
                .as_deref()
                .map(str::trim)
                .is_none_or(|value| value.is_empty())
                && vision_interpreter_model.is_some(),
            reference_summary.as_ref(),
            used_seed,
            resolve_prompt_assist_cloud_target(&state).await?.as_ref(),
            resolve_vision_assist_cloud_target(&state).await?.as_ref(),
            reference_image_path.as_deref(),
        )
        .await;
        if let Some(stop) = heartbeat {
            let _ = stop.send(());
        }
        let prepared = prepared_result?;
        ensure_not_canceled(&job_control)?;
        prompt_assist_note = prepared.prompt_assist_note;
        effective_request = prepared.effective_request;
        interpreter_model_name = prepared.interpreter_model_name;
        vision_model_name = prepared.vision_model_name;
        compiled_prompt = prepared.compiled_prompt;
        prompt_assist_sidecar = prepared.prompt_assist_sidecar.map(|mut trace| {
            trace.job_id = job_id;
            trace
        });
    }

    let kind_dir = state.paths.outputs_dir.join(request.kind.as_str());
    tokio::fs::create_dir_all(&kind_dir).await?;
    let (file_name, relative_path, output_path, mime, generation_note, planner_trace) = match model
        .backend
    {
        ModelBackend::LlamaCpp => {
            emit_progress(
                &state,
                job_id,
                0.36,
                "Planning",
                "Using llama.cpp with the bundled Vulkan runtime to plan the generation.",
            );

            let (file_name, relative_path, output_path) =
                build_output_path(&kind_dir, request.kind, &model.name, used_seed);
            let (planner_note, planner_trace) = match request.kind {
                MediaKind::Image => {
                    let heartbeat = spawn_progress_heartbeat(
                        state.clone(),
                        job_id,
                        0.4,
                        0.68,
                        "Planning",
                        "Still planning with the selected local model. Larger GGUFs can take a few minutes here.",
                    );
                    let plan_result = build_image_plan(
                        &state.paths.runtime_dir,
                        &state.paths.models_dir,
                        &model,
                        &effective_request.prompt,
                        &effective_request.settings,
                        used_seed,
                        reference_summary.as_ref(),
                    )
                    .await;
                    let _ = heartbeat.send(());
                    let planned = plan_result?;
                    ensure_not_canceled(&job_control)?;
                    let note = planned.note.clone();
                    let trace = planned.trace.clone();

                    emit_progress(
                        &state,
                        job_id,
                        0.72,
                        "Rendering",
                        "Painting the image and preparing the inline preview.",
                    );

                    let render_path = output_path.clone();
                    let render_settings = effective_request.settings.clone();
                    let render_reference = reference_summary.clone();
                    tokio::task::spawn_blocking(move || {
                        render_image(
                            &render_path,
                            &planned.plan,
                            &render_settings,
                            render_reference.as_ref(),
                        )
                    })
                    .await
                    .context("image render task failed to join")??;
                    (note, trace)
                }
                MediaKind::Gif => {
                    let heartbeat = spawn_progress_heartbeat(
                        state.clone(),
                        job_id,
                        0.4,
                        0.68,
                        "Planning",
                        "Still planning with the selected local model. Larger GGUFs can take a few minutes here.",
                    );
                    let plan_result = build_video_plan(
                        &state.paths.runtime_dir,
                        &state.paths.models_dir,
                        &model,
                        &effective_request.prompt,
                        &effective_request.settings,
                        used_seed,
                        reference_summary.as_ref(),
                    )
                    .await;
                    let _ = heartbeat.send(());
                    let planned = plan_result?;
                    ensure_not_canceled(&job_control)?;
                    let note = planned.note.clone();
                    let trace = planned.trace.clone();

                    emit_progress(
                        &state,
                        job_id,
                        0.72,
                        "Rendering",
                        "Building the animated GIF preview and saving the loop.",
                    );

                    let render_path = output_path.clone();
                    let render_settings = effective_request.settings.clone();
                    let render_reference = reference_summary.clone();
                    tokio::task::spawn_blocking(move || {
                        render_video(
                            &render_path,
                            &planned.plan,
                            &render_settings,
                            render_reference.as_ref(),
                        )
                    })
                    .await
                    .context("gif render task failed to join")??;
                    (note, trace)
                }
                MediaKind::Video => {
                    return Err(anyhow::anyhow!(
                        "The expressive backend currently creates animated GIFs, not true video files. Pick Generate GIF for this model."
                    ));
                }
                MediaKind::Audio => {
                    let heartbeat = spawn_progress_heartbeat(
                        state.clone(),
                        job_id,
                        0.4,
                        0.68,
                        "Planning",
                        "Still planning with the selected local model. Larger GGUFs can take a few minutes here.",
                    );
                    let plan_result = build_audio_plan(
                        &state.paths.runtime_dir,
                        &state.paths.models_dir,
                        &model,
                        &effective_request.prompt,
                        &effective_request.settings,
                        used_seed,
                        reference_summary.as_ref(),
                    )
                    .await;
                    let _ = heartbeat.send(());
                    let planned = plan_result?;
                    ensure_not_canceled(&job_control)?;
                    let note = planned.note.clone();
                    let trace = planned.trace.clone();

                    emit_progress(
                        &state,
                        job_id,
                        0.72,
                        "Rendering",
                        "Synthesizing the WAV output and preparing playback.",
                    );

                    let render_path = output_path.clone();
                    tokio::task::spawn_blocking(move || render_audio(&render_path, &planned.plan))
                        .await
                        .context("audio render task failed to join")??;
                    (note, trace)
                }
            };

            (
                file_name,
                relative_path,
                output_path,
                request.kind.output_mime().to_string(),
                planner_note,
                Some(planner_trace),
            )
        }
        ModelBackend::StableDiffusionCpp => {
            emit_progress(
                &state,
                job_id,
                0.34,
                "Realism",
                "Using the local stable-diffusion.cpp backend. The first realism run may build sd-cli.",
            );

            let output_extension = match request.kind {
                MediaKind::Image => "png",
                MediaKind::Gif => "gif",
                MediaKind::Video => "mp4",
                MediaKind::Audio => {
                    return Err(anyhow::anyhow!(
                        "stable-diffusion.cpp does not generate audio in Chatty-art."
                    ));
                }
            };
            let (file_name, relative_path, output_path) = build_output_path_with_extension(
                &kind_dir,
                request.kind,
                &model.name,
                used_seed,
                output_extension,
            );

            let generated = generate_with_sdcpp(
                &state.paths.diffuse_runtime_dir,
                &state.paths.models_dir,
                &state.paths.input_dir,
                &state.paths.outputs_dir,
                &model,
                &effective_request,
                reference_asset.as_ref(),
                end_reference_asset.as_ref(),
                control_reference_asset.as_ref(),
                used_seed,
                &output_path,
                &job_control.canceled,
            )
            .await?;

            emit_progress(
                &state,
                job_id,
                0.78,
                "Finishing",
                "Saving the local stable-diffusion.cpp result and preparing the preview.",
            );

            (
                file_name,
                relative_path,
                output_path,
                generated.mime,
                generated.note,
                None,
            )
        }
        ModelBackend::AudioRuntime => {
            emit_progress(
                &state,
                job_id,
                0.34,
                "Realism Audio",
                "Using the local realism audio runtime for speech generation.",
            );

            let (file_name, relative_path, output_path) =
                build_output_path(&kind_dir, request.kind, &model.name, used_seed);

            let generated = generate_with_audio_runtime(
                &state.paths.audio_runtime_dir,
                &state.paths.models_dir,
                &state.paths.input_dir,
                &state.paths.outputs_dir,
                &model,
                &effective_request,
                reference_asset.as_ref(),
                used_seed,
                &output_path,
                &job_control.canceled,
            )
            .await?;

            emit_progress(
                &state,
                job_id,
                0.78,
                "Finishing",
                "Saving the realism audio output and preparing playback.",
            );

            (
                file_name,
                relative_path,
                output_path,
                generated.mime,
                generated.note,
                None,
            )
        }
        ModelBackend::Cloud => {
            let cloud_target = media_generation_cloud_target
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing cloud media target for cloud generation job."))?;
            emit_progress(
                &state,
                job_id,
                0.38,
                "Dispatching",
                match request.kind {
                    MediaKind::Audio => {
                        "Sending the spoken text and delivery notes to the selected cloud speech provider."
                    }
                    MediaKind::Video => {
                        "Submitting the prepared video prompt to the selected cloud video provider and waiting for the render job to finish."
                    }
                    MediaKind::Image | MediaKind::Gif => {
                        "Sending the prepared image prompt to the selected cloud image provider."
                    }
                },
            );

            let (file_name, relative_path, output_path) =
                build_output_path(&kind_dir, request.kind, &model.name, used_seed);
            match request.kind {
                MediaKind::Image => {
                    let (width, height) = effective_request.settings.dimensions_for(request.kind);
                    let size = format!("{width}x{height}");
                    let result = await_cancelable(
                        &job_control,
                        generate_cloud_image(
                            &cloud_target.entry,
                            &cloud_target.api_key,
                            &effective_request.prompt,
                            &size,
                        ),
                    )
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Cloud image generation via {} failed: {}",
                            cloud_target.entry.display_name,
                            error
                        )
                    })?;
                    ensure_not_canceled(&job_control)?;

                    emit_progress(
                        &state,
                        job_id,
                        0.78,
                        "Saving",
                        "Writing the returned cloud image into outputs/ and preparing the preview.",
                    );
                    tokio::fs::write(&output_path, &result.bytes).await?;
                    let note = if let Some(revised_prompt) = result
                        .revised_prompt
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        format!(
                            "Cloud image generation ran through {}. Provider revised prompt: {}",
                            cloud_target.entry.display_name, revised_prompt
                        )
                    } else {
                        format!(
                            "Cloud image generation ran through {}.",
                            cloud_target.entry.display_name
                        )
                    };
                    (
                        file_name,
                        relative_path,
                        output_path,
                        result.mime,
                        note,
                        None,
                    )
                }
                MediaKind::Audio => {
                    let spoken_text_owned = effective_request
                        .prepared_spoken_text
                        .clone()
                        .or_else(|| request.combined_audio_literal_prompt())
                        .ok_or_else(|| anyhow::anyhow!("Cloud speech generation needs spoken text to voice."))?;
                    let spoken_text = spoken_text_owned.trim();
                    if spoken_text.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Cloud speech generation needs spoken text to voice."
                        ));
                    }
                    let instructions = (!effective_request.prompt.trim().is_empty())
                        .then_some(effective_request.prompt.as_str());
                    let result = await_cancelable(
                        &job_control,
                        generate_cloud_speech(
                            &cloud_target.entry,
                            &cloud_target.api_key,
                            spoken_text,
                            instructions,
                        ),
                    )
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Cloud speech generation via {} failed: {}",
                            cloud_target.entry.display_name,
                            error
                        )
                    })?;
                    ensure_not_canceled(&job_control)?;

                    emit_progress(
                        &state,
                        job_id,
                        0.78,
                        "Saving",
                        "Writing the returned cloud speech audio into outputs/ and preparing playback.",
                    );
                    tokio::fs::write(&output_path, &result.bytes).await?;
                    (
                        file_name,
                        relative_path,
                        output_path,
                        result.mime,
                        format!(
                            "Cloud speech generation ran through {} using the '{}' voice.",
                            cloud_target.entry.display_name, result.voice
                        ),
                        None,
                    )
                }
                MediaKind::Gif => {
                    return Err(anyhow::anyhow!(
                        "Cloud GIF generation is not enabled in Chatty-art yet."
                    ));
                }
                MediaKind::Video => {
                    let (width, height) = effective_request.settings.dimensions_for(request.kind);
                    let size = format!("{width}x{height}");
                    let reference_image_path = reference_asset
                        .as_ref()
                        .filter(|asset| asset.kind == MediaKind::Image)
                        .map(|asset| asset.disk_path(&state.paths.input_dir, &state.paths.outputs_dir));
                    let result = await_cancelable(
                        &job_control,
                        generate_cloud_video(
                            &cloud_target.entry,
                            &cloud_target.api_key,
                            &effective_request.prompt,
                            &size,
                            effective_request.settings.video_duration_seconds,
                            reference_image_path.as_deref(),
                        ),
                    )
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Cloud video generation via {} failed: {}",
                            cloud_target.entry.display_name,
                            error
                        )
                    })?;
                    ensure_not_canceled(&job_control)?;

                    emit_progress(
                        &state,
                        job_id,
                        0.82,
                        "Saving",
                        "Writing the returned cloud MP4 into outputs/ and preparing playback.",
                    );
                    tokio::fs::write(&output_path, &result.bytes).await?;
                    (
                        file_name,
                        relative_path,
                        output_path,
                        result.mime,
                        if matches!(cloud_target.entry.provider_kind, CloudProviderKind::Gemini) {
                            format!(
                                "Cloud video generation ran through {} using Gemini model {} at {} for {} second(s).",
                                cloud_target.entry.display_name, result.model, result.size, result.seconds
                            )
                        } else {
                            format!(
                                "Cloud video generation ran through {} using {} at {} for {} second(s). This OpenAI Videos API path is deprecated and must be replaced before September 24, 2026.",
                                cloud_target.entry.display_name, result.model, result.size, result.seconds
                            )
                        },
                        None,
                    )
                }
            }
        }
    };

    emit_progress(
        &state,
        job_id,
        0.93,
        "Saving",
        "Writing the output file and metadata into outputs/.",
    );

    let note = combine_notes(
        &prompt_assist_note,
        &generation_note,
        reference_summary.as_ref(),
        match model.backend {
            ModelBackend::LlamaCpp => {
                "Generated locally with the bundled llama.cpp Vulkan runtime."
            }
            ModelBackend::StableDiffusionCpp => {
                "Generated locally with the stable-diffusion.cpp realism backend."
            }
            ModelBackend::AudioRuntime => "Generated locally with the realism audio backend.",
            ModelBackend::Cloud => {
                if request.kind == MediaKind::Audio {
                    "Generated through the explicitly selected cloud speech provider."
                } else {
                    "Generated through the explicitly selected cloud media route."
                }
            }
        },
    );
    let created_at = Utc::now();
    let prompt_assist_route = if request.prompt_assist == PromptAssistMode::Off {
        None
    } else if interpreter_model_name
        .as_deref()
        .is_some_and(|value| value.contains("(cloud:"))
    {
        Some("cloud".to_string())
    } else {
        Some("local".to_string())
    };
    let vision_assist_route = if vision_model_name.is_none() {
        None
    } else if vision_model_name
        .as_deref()
        .is_some_and(|value| value.contains("(cloud:"))
    {
        Some("cloud".to_string())
    } else {
        Some("local".to_string())
    };
    let output_route = if media_generation_cloud_target.is_some() {
        Some("cloud".to_string())
    } else {
        Some("local".to_string())
    };
    let output = OutputEntry {
        id: file_name
            .trim_end_matches(&format!(
                ".{}",
                Path::new(&file_name)
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
            ))
            .to_string(),
        job_id,
        kind: request.kind,
        style: request.style,
        backend: model.backend,
        model: model.name.clone(),
        prompt: request.prompt.clone(),
        negative_prompt: effective_request.negative_prompt.clone(),
        compiled_prompt,
        spoken_text: effective_request.prepared_spoken_text.clone(),
        prompt_assist: request.prompt_assist,
        interpreter_model: interpreter_model_name,
        prompt_assist_route: prompt_assist_route.clone(),
        vision_model: vision_model_name,
        vision_assist_route: vision_assist_route.clone(),
        output_route: output_route.clone(),
        lora_name: selected_loras.first().map(|lora| lora.name.clone()),
        lora_weight: effective_request
            .normalized_lora_selections()
            .first()
            .and_then(|selection| selection.weight)
            .or(Some(1.0))
            .filter(|_| !selected_loras.is_empty()),
        lora_labels: selected_loras
            .iter()
            .zip(effective_request.normalized_lora_selections().iter())
            .map(|(lora, selection)| {
                format!("{} @ {:.2}", lora.name, selection.weight.unwrap_or(1.0))
            })
            .collect(),
        file_name: file_name.clone(),
        relative_path: relative_path.clone(),
        url: format!("/outputs/{relative_path}"),
        mime,
        created_at,
        settings: effective_request.settings.clone(),
        used_seed: u64::from(used_seed),
        resolution_label: effective_request
            .settings
            .resolution_label_for(request.kind),
        reference_asset: reference_asset.map(|asset| asset.relative_path),
        reference_intent: reference_summary.as_ref().map(|summary| summary.intent),
        end_reference_asset: end_reference_asset.map(|asset| asset.relative_path),
        control_reference_asset: control_reference_asset.map(|asset| asset.relative_path),
        note,
    };

    let sidecar_path = output_path.with_extension(format!(
        "{}.json",
        output_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
    ));
    tokio::fs::write(&sidecar_path, serde_json::to_vec_pretty(&output)?).await?;

    if let Some(trace) = planner_trace {
        let planner_sidecar = PlannerTraceSidecar {
            job_id,
            kind: request.kind,
            style: request.style,
            backend: model.backend,
            model: model.name.clone(),
            prompt_assist_route: prompt_assist_route.clone(),
            vision_assist_route: vision_assist_route.clone(),
            output_route: output_route.clone().unwrap_or_else(|| "local".to_string()),
            prompt: request.prompt.clone(),
            note: generation_note.clone(),
            used_fallback: trace.used_fallback,
            extracted_json: trace.extracted_json,
            raw_output: trace.raw_output,
            stderr: trace.stderr,
        };
        let planner_sidecar_path = output_path.with_extension(format!(
            "{}.planner.json",
            output_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
        ));
        tokio::fs::write(
            &planner_sidecar_path,
            serde_json::to_vec_pretty(&planner_sidecar)?,
        )
        .await?;
    }

    if let Some(trace) = prompt_assist_sidecar {
        let prompt_assist_sidecar_path = output_path.with_extension(format!(
            "{}.compiler.json",
            output_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
        ));
        tokio::fs::write(
            &prompt_assist_sidecar_path,
            serde_json::to_vec_pretty(&trace)?,
        )
        .await?;
    }

    let _ = state.events.send(ServerEvent::Completed { job_id, output });

    Ok(())
}

async fn run_generation_batch(
    state: AppState,
    job_id: Uuid,
    job_control: Arc<JobControl>,
    model: ModelInfo,
    media_generation_cloud_target: Option<MediaGenerationCloudTarget>,
    selected_loras: Vec<LoraInfo>,
    prompt_interpreter_model: Option<ModelInfo>,
    vision_interpreter_model: Option<ModelInfo>,
    request: GenerateRequest,
    reference_asset: Option<InputAsset>,
    end_reference_asset: Option<InputAsset>,
    control_reference_asset: Option<InputAsset>,
    first_seed: u32,
) -> Result<()> {
    let batch_total = request.normalized_batch_count();

    for batch_index in 0..batch_total {
        ensure_not_canceled(&job_control)?;
        let mut run_request = request.clone();
        run_request.batch_count = 1;
        let seed = if batch_total == 1 {
            first_seed
        } else {
            resolve_runtime_seed(None).map_err(|message| anyhow::anyhow!(message))?
        };
        run_request.settings.seed = Some(u64::from(seed));

        if batch_total > 1 {
            emit_progress(
                &state,
                job_id,
                0.02,
                "Batch Queue",
                &format!(
                    "Starting batch item {} of {} with a fresh random seed.",
                    batch_index + 1,
                    batch_total
                ),
            );
        }

        run_generation_job(
            state.clone(),
            job_id,
            job_control.clone(),
            model.clone(),
            media_generation_cloud_target.clone(),
            selected_loras.clone(),
            prompt_interpreter_model.clone(),
            vision_interpreter_model.clone(),
            run_request,
            reference_asset.clone(),
            end_reference_asset.clone(),
            control_reference_asset.clone(),
        )
        .await?;

        if batch_total > 1 && batch_index + 1 < batch_total {
            ensure_not_canceled(&job_control)?;
            emit_progress(
                &state,
                job_id,
                0.98,
                "Batch Saved",
                &format!(
                    "Finished batch item {} of {}. Preparing the next queued generation.",
                    batch_index + 1,
                    batch_total
                ),
            );
        }
    }

    Ok(())
}

fn emit_progress(state: &AppState, job_id: Uuid, percent: f32, phase: &str, message: &str) {
    let _ = state.events.send(ServerEvent::Progress {
        job_id,
        percent,
        phase: phase.to_string(),
        message: message.to_string(),
    });
}

fn spawn_progress_heartbeat(
    state: AppState,
    job_id: Uuid,
    start_percent: f32,
    max_percent: f32,
    phase: &'static str,
    message: &'static str,
) -> oneshot::Sender<()> {
    let (stop_tx, mut stop_rx) = oneshot::channel();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(12));
        let mut percent = start_percent;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    percent = (percent + 0.04).min(max_percent);
                    emit_progress(&state, job_id, percent, phase, message);
                }
                _ = &mut stop_rx => break,
            }
        }
    });
    stop_tx
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    error!("{error:#}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong while handling that request.".to_string(),
    )
}

fn summarize_provider(paths: &AppPaths, entry: &CloudProviderEntry) -> Result<CloudProviderSummary> {
    Ok(CloudProviderSummary {
        id: entry.id.clone(),
        display_name: entry.display_name.clone(),
        provider_kind: entry.provider_kind,
        base_url: entry.base_url.clone(),
        prompt_assist_model_name: entry.prompt_assist_model_name.clone(),
        vision_model_name: entry.vision_model_name.clone(),
        image_generation_model_name: entry.image_generation_model_name.clone(),
        video_generation_model_name: entry.video_generation_model_name.clone(),
        audio_generation_model_name: entry.audio_generation_model_name.clone(),
        audio_generation_voice: entry.audio_generation_voice.clone(),
        enabled: entry.enabled,
        capabilities: entry.capabilities.clone(),
        prompt_assist_verification: entry.prompt_assist_verification.clone(),
        vision_assist_verification: entry.vision_assist_verification.clone(),
        media_generation_verification: entry.media_generation_verification.clone(),
        has_api_key: secrets::has_api_key(&paths.secrets_path, &entry.id)?,
    })
}

fn now_unix_ms() -> u64 {
    Utc::now().timestamp_millis().max(0) as u64
}

fn build_output_path(
    kind_dir: &Path,
    kind: MediaKind,
    model_name: &str,
    seed: u32,
) -> (String, String, PathBuf) {
    build_output_path_with_extension(kind_dir, kind, model_name, seed, kind.output_extension())
}

fn build_output_path_with_extension(
    kind_dir: &Path,
    kind: MediaKind,
    model_name: &str,
    seed: u32,
    extension: &str,
) -> (String, String, PathBuf) {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let slug = slugify(model_name);
    let file_name = format!(
        "{}-{}-{}-{}.{}",
        timestamp,
        kind.as_str(),
        slug,
        seed,
        extension
    );
    let relative_path = format!("{}/{}", kind.as_str(), file_name);
    let output_path = kind_dir.join(&file_name);
    (file_name, relative_path, output_path)
}

fn combine_notes(
    prompt_assist_note: &str,
    planner_note: &str,
    reference: Option<&ReferenceSummary>,
    default_note: &str,
) -> String {
    let mut parts = Vec::new();
    if let Some(reference) = reference {
        parts.push(reference.note.clone());
    }
    if !prompt_assist_note.trim().is_empty() {
        parts.push(prompt_assist_note.trim().to_string());
    }
    if !planner_note.trim().is_empty() {
        parts.push(planner_note.trim().to_string());
    }
    if parts.is_empty() {
        default_note.to_string()
    } else {
        parts.push(default_note.to_string());
        parts.join(" ")
    }
}

fn scan_models(
    models_dir: &Path,
    diffuse_runtime_dir: &Path,
    audio_runtime_dir: &Path,
) -> Result<Vec<ModelInfo>> {
    let mut models = Vec::new();

    for entry in std::fs::read_dir(models_dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        let relative_path = to_slash_path(path.strip_prefix(models_dir)?);
        let slug = slugify(&file_name);

        let Some(support) =
            detect_audio_runtime_package_support(&file_name, &path, audio_runtime_dir)
        else {
            continue;
        };

        models.push(ModelInfo {
            id: relative_path.clone(),
            name: file_name.clone(),
            slug,
            file_name,
            relative_path,
            family: support.family,
            backend: ModelBackend::AudioRuntime,
            generation_style: GenerationStyle::Realism,
            runtime_supported: support.runtime_supported,
            compatibility_note: support.compatibility_note,
            supported_kinds: support.supported_kinds,
            requires_reference: false,
            supports_image_reference: false,
            supports_reference_strength: false,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: support.supports_audio_reference,
            supports_voice_output: support.supports_voice_output,
            mmproj_path: None,
        });
    }

    for entry in WalkDir::new(models_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if !has_extension(path, "gguf") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let lower = file_name.to_ascii_lowercase();

        if lower.contains("mmproj") || lower.contains("vocoder") {
            continue;
        }

        let relative_path = to_slash_path(path.strip_prefix(models_dir)?);
        let mmproj_path = find_mmproj_file(models_dir, path);
        let slug = slugify(
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(&file_name),
        );
        let support = detect_model_support(
            &lower,
            mmproj_path.is_some(),
            diffuse_runtime_dir,
            audio_runtime_dir,
            models_dir,
            &relative_path,
            path,
        );

        models.push(ModelInfo {
            id: relative_path.clone(),
            name: path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(&file_name)
                .to_string(),
            slug,
            file_name,
            relative_path,
            family: support.family,
            backend: support.backend,
            generation_style: support.style,
            runtime_supported: support.runtime_supported,
            compatibility_note: support.compatibility_note,
            supported_kinds: support.supported_kinds,
            requires_reference: support.requires_reference,
            supports_image_reference: support.supports_image_reference,
            supports_reference_strength: support.supports_reference_strength,
            requires_end_image_reference: support.requires_end_image_reference,
            supports_end_image_reference: support.supports_end_image_reference,
            supports_video_reference: support.supports_video_reference,
            supports_audio_reference: support.supports_audio_reference,
            supports_voice_output: support.supports_voice_output || has_sibling_vocoder(path),
            mmproj_path: mmproj_path.map(|path| to_slash_path(path)),
        });
    }

    models.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(models)
}

fn scan_loras(models_dir: &Path) -> Result<Vec<LoraInfo>> {
    let lora_dirs = available_lora_dirs(models_dir);
    if lora_dirs.is_empty() {
        return Ok(Vec::new());
    }

    let mut loras = Vec::new();
    for loras_dir in lora_dirs {
        for entry in WalkDir::new(&loras_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            if !has_extension(path, "safetensors") && !has_extension(path, "ckpt") {
                continue;
            }

            let relative_path = to_slash_path(path.strip_prefix(models_dir)?);
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();
            let family_key = infer_lora_family_key(&relative_path, &file_name)
                .unwrap_or_else(|| "unknown".to_string());
            let runtime_supported = family_key != "unknown";
            let family = lora_family_label(&family_key).to_string();
            let compatibility_note = if runtime_supported {
                format!(
                    "{} LoRA ready for Realism + Advanced. Put compatible files in models/loras/{}/ or models/lora/{}/ and pair them with matching {} models.",
                    family, family_key, family_key, family
                )
            } else {
                "Detected, but Chatty-art could not determine a compatible LoRA family from this file or folder name. Put LoRAs in models/loras/<family>/ or models/lora/<family>/ using folders like flux, sd, sd3, wan, or qwen."
                    .to_string()
            };

            loras.push(LoraInfo {
                id: relative_path.clone(),
                name: path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or(&file_name)
                    .to_string(),
                file_name,
                relative_path,
                family,
                family_key,
                runtime_supported,
                compatibility_note,
            });
        }
    }

    loras.sort_by(|left, right| {
        left.family_key
            .cmp(&right.family_key)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(loras)
}

fn available_lora_dirs(models_dir: &Path) -> Vec<PathBuf> {
    ["loras", "lora"]
        .into_iter()
        .map(|name| models_dir.join(name))
        .filter(|path| path.exists() && path.is_dir())
        .collect()
}

fn import_bridge_lora_files(paths: &AppPaths, request: BridgeLoraImportRequest) -> Result<Vec<String>> {
    let lane_id = sanitize_bridge_lane_id(&request.lane_id);
    if lane_id.is_empty() {
        bail!("Bridge lane id is required.");
    }
    let family_key = sanitize_bridge_family_key(&request.family_key);
    if family_key.is_empty() {
        bail!("Choose a LoRA family folder first.");
    }

    let requested_asset_ids = request
        .asset_ids
        .into_iter()
        .map(|asset_id| asset_id.trim().to_string())
        .filter(|asset_id| !asset_id.is_empty())
        .collect::<Vec<_>>();
    if requested_asset_ids.is_empty() {
        bail!("Select at least one bridge LoRA first.");
    }

    let lane_dir = paths
        .models_dir
        .parent()
        .unwrap_or(&paths.models_dir)
        .join("bridge")
        .join("incoming_assets")
        .join(&lane_id);
    let root = paths
        .models_dir
        .parent()
        .unwrap_or(&paths.models_dir)
        .canonicalize()
        .with_context(|| format!("could not resolve {}", paths.models_dir.display()))?;
    let lane_dir = lane_dir
        .canonicalize()
        .with_context(|| format!("could not resolve {}", lane_dir.display()))?;
    if !lane_dir.starts_with(&root) {
        bail!("Refusing to import a bridge lane outside the local Chatty-art project folder.");
    }

    let target_dir = paths.models_dir.join("loras").join(&family_key);
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("could not create {}", target_dir.display()))?;

    let mut notes = Vec::new();
    let mut imported = 0usize;
    for asset_id in requested_asset_ids {
        let record_path = lane_dir.join(format!("{asset_id}.json"));
        let record_bytes = std::fs::read(&record_path)
            .with_context(|| format!("could not read {}", record_path.display()))?;
        let record: BridgeIncomingAssetRecord = serde_json::from_slice(&record_bytes)
            .with_context(|| format!("could not parse {}", record_path.display()))?;
        let payload_path = lane_dir.join(record.payload_file_name.trim());
        if !payload_path.is_file() {
            bail!("Bridge payload file is missing for asset {}.", asset_id);
        }
        if !has_extension(&payload_path, "safetensors") && !has_extension(&payload_path, "ckpt") {
            bail!(
                "Bridge asset {} is not a supported LoRA file. Chatty-art expects .safetensors or .ckpt.",
                asset_id
            );
        }

        let file_name = payload_path
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(record.file_name.as_str());
        let destination_path = unique_lora_destination_path(&target_dir, file_name);
        std::fs::copy(&payload_path, &destination_path).with_context(|| {
            format!(
                "copy {} -> {}",
                payload_path.display(),
                destination_path.display()
            )
        })?;
        imported += 1;
    }

    if imported == 0 {
        notes.push("No bridge LoRA files were imported.".to_string());
    } else {
        notes.push(format!(
            "Imported {} bridge LoRA file{} into models/loras/{}.",
            imported,
            if imported == 1 { "" } else { "s" },
            family_key
        ));
    }

    Ok(notes)
}

fn sanitize_bridge_lane_id(raw: &str) -> String {
    raw.trim()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn sanitize_bridge_family_key(raw: &str) -> String {
    let normalized = sanitize_bridge_lane_id(raw);
    match normalized.as_str() {
        "flux" | "sd" | "sd3" | "wan" | "qwen" => normalized,
        _ => String::new(),
    }
}

fn sanitize_output_relative_path(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            _ => return None,
        }
    }
    (!normalized.as_os_str().is_empty()).then_some(normalized)
}

fn delete_output_files(paths: &AppPaths, request: DeleteOutputsRequest) -> Result<DeleteOutputsResponse> {
    let mut relative_paths = request.relative_paths;
    relative_paths.sort();
    relative_paths.dedup();
    if relative_paths.is_empty() {
        bail!("Select at least one saved output to delete.");
    }

    let outputs_root = paths
        .outputs_dir
        .canonicalize()
        .with_context(|| format!("canonicalize {}", paths.outputs_dir.display()))?;
    let mut deleted_paths = Vec::new();
    let mut deleted_sidecars = 0usize;

    for relative in relative_paths {
        let rel_path = sanitize_output_relative_path(&relative)
            .ok_or_else(|| anyhow::anyhow!("invalid output path `{relative}`"))?;
        let candidate = paths.outputs_dir.join(&rel_path);
        let resolved = candidate
            .canonicalize()
            .with_context(|| format!("could not resolve {}", candidate.display()))?;
        if !resolved.starts_with(&outputs_root) {
            bail!("output path escapes outputs/: {}", relative);
        }
        if !resolved.is_file() {
            bail!("output file was not found: {}", relative);
        }

        std::fs::remove_file(&resolved)
            .with_context(|| format!("could not delete {}", resolved.display()))?;
        deleted_paths.push(to_slash_path(rel_path.as_path()));

        let file_name = resolved
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let parent = resolved.parent().unwrap_or(&outputs_root);
        for sidecar_name in [
            format!("{file_name}.json"),
            format!("{file_name}.compiler.json"),
            format!("{file_name}.planner.json"),
        ] {
            let sidecar = parent.join(sidecar_name);
            if sidecar.is_file() {
                std::fs::remove_file(&sidecar)
                    .with_context(|| format!("could not delete {}", sidecar.display()))?;
                deleted_sidecars += 1;
            }
        }
    }

    let mut notes = vec![format!(
        "Deleted {} saved output file{} from outputs/.",
        deleted_paths.len(),
        if deleted_paths.len() == 1 { "" } else { "s" }
    )];
    if deleted_sidecars > 0 {
        notes.push(format!(
            "Also removed {} metadata sidecar{} that belonged to those outputs.",
            deleted_sidecars,
            if deleted_sidecars == 1 { "" } else { "s" }
        ));
    }

    Ok(DeleteOutputsResponse {
        deleted_paths,
        notes,
    })
}

fn unique_lora_destination_path(target_dir: &Path, file_name: &str) -> PathBuf {
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("imported-lora");
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("safetensors");

    let mut candidate = target_dir.join(format!("{stem}.{ext}"));
    let mut suffix = 2usize;
    while candidate.exists() {
        candidate = target_dir.join(format!("{stem}-{suffix}.{ext}"));
        suffix += 1;
    }
    candidate
}

fn infer_lora_family_key(relative_path: &str, file_name: &str) -> Option<String> {
    let lower = format!(
        "{}/{}",
        relative_path.to_ascii_lowercase(),
        file_name.to_ascii_lowercase()
    );

    if lower.contains("flux") || lower.contains("kontext") {
        return Some("flux".to_string());
    }
    if lower.contains("sd3")
        || lower.contains("sd-3")
        || lower.contains("sd35")
        || lower.contains("sd3.5")
        || lower.contains("stable-diffusion-3")
    {
        return Some("sd3".to_string());
    }
    if lower.contains("wan") {
        return Some("wan".to_string());
    }
    if lower.contains("qwen") {
        return Some("qwen".to_string());
    }
    if lower.contains("/sd/")
        || lower.contains("stable-diffusion")
        || lower.contains("sdxl")
        || lower.contains("sd15")
        || lower.contains("sd1.5")
        || lower.contains("sd21")
        || lower.contains("sd2.1")
        || lower.contains("sd2")
    {
        return Some("sd".to_string());
    }

    None
}

fn lora_family_label(family_key: &str) -> &'static str {
    match family_key {
        "flux" => "FLUX",
        "sd3" => "SD3 / SD3.5",
        "wan" => "Wan",
        "qwen" => "Qwen Image",
        "sd" => "Stable Diffusion",
        _ => "Unknown",
    }
}

fn model_lora_family_key(model: &ModelInfo) -> Option<&'static str> {
    if model.backend != ModelBackend::StableDiffusionCpp {
        return None;
    }

    let family = model.family.to_ascii_lowercase();
    if family.contains("flux") {
        return Some("flux");
    }
    if family.contains("sd3") {
        return Some("sd3");
    }
    if family.contains("wan") {
        return Some("wan");
    }
    if family.contains("qwen") {
        return Some("qwen");
    }
    if family.contains("stable diffusion")
        || family.contains("self-contained diffusion")
        || family.contains("diffusion gguf")
    {
        return Some("sd");
    }

    None
}

fn model_lora_family_label(model: &ModelInfo) -> &'static str {
    model_lora_family_key(model)
        .map(lora_family_label)
        .unwrap_or("LoRA-compatible")
}

fn lora_matches_model(lora: &LoraInfo, model: &ModelInfo) -> bool {
    model_lora_family_key(model)
        .map(|family_key| family_key == lora.family_key)
        .unwrap_or(false)
}

struct DetectedModelSupport {
    family: String,
    backend: ModelBackend,
    style: GenerationStyle,
    runtime_supported: bool,
    compatibility_note: String,
    supported_kinds: Vec<MediaKind>,
    requires_reference: bool,
    supports_image_reference: bool,
    supports_reference_strength: bool,
    requires_end_image_reference: bool,
    supports_end_image_reference: bool,
    supports_video_reference: bool,
    supports_audio_reference: bool,
    supports_voice_output: bool,
}

fn scan_assets(input_dir: &Path, outputs_dir: &Path) -> Result<Vec<InputAsset>> {
    let mut assets = Vec::new();

    scan_input_assets_into(input_dir, &mut assets)?;
    scan_output_assets_into(outputs_dir, &mut assets)?;

    assets.sort_by(|left, right| {
        left.source
            .as_str()
            .cmp(right.source.as_str())
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    Ok(assets)
}

fn scan_input_assets_into(input_dir: &Path, assets: &mut Vec<InputAsset>) -> Result<()> {
    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let relative = to_slash_path(path.strip_prefix(input_dir)?);
        let kind = match relative.split('/').next() {
            Some("images") => MediaKind::Image,
            Some("audio") => MediaKind::Audio,
            Some("video") => MediaKind::Video,
            _ => continue,
        };

        assets.push(InputAsset {
            id: format!("input:{relative}"),
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            relative_path: relative.clone(),
            kind,
            url: format!("/input/{relative}"),
            source: AssetSource::Input,
        });
    }

    Ok(())
}

fn scan_output_assets_into(outputs_dir: &Path, assets: &mut Vec<InputAsset>) -> Result<()> {
    for entry in WalkDir::new(outputs_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let Some((kind, _)) = infer_output_media(path) else {
            continue;
        };
        let relative = to_slash_path(path.strip_prefix(outputs_dir)?);

        assets.push(InputAsset {
            id: format!("output:{relative}"),
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            relative_path: relative.clone(),
            kind,
            url: format!("/outputs/{relative}"),
            source: AssetSource::Output,
        });
    }

    Ok(())
}

fn scan_outputs(outputs_dir: &Path) -> Result<Vec<OutputEntry>> {
    let mut outputs = Vec::new();

    for entry in WalkDir::new(outputs_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let Some((kind, mime)) = infer_output_media(path) else {
            continue;
        };

        let actual_extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let sidecar = path.with_extension(format!("{actual_extension}.json"));
        if sidecar.exists() {
            let content = std::fs::read_to_string(&sidecar)?;
            if let Ok(output) = serde_json::from_str::<OutputEntry>(&content) {
                outputs.push(output);
                continue;
            }
        }

        let relative_path = to_slash_path(path.strip_prefix(outputs_dir)?);
        let metadata = std::fs::metadata(path)?;
        let created_at = metadata
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());

        outputs.push(OutputEntry {
            id: path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            job_id: Uuid::nil(),
            kind,
            model: "Unknown".to_string(),
            prompt: String::new(),
            negative_prompt: None,
            compiled_prompt: None,
            spoken_text: None,
            prompt_assist: PromptAssistMode::Off,
            interpreter_model: None,
            prompt_assist_route: None,
            vision_model: None,
            vision_assist_route: None,
            output_route: None,
            lora_name: None,
            lora_weight: None,
            lora_labels: Vec::new(),
            file_name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            relative_path: relative_path.clone(),
            url: format!("/outputs/{relative_path}"),
            mime,
            created_at,
            settings: default_settings(),
            used_seed: 0,
            resolution_label: default_settings().resolution_label_for(kind),
            reference_asset: None,
            reference_intent: None,
            end_reference_asset: None,
            control_reference_asset: None,
            note: "Saved output found in outputs/.".to_string(),
            style: GenerationStyle::Expressive,
            backend: ModelBackend::LlamaCpp,
        });
    }

    outputs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(outputs)
}

fn infer_output_media(path: &Path) -> Option<(MediaKind, String)> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some((MediaKind::Image, "image/png".to_string())),
        "jpg" | "jpeg" => Some((MediaKind::Image, "image/jpeg".to_string())),
        "webp" => Some((MediaKind::Image, "image/webp".to_string())),
        "gif" => Some((MediaKind::Gif, "image/gif".to_string())),
        "avi" => Some((MediaKind::Video, "video/x-msvideo".to_string())),
        "mp4" => Some((MediaKind::Video, "video/mp4".to_string())),
        "webm" => Some((MediaKind::Video, "video/webm".to_string())),
        "wav" => Some((MediaKind::Audio, "audio/wav".to_string())),
        "mp3" => Some((MediaKind::Audio, "audio/mpeg".to_string())),
        "ogg" => Some((MediaKind::Audio, "audio/ogg".to_string())),
        _ => None,
    }
}

fn find_mmproj_file(models_dir: &Path, path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let model_name = path.file_name()?.to_str()?.to_ascii_lowercase();
    let matches = std::fs::read_dir(parent).ok()?;
    let candidates = matches
        .flatten()
        .map(|entry| entry.path())
        .filter(|candidate| candidate != path && has_extension(candidate, "gguf"))
        .filter_map(|candidate| {
            let candidate_name = candidate.file_name()?.to_str()?.to_ascii_lowercase();
            if !candidate_name.contains("mmproj") {
                return None;
            }
            Some((candidate, candidate_name))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return None;
    }

    for (candidate, candidate_name) in &candidates {
        if is_probable_mmproj_match(&model_name, candidate_name) {
            return candidate.strip_prefix(models_dir).ok().map(to_slash_path);
        }
    }

    if candidates.len() == 1 {
        return candidates[0]
            .0
            .strip_prefix(models_dir)
            .ok()
            .map(to_slash_path);
    }

    None
}

fn is_probable_mmproj_match(model_name: &str, candidate_name: &str) -> bool {
    let model_tokens = meaningful_mmproj_tokens(model_name);
    let candidate_tokens = meaningful_mmproj_tokens(candidate_name);
    let shared = model_tokens
        .iter()
        .filter(|token| candidate_tokens.contains(token))
        .cloned()
        .collect::<Vec<_>>();

    match shared.len() {
        0 => false,
        len if len >= 2 => true,
        _ => matches!(shared[0].as_str(), "moondream2"),
    }
}

fn meaningful_mmproj_tokens(name: &str) -> Vec<String> {
    name.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '.'))
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| {
            token.len() >= 3
                || token == "vl"
                || (token.ends_with('b')
                    && token.len() >= 2
                    && token[..token.len() - 1].chars().all(|ch| ch.is_ascii_digit()))
        })
        .filter(|token| {
            !matches!(
                token.as_str(),
                "gguf"
                    | "mmproj"
                    | "model"
                    | "text"
                    | "f16"
                    | "q2"
                    | "q3"
                    | "q4"
                    | "q5"
                    | "q6"
                    | "q8"
                    | "imat"
                    | "instruct"
                    | "chat"
                    | "alpha"
                    | "beta"
                    | "one"
                    | "two"
                    | "three"
                    | "abliterated"
            )
        })
        .collect()
}

fn has_sibling_vocoder(path: &Path) -> bool {
    path.parent()
        .and_then(|parent| std::fs::read_dir(parent).ok())
        .map(|iter| {
            iter.flatten().any(|entry| {
                let candidate = entry.path();
                let lower = candidate
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                candidate != path && has_extension(&candidate, "gguf") && lower.contains("vocoder")
            })
        })
        .unwrap_or(false)
}

fn choose_prompt_interpreter_model(
    models: &[ModelInfo],
    selected_model: &ModelInfo,
) -> Option<ModelInfo> {
    let mut preferred_candidates = models
        .iter()
        .filter(|model| supports_auto_prompt_interpreter(model))
        .cloned()
        .collect::<Vec<_>>();
    if preferred_candidates.is_empty() {
        preferred_candidates = models
            .iter()
            .filter(|model| supports_prompt_interpreter(model))
            .cloned()
            .collect::<Vec<_>>();
    }
    if preferred_candidates.is_empty() {
        return None;
    }

    preferred_candidates.sort_by(|left, right| {
        prompt_interpreter_sort_key(left)
            .cmp(&prompt_interpreter_sort_key(right))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    if supports_auto_prompt_interpreter(selected_model) {
        let selected_size = parameter_hint(&selected_model.name);
        let smallest_size = parameter_hint(&preferred_candidates[0].name);
        if selected_size <= smallest_size.saturating_add(2) {
            return Some(selected_model.clone());
        }
    }

    preferred_candidates.into_iter().next()
}

fn choose_vision_interpreter_model(
    models: &[ModelInfo],
    selected_model: &ModelInfo,
    prompt_interpreter_model: Option<&ModelInfo>,
) -> Option<ModelInfo> {
    if prompt_interpreter_model.is_some_and(supports_vision_interpreter) {
        return prompt_interpreter_model.cloned();
    }

    let mut candidates = models
        .iter()
        .filter(|model| supports_vision_interpreter(model))
        .cloned()
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by(|left, right| {
        vision_interpreter_sort_key(left)
            .cmp(&vision_interpreter_sort_key(right))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    if supports_vision_interpreter(selected_model) {
        let selected_size = parameter_hint(&selected_model.name);
        let smallest_size = parameter_hint(&candidates[0].name);
        if selected_size <= smallest_size.saturating_add(2) {
            return Some(selected_model.clone());
        }
    }

    candidates.into_iter().next()
}

fn supports_prompt_interpreter(model: &ModelInfo) -> bool {
    model.runtime_supported
        && model.backend == ModelBackend::LlamaCpp
        && model.generation_style == GenerationStyle::Expressive
        && model.family.to_ascii_lowercase() != "voice"
        && !model.supports_voice_output
}

fn supports_manual_prompt_interpreter(model: &ModelInfo) -> bool {
    supports_prompt_interpreter(model)
}

fn supports_auto_prompt_interpreter(model: &ModelInfo) -> bool {
    supports_prompt_interpreter(model) && !supports_vision_interpreter(model)
}

fn supports_vision_interpreter(model: &ModelInfo) -> bool {
    supports_prompt_interpreter(model) && model.supports_image_reference && model.mmproj_path.is_some()
}

fn prompt_interpreter_sort_key(model: &ModelInfo) -> (u32, u8) {
    (
        parameter_hint(&model.name),
        if model.family.eq_ignore_ascii_case("vision") {
            1
        } else {
            0
        },
    )
}

fn vision_interpreter_sort_key(model: &ModelInfo) -> (u8, u32, u8) {
    let lower = model.name.to_ascii_lowercase();
    let family_bias = if lower.contains("qwen2.5-vl") {
        0
    } else if lower.contains("llava") {
        1
    } else if lower.contains("qwen2-vl-2b") {
        3
    } else if lower.contains("moondream") {
        4
    } else {
        2
    };
    (
        family_bias,
        parameter_hint(&model.name),
        if model.family.eq_ignore_ascii_case("vision") {
            1
        } else {
            0
        },
    )
}

fn parameter_hint(name: &str) -> u32 {
    let chars = name.chars().collect::<Vec<_>>();
    let mut best: Option<u32> = None;

    for index in 0..chars.len() {
        if !chars[index].is_ascii_digit() {
            continue;
        }
        if index > 0 && (chars[index - 1].is_ascii_digit() || chars[index - 1] == '.') {
            continue;
        }

        let mut end = index;
        while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
            end += 1;
        }

        if end >= chars.len() || !matches!(chars[end], 'b' | 'B') {
            continue;
        }

        let number = chars[index..end].iter().collect::<String>();
        if let Ok(parsed) = number.parse::<f32>() {
            let scaled = (parsed * 10.0).round() as u32;
            best = Some(best.map(|current| current.min(scaled)).unwrap_or(scaled));
        }
    }

    best.unwrap_or(9_999)
}

fn is_vision_model(name: &str) -> bool {
    name.contains("llava")
        || name.contains("qwen2vl")
        || name.contains("vision")
        || name.contains("minicpmv")
        || name.contains("gemma3")
        || name.contains("vl")
}

fn is_voice_model(name: &str) -> bool {
    name.contains("tts") || name.contains("oute") || name.contains("kokoro")
}

fn detect_model_support(
    name: &str,
    has_mmproj: bool,
    diffuse_runtime_dir: &Path,
    audio_runtime_dir: &Path,
    models_dir: &Path,
    relative_path: &str,
    model_path: &Path,
) -> DetectedModelSupport {
    if let Some(support) =
        detect_sdcpp_support(diffuse_runtime_dir, models_dir, relative_path, name)
    {
        return DetectedModelSupport {
            family: support.family,
            backend: ModelBackend::StableDiffusionCpp,
            style: GenerationStyle::Realism,
            runtime_supported: support.runtime_supported,
            compatibility_note: support.compatibility_note,
            supported_kinds: support.supported_kinds,
            requires_reference: support.requires_reference,
            supports_image_reference: support.supports_image_reference,
            supports_reference_strength: support.supports_reference_strength,
            requires_end_image_reference: support.requires_end_image_reference,
            supports_end_image_reference: support.supports_end_image_reference,
            supports_video_reference: support.supports_video_reference,
            supports_audio_reference: support.supports_audio_reference,
            supports_voice_output: false,
        };
    }

    if let Some(support) = detect_audio_runtime_support(name, audio_runtime_dir) {
        return DetectedModelSupport {
            family: support.family,
            backend: ModelBackend::AudioRuntime,
            style: GenerationStyle::Realism,
            runtime_supported: support.runtime_supported,
            compatibility_note: support.compatibility_note,
            supported_kinds: support.supported_kinds,
            requires_reference: false,
            supports_image_reference: false,
            supports_reference_strength: false,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: support.supports_audio_reference,
            supports_voice_output: support.supports_voice_output,
        };
    }

    let gguf = inspect_gguf(model_path);

    if let Some(architecture) = gguf.as_ref().and_then(|summary| summary.architecture()) {
        if is_companion_only_architecture(architecture) {
            return DetectedModelSupport {
                family: "Companion GGUF".to_string(),
                backend: ModelBackend::StableDiffusionCpp,
                style: GenerationStyle::Realism,
                runtime_supported: false,
                compatibility_note: format!(
                    "This GGUF is a helper weight with architecture '{architecture}', not a selectable generation model. Keep it in models/ as a companion file for realism models."
                ),
                supported_kinds: Vec::new(),
                requires_reference: false,
                supports_image_reference: false,
                supports_reference_strength: false,
                requires_end_image_reference: false,
                supports_end_image_reference: false,
                supports_video_reference: false,
                supports_audio_reference: false,
                supports_voice_output: false,
            };
        }

        if !is_supported_llama_architecture(architecture) {
            return DetectedModelSupport {
                family: "Unsupported GGUF".to_string(),
                backend: ModelBackend::LlamaCpp,
                style: GenerationStyle::Expressive,
                runtime_supported: false,
                compatibility_note: format!(
                    "The bundled llama.cpp runtime does not recognize GGUF architecture '{architecture}'. This file may need a different backend or a newer runtime."
                ),
                supported_kinds: Vec::new(),
                requires_reference: false,
                supports_image_reference: false,
                supports_reference_strength: false,
                requires_end_image_reference: false,
                supports_end_image_reference: false,
                supports_video_reference: false,
                supports_audio_reference: false,
                supports_voice_output: false,
            };
        }
    }

    if is_voice_model(name) {
        return DetectedModelSupport {
            family: "Voice".to_string(),
            backend: ModelBackend::LlamaCpp,
            style: GenerationStyle::Expressive,
            runtime_supported: true,
            compatibility_note: "Compatible with the bundled llama.cpp runtime in expressive mode."
                .to_string(),
            supported_kinds: vec![MediaKind::Audio],
            requires_reference: false,
            supports_image_reference: false,
            supports_reference_strength: false,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: name.contains("audio") || name.contains("omni"),
            supports_voice_output: true,
        };
    }

    if is_vision_model(name) || has_mmproj {
        return DetectedModelSupport {
            family: "Vision".to_string(),
            backend: ModelBackend::LlamaCpp,
            style: GenerationStyle::Expressive,
            runtime_supported: true,
            compatibility_note: "Compatible with the bundled llama.cpp runtime in expressive mode."
                .to_string(),
            supported_kinds: vec![MediaKind::Image, MediaKind::Gif, MediaKind::Audio],
            requires_reference: false,
            supports_image_reference: true,
            supports_reference_strength: false,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: name.contains("audio") || name.contains("omni"),
            supports_voice_output: false,
        };
    }

    DetectedModelSupport {
        family: "Text".to_string(),
        backend: ModelBackend::LlamaCpp,
        style: GenerationStyle::Expressive,
        runtime_supported: true,
        compatibility_note: "Compatible with the bundled llama.cpp runtime in expressive mode."
            .to_string(),
        supported_kinds: vec![MediaKind::Image, MediaKind::Gif, MediaKind::Audio],
        requires_reference: false,
        supports_image_reference: false,
        supports_reference_strength: false,
        requires_end_image_reference: false,
        supports_end_image_reference: false,
        supports_video_reference: false,
        supports_audio_reference: name.contains("audio") || name.contains("omni"),
        supports_voice_output: false,
    }
}

fn is_companion_only_architecture(architecture: &str) -> bool {
    matches!(architecture, "t5encoder")
}

fn is_supported_llama_architecture(architecture: &str) -> bool {
    matches!(
        architecture,
        "llama"
            | "mllama"
            | "gpt-oss"
            | "gpt2"
            | "gptj"
            | "gptneox"
            | "falcon"
            | "mpt"
            | "starcoder"
            | "starcoder2"
            | "bert"
            | "nomic-bert"
            | "jina-bert-v2"
            | "qwen2"
            | "qwen2moe"
            | "qwen2vl"
            | "qwen3"
            | "qwen3moe"
            | "gemma"
            | "gemma2"
            | "gemma3"
            | "phi2"
            | "phi3"
            | "phimoe"
            | "deepseek"
            | "deepseek2"
            | "deepseek3"
            | "command-r"
            | "cohere2"
            | "olmo"
            | "olmo2"
            | "openelm"
            | "baichuan"
            | "xverse"
            | "internlm2"
            | "minicpm"
            | "minicpm3"
            | "exaone"
            | "mamba"
            | "rwkv6"
            | "rwkv6qwen2"
            | "granite"
            | "granitemoe"
            | "glm4"
            | "megrez"
            | "bloom"
            | "chameleon"
            | "hunyuan"
            | "smollm3"
            | "orion"
            | "refact"
            | "stablelm"
            | "persimmon"
            | "plamo"
            | "dbrx"
            | "grok"
            | "deci"
    )
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case(extension))
        .unwrap_or(false)
}

fn to_slash_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "model".to_string()
    } else {
        trimmed.to_string()
    }
}

fn default_settings() -> GenerationSettings {
    GenerationSettings {
        temperature: 0.6,
        steps: 28,
        cfg_scale: 7.5,
        sampler: "euler".to_string(),
        scheduler: "default".to_string(),
        reference_strength: 0.8,
        flow_shift: 3.0,
        resolution: ResolutionPreset::Square512,
        video_resolution: VideoResolutionPreset::Square256,
        video_duration_seconds: 2,
        video_fps: 8,
        audio_duration_seconds: 10,
        low_vram_mode: false,
        seed: Some(0),
    }
}

fn resolve_runtime_seed(seed: Option<u64>) -> std::result::Result<u32, String> {
    match seed {
        Some(seed) => u32::try_from(seed).map_err(|_| {
            "Seed is too large for the bundled llama.cpp runtime on Windows.".to_string()
        }),
        None => Ok(random::<u32>()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        AppPaths, MAX_RUNTIME_SEED, build_prompt_handoff, choose_prompt_interpreter_model,
        detect_model_support, parameter_hint, resolve_runtime_seed,
    };
    use crate::types::{
        AudioPromptSegment, GenerationSettings, GenerationStyle, MediaKind, ModelBackend,
        ModelInfo, PromptAssistMode, ReferenceIntent, ResolutionPreset, VideoResolutionPreset,
    };

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("chatty-art-main-{label}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fake_model(name: &str, family: &str) -> ModelInfo {
        ModelInfo {
            id: name.to_string(),
            name: name.to_string(),
            slug: name.to_string(),
            file_name: format!("{name}.gguf"),
            relative_path: format!("{name}.gguf"),
            family: family.to_string(),
            backend: ModelBackend::LlamaCpp,
            generation_style: GenerationStyle::Expressive,
            runtime_supported: true,
            compatibility_note: String::new(),
            supported_kinds: vec![MediaKind::Image, MediaKind::Gif, MediaKind::Audio],
            requires_reference: false,
            supports_image_reference: false,
            supports_reference_strength: false,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: false,
            supports_voice_output: false,
            mmproj_path: None,
        }
    }

    fn fake_model_with_multimodal_support(name: &str, family: &str) -> ModelInfo {
        ModelInfo {
            supports_image_reference: true,
            mmproj_path: Some(format!("{name}-mmproj.gguf")),
            ..fake_model(name, family)
        }
    }

    fn fake_audio_model(name: &str, supports_voice_output: bool) -> ModelInfo {
        ModelInfo {
            id: name.to_string(),
            name: name.to_string(),
            slug: name.to_string(),
            file_name: name.to_string(),
            relative_path: name.to_string(),
            family: if supports_voice_output {
                "OuteTTS".to_string()
            } else {
                "Stable Audio Open".to_string()
            },
            backend: ModelBackend::AudioRuntime,
            generation_style: GenerationStyle::Realism,
            runtime_supported: true,
            compatibility_note: String::new(),
            supported_kinds: vec![MediaKind::Audio],
            requires_reference: false,
            supports_image_reference: false,
            supports_reference_strength: false,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: false,
            supports_voice_output,
            mmproj_path: None,
        }
    }

    fn fake_realism_image_model(name: &str, family: &str) -> ModelInfo {
        ModelInfo {
            id: name.to_string(),
            name: name.to_string(),
            slug: name.to_string(),
            file_name: format!("{name}.gguf"),
            relative_path: format!("{name}.gguf"),
            family: family.to_string(),
            backend: ModelBackend::StableDiffusionCpp,
            generation_style: GenerationStyle::Realism,
            runtime_supported: true,
            compatibility_note: String::new(),
            supported_kinds: vec![MediaKind::Image, MediaKind::Gif],
            requires_reference: false,
            supports_image_reference: true,
            supports_reference_strength: true,
            requires_end_image_reference: false,
            supports_end_image_reference: false,
            supports_video_reference: false,
            supports_audio_reference: false,
            supports_voice_output: false,
            mmproj_path: None,
        }
    }

    fn test_settings() -> GenerationSettings {
        GenerationSettings {
            temperature: 0.6,
            steps: 24,
            cfg_scale: 6.0,
            sampler: "euler".to_string(),
            scheduler: "default".to_string(),
            reference_strength: 0.8,
            flow_shift: 3.0,
            resolution: ResolutionPreset::Square512,
            video_resolution: VideoResolutionPreset::Square256,
            video_duration_seconds: 2,
            video_fps: 8,
            audio_duration_seconds: 10,
            low_vram_mode: true,
            seed: Some(1234),
        }
    }

    fn dummy_paths() -> AppPaths {
        AppPaths {
            models_dir: PathBuf::from("."),
            input_dir: PathBuf::from("."),
            outputs_dir: PathBuf::from("."),
            runtime_dir: PathBuf::from("."),
            config_dir: PathBuf::from("."),
            config_path: PathBuf::from("."),
            secrets_path: PathBuf::from("."),
            diffuse_runtime_dir: PathBuf::from("."),
            audio_runtime_dir: PathBuf::from("."),
        }
    }

    fn tiny_gguf_with_architecture(architecture: &str) -> Vec<u8> {
        fn encode_string(value: &str) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
            bytes.extend_from_slice(value.as_bytes());
            bytes
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        bytes.extend_from_slice(&encode_string("general.architecture"));
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(&encode_string(architecture));
        bytes.extend_from_slice(&encode_string("token_embd.weight"));
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&8_u64.to_le_bytes());
        bytes.extend_from_slice(&8_u64.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes
    }

    #[test]
    fn accepts_max_supported_seed() {
        assert_eq!(
            resolve_runtime_seed(Some(MAX_RUNTIME_SEED)).unwrap(),
            u32::MAX
        );
    }

    #[test]
    fn rejects_seed_above_supported_range() {
        assert!(resolve_runtime_seed(Some(MAX_RUNTIME_SEED + 1)).is_err());
    }

    #[test]
    fn parameter_hint_reads_model_size_from_name() {
        assert_eq!(parameter_hint("Qwen3-8B-abliterated-q8_0.gguf"), 80);
        assert_eq!(parameter_hint("gpt-oss-20b-Q4_K_M.gguf"), 200);
        assert_eq!(parameter_hint("plain-model.gguf"), 9_999);
    }

    #[test]
    fn prompt_interpreter_prefers_fast_expressive_model() {
        let selected = fake_model("gpt-oss-20b-Q4_K_M", "Text");
        let fast = fake_model("Qwen3-8B-abliterated-q8_0", "Text");
        let models = vec![selected.clone(), fast.clone()];
        let chosen = choose_prompt_interpreter_model(&models, &selected).unwrap();
        assert_eq!(chosen.name, fast.name);
    }

    #[test]
    fn prompt_interpreter_auto_prefers_text_only_helper_over_multimodal() {
        let selected = fake_model_with_multimodal_support("Qwen2.5-VL-7B-Instruct-Q4_K_M", "Vision");
        let text_only = fake_model("Qwen3-8B-abliterated-q8_0", "Text");
        let models = vec![selected.clone(), text_only.clone()];
        let chosen = choose_prompt_interpreter_model(&models, &selected).unwrap();
        assert_eq!(chosen.name, text_only.name);
    }

    #[test]
    fn prompt_interpreter_auto_falls_back_to_multimodal_when_text_only_missing() {
        let selected = fake_model_with_multimodal_support("Qwen2.5-VL-7B-Instruct-Q4_K_M", "Vision");
        let models = vec![selected.clone()];
        let chosen = choose_prompt_interpreter_model(&models, &selected).unwrap();
        assert_eq!(chosen.name, selected.name);
    }

    #[test]
    fn enables_qwen_image_for_realism_mode() {
        let diffuse_dir = temp_dir("diffuse");
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

        let models_dir = temp_dir("models");
        fs::write(models_dir.join("qwen-image-q4_k_m.gguf"), b"").unwrap();
        fs::write(models_dir.join("qwen_image_vae.safetensors"), b"").unwrap();
        fs::write(models_dir.join("Qwen2.5-VL-7B-Instruct-Q8_0.gguf"), b"").unwrap();
        let support = detect_model_support(
            "qwen-image-q4_k_m.gguf",
            false,
            &diffuse_dir,
            Path::new("."),
            &models_dir,
            "qwen-image-q4_k_m.gguf",
            &models_dir.join("qwen-image-q4_k_m.gguf"),
        );
        assert!(support.runtime_supported);
        assert_eq!(
            support.backend,
            crate::types::ModelBackend::StableDiffusionCpp
        );
        assert_eq!(support.style, crate::types::GenerationStyle::Realism);
    }

    #[test]
    fn marks_ltx_as_not_yet_wired() {
        let support = detect_model_support(
            "ltx-2.3-22b-dev-q4_k_m.gguf",
            false,
            Path::new("."),
            Path::new("."),
            Path::new("."),
            "ltx-2.3-22b-dev-q4_k_m.gguf",
            Path::new("ltx-2.3-22b-dev-q4_k_m.gguf"),
        );
        assert!(!support.runtime_supported);
        assert!(support.compatibility_note.contains("not wired"));
    }

    #[test]
    fn marks_t5_encoder_gguf_as_companion_weight() {
        let dir = temp_dir("t5");
        let model_path = dir.join("umt5-xxl-encoder-Q4_K_M.gguf");
        fs::write(&model_path, tiny_gguf_with_architecture("t5encoder")).unwrap();

        let support = detect_model_support(
            "umt5-xxl-encoder-q4_k_m.gguf",
            false,
            Path::new("."),
            Path::new("."),
            &dir,
            "umt5-xxl-encoder-Q4_K_M.gguf",
            &model_path,
        );

        assert!(!support.runtime_supported);
        assert_eq!(support.style, GenerationStyle::Realism);
        assert_eq!(support.backend, ModelBackend::StableDiffusionCpp);
        assert!(support.compatibility_note.contains("helper weight"));
    }

    #[test]
    fn marks_unknown_expressive_architecture_as_not_ready() {
        let dir = temp_dir("vaetki");
        let model_path = dir.join("VAETKI-VL-7B-A1B-Q4_K_M.gguf");
        fs::write(&model_path, tiny_gguf_with_architecture("vaetki")).unwrap();

        let support = detect_model_support(
            "vaetki-vl-7b-a1b-q4_k_m.gguf",
            false,
            Path::new("."),
            Path::new("."),
            &dir,
            "VAETKI-VL-7B-A1B-Q4_K_M.gguf",
            &model_path,
        );

        assert!(!support.runtime_supported);
        assert_eq!(support.style, GenerationStyle::Expressive);
        assert!(support.compatibility_note.contains("vaetki"));
    }

    #[test]
    fn detects_outetts_as_realism_audio_candidate() {
        let dir = temp_dir("outetts");
        let model_path = dir.join("Llama-OuteTTS-1.0-1B-Q4_K_M.gguf");
        fs::write(&model_path, tiny_gguf_with_architecture("llama")).unwrap();

        let support = detect_model_support(
            "llama-outetts-1.0-1b-q4_k_m.gguf",
            false,
            Path::new("."),
            &dir,
            &dir,
            "Llama-OuteTTS-1.0-1B-Q4_K_M.gguf",
            &model_path,
        );

        assert!(!support.runtime_supported);
        assert_eq!(support.style, GenerationStyle::Realism);
        assert_eq!(support.backend, ModelBackend::AudioRuntime);
        assert_eq!(support.supported_kinds, vec![MediaKind::Audio]);
        assert!(support.supports_voice_output);
        assert!(support.compatibility_note.contains("outetts"));
    }

    #[test]
    fn detects_qwen3_tts_as_realism_audio_candidate() {
        let dir = temp_dir("qwen3tts");
        let model_path = dir.join("Qwen3-TTS-4B-Q4_K_M.gguf");
        fs::write(&model_path, tiny_gguf_with_architecture("qwen3")).unwrap();

        let support = detect_model_support(
            "qwen3-tts-4b-q4_k_m.gguf",
            false,
            Path::new("."),
            &dir,
            &dir,
            "Qwen3-TTS-4B-Q4_K_M.gguf",
            &model_path,
        );

        assert!(!support.runtime_supported);
        assert_eq!(support.style, GenerationStyle::Realism);
        assert_eq!(support.backend, ModelBackend::AudioRuntime);
        assert_eq!(support.supported_kinds, vec![MediaKind::Audio]);
        assert!(support.supports_voice_output);
        assert!(support.compatibility_note.contains("Qwen3-TTS"));
    }

    #[tokio::test]
    async fn prompt_assist_keeps_sound_segments_out_of_prepared_prompt() {
        let request = crate::types::GenerateRequest {
            prompt: "cinematic city rain ambience".to_string(),
            negative_prompt: Some("distortion, clipping".to_string()),
            batch_count: 1,
            prompt_assist: PromptAssistMode::Gentle,
            model: "stable-audio-open-1.0".to_string(),
            kind: MediaKind::Audio,
            style: GenerationStyle::Realism,
            settings: test_settings(),
            reference_asset: None,
            reference_intent: ReferenceIntent::Guide,
            end_reference_asset: None,
            control_reference_asset: None,
            selected_prompt_model: None,
            selected_vision_model: None,
            selected_lora: None,
            selected_lora_weight: None,
            selected_loras: Vec::new(),
            prepared_prompt: Some(
                "cinematic city rain ambience, spacious stereo field, soft reflections"
                    .to_string(),
            ),
            prepared_negative_prompt: Some("distortion, clipping".to_string()),
            prepared_note: Some("Preview Handoff was reviewed before generation.".to_string()),
            prepared_interpreter_model: Some("Qwen3-8B-abliterated-q8_0".to_string()),
            prepared_spoken_text: None,
            audio_literal_prompt: None,
            audio_segments: vec![
                AudioPromptSegment {
                    label: Some("Rain Bed".to_string()),
                    literal: "steady rain on pavement".to_string(),
                    same_time_as_previous: false,
                },
                AudioPromptSegment {
                    label: Some("Thunder Hit".to_string()),
                    literal: "distant thunder crack".to_string(),
                    same_time_as_previous: true,
                },
            ],
            manual_focus_tags: Vec::new(),
            manual_assumptions: Vec::new(),
            manual_preserve_items: Vec::new(),
            manual_change_targets: Vec::new(),
            manual_avoid_items: Vec::new(),
        };

        let prepared = build_prompt_handoff(
            &dummy_paths(),
            &request,
            &fake_audio_model("stable-audio-open-1.0", false),
            None,
            None,
            false,
            None,
            1234,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(
            prepared.effective_request.prompt,
            "cinematic city rain ambience, spacious stereo field, soft reflections"
        );
        assert!(!prepared
            .effective_request
            .prompt
            .contains("steady rain on pavement"));
        assert!(!prepared
            .effective_request
            .prompt
            .contains("distant thunder crack"));
    }

    #[tokio::test]
    async fn sound_audio_without_prompt_assist_keeps_literal_lane_separate() {
        let request = crate::types::GenerateRequest {
            prompt: "lush forest ambience, soft wind".to_string(),
            negative_prompt: Some("distortion".to_string()),
            batch_count: 1,
            prompt_assist: PromptAssistMode::Off,
            model: "stable-audio-open-1.0".to_string(),
            kind: MediaKind::Audio,
            style: GenerationStyle::Realism,
            settings: test_settings(),
            reference_asset: None,
            reference_intent: ReferenceIntent::Guide,
            end_reference_asset: None,
            control_reference_asset: None,
            selected_prompt_model: None,
            selected_vision_model: None,
            selected_lora: None,
            selected_lora_weight: None,
            selected_loras: Vec::new(),
            prepared_prompt: None,
            prepared_negative_prompt: None,
            prepared_note: None,
            prepared_interpreter_model: None,
            prepared_spoken_text: None,
            audio_literal_prompt: Some("bird chirps, creek water".to_string()),
            audio_segments: Vec::new(),
            manual_focus_tags: Vec::new(),
            manual_assumptions: Vec::new(),
            manual_preserve_items: Vec::new(),
            manual_change_targets: Vec::new(),
            manual_avoid_items: Vec::new(),
        };

        let prepared = build_prompt_handoff(
            &dummy_paths(),
            &request,
            &fake_audio_model("stable-audio-open-1.0", false),
            None,
            None,
            false,
            None,
            1234,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(prepared.effective_request.prompt, "lush forest ambience, soft wind");
        assert_eq!(
            prepared.effective_request.combined_audio_literal_prompt().as_deref(),
            Some("bird chirps, creek water")
        );
    }

    #[tokio::test]
    async fn manual_realism_handoff_inputs_flow_into_prepared_prompt() {
        let request = crate::types::GenerateRequest {
            prompt: "a lighthouse on a cliff".to_string(),
            negative_prompt: Some("blurry".to_string()),
            batch_count: 1,
            prompt_assist: PromptAssistMode::Off,
            model: "stable-diffusion-v1-5".to_string(),
            kind: MediaKind::Image,
            style: GenerationStyle::Realism,
            settings: test_settings(),
            reference_asset: None,
            reference_intent: ReferenceIntent::Guide,
            end_reference_asset: None,
            control_reference_asset: None,
            selected_prompt_model: None,
            selected_vision_model: None,
            selected_lora: None,
            selected_lora_weight: None,
            selected_loras: Vec::new(),
            prepared_prompt: None,
            prepared_negative_prompt: None,
            prepared_note: None,
            prepared_interpreter_model: None,
            prepared_spoken_text: None,
            audio_literal_prompt: None,
            audio_segments: Vec::new(),
            manual_focus_tags: vec![
                "golden hour".to_string(),
                "cinematic framing".to_string(),
            ],
            manual_assumptions: vec!["stormy coast".to_string()],
            manual_preserve_items: Vec::new(),
            manual_change_targets: Vec::new(),
            manual_avoid_items: Vec::new(),
        };

        let prepared = build_prompt_handoff(
            &dummy_paths(),
            &request,
            &fake_realism_image_model("stable-diffusion-v1-5", "Stable Diffusion"),
            None,
            None,
            false,
            None,
            1234,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(prepared
            .effective_request
            .prompt
            .contains("golden hour"));
        assert!(prepared
            .effective_request
            .prompt
            .contains("cinematic framing"));
        assert_eq!(
            prepared.focus_tags,
            vec!["golden hour".to_string(), "cinematic framing".to_string()]
        );
        assert_eq!(prepared.assumptions, vec!["stormy coast".to_string()]);
        assert!(prepared
            .prompt_assist_note
            .contains("Manual assumptions: stormy coast"));
        assert!(prepared
            .prompt_assist_note
            .contains("Manual focus cues: golden hour, cinematic framing"));
    }
}
