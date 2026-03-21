const state = {
  models: [],
  assets: [],
  outputs: [],
  runtimeStatus: null,
  hardwareProfile: null,
  gpuTelemetry: null,
  selectedReference: null,
  primaryReference: null,
  endReference: null,
  controlReference: null,
  referenceIntent: "guide",
  currentPreview: null,
  currentJobId: null,
  activeFilter: "all",
  generating: false,
  generationStyle: "expressive",
};

const MAX_RUNTIME_SEED = 4294967295;
const MODE_DEFAULTS = {
  expressive: {
    temperature: "0.6",
    steps: "28",
    cfgScale: "7.5",
    resolution: "square512",
    videoResolution: "square512",
    videoDuration: "5",
    videoFps: "8",
    lowVram: false,
  },
  realism: {
    temperature: "0.6",
    steps: "24",
    cfgScale: "6.0",
    resolution: "square512",
    videoResolution: "square256",
    videoDuration: "2",
    videoFps: "8",
    lowVram: true,
  },
};

const elements = {
  promptInput: document.getElementById("promptInput"),
  negativePromptInput: document.getElementById("negativePromptInput"),
  negativePromptBlock: document.getElementById("negativePromptBlock"),
  styleButtons: [...document.querySelectorAll("[data-style]")],
  styleSummary: document.getElementById("styleSummary"),
  runtimeBadges: document.getElementById("runtimeBadges"),
  modelSelect: document.getElementById("modelSelect"),
  modelSummary: document.getElementById("modelSummary"),
  temperatureCard: document.getElementById("temperatureCard"),
  temperatureInput: document.getElementById("temperatureInput"),
  temperatureValue: document.getElementById("temperatureValue"),
  temperatureCopy: document.getElementById("temperatureCopy"),
  stepsInput: document.getElementById("stepsInput"),
  stepsValue: document.getElementById("stepsValue"),
  cfgInput: document.getElementById("cfgInput"),
  cfgValue: document.getElementById("cfgValue"),
  resolutionInput: document.getElementById("resolutionInput"),
  videoResolutionInput: document.getElementById("videoResolutionInput"),
  videoDurationInput: document.getElementById("videoDurationInput"),
  videoDurationCopy: document.getElementById("videoDurationCopy"),
  videoFpsInput: document.getElementById("videoFpsInput"),
  videoFpsCopy: document.getElementById("videoFpsCopy"),
  lowVramCard: document.getElementById("lowVramCard"),
  lowVramInput: document.getElementById("lowVramInput"),
  lowVramCopy: document.getElementById("lowVramCopy"),
  promptAssistInput: document.getElementById("promptAssistInput"),
  seedInput: document.getElementById("seedInput"),
  refreshAll: document.getElementById("refreshAll"),
  progressFill: document.getElementById("progressFill"),
  progressPhase: document.getElementById("progressPhase"),
  progressMessage: document.getElementById("progressMessage"),
  gpuTelemetryPanel: document.getElementById("gpuTelemetryPanel"),
  gpuTelemetryLabel: document.getElementById("gpuTelemetryLabel"),
  gpuTelemetryValue: document.getElementById("gpuTelemetryValue"),
  gpuTelemetryArea: document.getElementById("gpuTelemetryArea"),
  gpuTelemetryLine: document.getElementById("gpuTelemetryLine"),
  gpuTelemetryNote: document.getElementById("gpuTelemetryNote"),
  selectedReferenceName: document.getElementById("selectedReferenceName"),
  previewSurface: document.getElementById("previewSurface"),
  historyGrid: document.getElementById("historyGrid"),
  leftColumn: document.querySelector(".left-column"),
  centerColumn: document.querySelector(".center-column"),
  assetList: document.getElementById("assetList"),
  trayPreview: document.getElementById("trayPreview"),
  referenceGuide: document.getElementById("referenceGuide"),
  referenceEdit: document.getElementById("referenceEdit"),
  referenceEnd: document.getElementById("referenceEnd"),
  referenceControl: document.getElementById("referenceControl"),
  referenceModeNote: document.getElementById("referenceModeNote"),
  referenceAssignments: document.getElementById("referenceAssignments"),
  tray: document.getElementById("tray"),
  toggleLeftColumn: document.getElementById("toggleLeftColumn"),
  toggleCenterColumn: document.getElementById("toggleCenterColumn"),
  toggleTray: document.getElementById("toggleTray"),
  showLeftColumn: document.getElementById("showLeftColumn"),
  showCenterColumn: document.getElementById("showCenterColumn"),
  showTray: document.getElementById("showTray"),
  clearReference: document.getElementById("clearReference"),
  actionButtons: [...document.querySelectorAll(".action-button")],
  trayFilters: [...document.querySelectorAll(".tray-filter")],
};

const GPU_TELEMETRY_WIDTH = 180;
const GPU_TELEMETRY_HEIGHT = 44;

bindSettingDisplay(elements.temperatureInput, elements.temperatureValue, (value) => Number(value).toFixed(1));
bindSettingDisplay(elements.stepsInput, elements.stepsValue, (value) => `${value}`);
bindSettingDisplay(elements.cfgInput, elements.cfgValue, (value) => Number(value).toFixed(1));

const trackedSettingInputs = [
  elements.temperatureInput,
  elements.stepsInput,
  elements.cfgInput,
  elements.resolutionInput,
  elements.videoResolutionInput,
  elements.videoDurationInput,
  elements.videoFpsInput,
  elements.lowVramInput,
];

elements.refreshAll.addEventListener("click", () => refreshEverything());
elements.clearReference.addEventListener("click", () => clearReferenceSlots());
elements.referenceGuide.addEventListener("click", () => assignSelectedReference("primary", "guide"));
elements.referenceEdit.addEventListener("click", () => assignSelectedReference("primary", "edit"));
elements.referenceEnd.addEventListener("click", () => assignSelectedReference("end"));
elements.referenceControl.addEventListener("click", () => assignSelectedReference("control"));
elements.toggleLeftColumn.addEventListener("click", () => toggleColumn("left", false));
elements.toggleCenterColumn.addEventListener("click", () => toggleColumn("center", false));
elements.toggleTray.addEventListener("click", () => toggleTray(false));
elements.showLeftColumn.addEventListener("click", () => toggleColumn("left", true));
elements.showCenterColumn.addEventListener("click", () => toggleColumn("center", true));
elements.showTray.addEventListener("click", () => toggleTray(true));
elements.modelSelect.addEventListener("change", () => {
  renderModelSummary();
  renderReferenceIntentControls();
  syncActionState();
});
elements.actionButtons.forEach((button) => {
  button.addEventListener("click", () => submitGeneration(button.dataset.kind));
});
elements.trayFilters.forEach((button) => {
  button.addEventListener("click", () => {
    state.activeFilter = button.dataset.filter;
    renderTrayFilters();
    renderAssets();
  });
});
elements.styleButtons.forEach((button) => {
  button.addEventListener("click", () => {
    const nextStyle = button.dataset.style;
    const previousStyle = state.generationStyle;
    if (settingsMatchPreset(previousStyle)) {
      applyModeDefaults(nextStyle);
    }
    state.generationStyle = nextStyle;
    renderStyleMode();
    renderModels();
    renderReferenceIntentControls();
    syncActionState();
  });
});

trackedSettingInputs.forEach((input) => {
  input.addEventListener("change", () => {
    state.lastAutoDefaultsStyle = null;
    refreshVideoSettingCopy();
  });
});

connectSocket();
applyModeDefaults(state.generationStyle);
renderStyleMode();
refreshEverything();
startGpuTelemetryPolling();

async function refreshEverything() {
  await Promise.all([
    loadRuntimeStatus(),
    loadHardwareProfile(),
    loadModels(),
    loadAssets(),
    loadOutputs(),
    loadGpuTelemetry(),
  ]);
}

async function loadRuntimeStatus() {
  try {
    state.runtimeStatus = await fetchJson("/api/runtime");
  } catch (error) {
    state.runtimeStatus = null;
  }
  renderStyleMode();
}

async function loadHardwareProfile() {
  try {
    state.hardwareProfile = await fetchJson("/api/hardware");
  } catch (error) {
    state.hardwareProfile = null;
  }

  renderModelSummary();
}

async function loadGpuTelemetry() {
  try {
    state.gpuTelemetry = await fetchJson("/api/telemetry/gpu");
  } catch (error) {
    state.gpuTelemetry = {
      supported: false,
      label: "GPU activity",
      note: "GPU telemetry is temporarily unavailable.",
      current_percent: 0,
      history: [],
    };
  }

  renderGpuTelemetry();
}

function startGpuTelemetryPolling() {
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) {
      loadGpuTelemetry();
    }
  });

  setInterval(() => {
    if (!document.hidden) {
      loadGpuTelemetry();
    }
  }, 1500);
}

function applyModeDefaults(style) {
  const preset = MODE_DEFAULTS[style];
  if (!preset) {
    return;
  }

  elements.temperatureInput.value = preset.temperature;
  elements.stepsInput.value = preset.steps;
  elements.cfgInput.value = preset.cfgScale;
  elements.resolutionInput.value = preset.resolution;
  elements.videoResolutionInput.value = preset.videoResolution;
  elements.videoDurationInput.value = preset.videoDuration;
  elements.videoFpsInput.value = preset.videoFps;
  elements.lowVramInput.checked = Boolean(preset.lowVram);
  refreshSettingDisplays();
}

function refreshSettingDisplays() {
  [elements.temperatureInput, elements.stepsInput, elements.cfgInput].forEach((input) => {
    input.dispatchEvent(new Event("input"));
  });
  refreshVideoSettingCopy();
}

function settingsMatchPreset(style) {
  const preset = MODE_DEFAULTS[style];
  if (!preset) {
    return false;
  }

  return (
    elements.temperatureInput.value === preset.temperature
    && elements.stepsInput.value === preset.steps
    && elements.cfgInput.value === preset.cfgScale
    && elements.resolutionInput.value === preset.resolution
    && elements.videoResolutionInput.value === preset.videoResolution
    && elements.videoDurationInput.value === preset.videoDuration
    && elements.videoFpsInput.value === preset.videoFps
    && elements.lowVramInput.checked === Boolean(preset.lowVram)
  );
}

function refreshVideoSettingCopy() {
  const seconds = Number(elements.videoDurationInput.value || 0);
  const fps = Number(elements.videoFpsInput.value || 0);
  const frames = Math.max(1, seconds * fps);
  elements.videoDurationCopy.textContent = `Used for GIF/video output. ${seconds}s at ${fps} FPS = ${frames} frames.`;
  elements.videoFpsCopy.textContent = "Playback speed for GIF/video output. Higher FPS is smoother but heavier.";
}

async function loadModels() {
  try {
    state.models = await fetchJson("/api/models");
  } catch (error) {
    state.models = [];
    setProgress(0, "Models", error.message);
  }
  renderModels();
}

async function loadAssets() {
  try {
    state.assets = await fetchJson("/api/assets");
  } catch (error) {
    state.assets = [];
  }
  reconcileAssignedAssets();
  renderAssets();
}

async function loadOutputs() {
  try {
    state.outputs = await fetchJson("/api/outputs");
  } catch (error) {
    state.outputs = [];
  }

  if (!state.currentPreview && state.outputs.length > 0) {
    state.currentPreview = state.outputs[0];
  }

  renderPreview();
  renderHistory();
}

function renderStyleMode() {
  elements.styleButtons.forEach((button) => {
    button.classList.toggle("active", button.dataset.style === state.generationStyle);
  });

  const realism = state.generationStyle === "realism";
  elements.styleSummary.textContent = realism
    ? "Realism uses a local stable-diffusion.cpp backend. Chatty-art builds sd-cli from diffuse_runtime/ on first use, then runs diffusion, GIF, and supported video jobs fully local."
    : "Expressive uses the bundled llama.cpp planner plus Chatty-art's local renderer for fast image, GIF, and audio output.";
  renderRuntimeBadges();

  elements.negativePromptBlock.classList.toggle("hidden", !realism);
  elements.temperatureCard.classList.toggle("muted-setting", realism);
  elements.temperatureInput.disabled = realism;
  elements.lowVramCard.classList.toggle("muted-setting", !realism);
  elements.lowVramInput.disabled = !realism;
  elements.temperatureCopy.textContent = realism
    ? "Expressive mode uses Temperature. Realism mode ignores it and relies mostly on steps, CFG, resolution, and seed."
    : "How creative/random. 0 stays predictable, 2 gets wild.";
  elements.lowVramCopy.textContent = realism
    ? "Helpful for realism jobs on GPUs that hit VRAM limits, especially higher resolutions and video. It is slower, but safer."
    : "Expressive mode does not use this. Realism mode can spill more work to CPU and tile VAE decode when this is enabled.";
  renderReferenceIntentControls();
}

function renderRuntimeBadges() {
  const backendStatus = state.runtimeStatus
    ? (state.generationStyle === "realism" ? state.runtimeStatus.realism : state.runtimeStatus.expressive)
    : null;

  if (!backendStatus) {
    elements.runtimeBadges.innerHTML = "";
    return;
  }

  const tone = runtimeAccelerationTone(backendStatus.acceleration);
  elements.runtimeBadges.innerHTML = `
    <span class="runtime-pill">${escapeHtml(formatBackendBadge(backendStatus.backend))}</span>
    <span class="runtime-pill runtime-${escapeHtml(tone)}">${escapeHtml(backendStatus.label)}</span>
    <span class="runtime-note">${escapeHtml(backendStatus.note)}</span>
  `;
}

function renderGpuTelemetry() {
  const telemetry = state.gpuTelemetry;
  if (!telemetry) {
    elements.gpuTelemetryPanel.classList.add("hidden");
    return;
  }

  const label = String(telemetry.label || "GPU activity").trim() || "GPU activity";
  const note = String(telemetry.note || "Shows the busiest local GPU engine.").trim();
  const currentPercent = clampPercent(telemetry.current_percent);
  const history = normalizeGpuHistory(telemetry.history, currentPercent);

  elements.gpuTelemetryPanel.classList.remove("hidden");
  elements.gpuTelemetryLabel.textContent = label;
  elements.gpuTelemetryValue.textContent = `${Math.round(currentPercent)}%`;
  elements.gpuTelemetryNote.textContent = note;
  elements.gpuTelemetryLine.setAttribute("points", buildSparklinePoints(history));
  elements.gpuTelemetryArea.setAttribute("d", buildSparklineArea(history));
}

function normalizeGpuHistory(history, fallbackPercent) {
  const values = Array.isArray(history)
    ? history.map((value) => clampPercent(value)).filter((value) => Number.isFinite(value))
    : [];

  if (!values.length) {
    return [clampPercent(fallbackPercent), clampPercent(fallbackPercent)];
  }

  if (values.length === 1) {
    return [values[0], values[0]];
  }

  return values;
}

function buildSparklinePoints(history) {
  return history
    .map((value, index) => {
      const x = history.length === 1
        ? GPU_TELEMETRY_WIDTH
        : (index / (history.length - 1)) * GPU_TELEMETRY_WIDTH;
      const y = percentToSparklineY(value);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function buildSparklineArea(history) {
  const points = history.map((value, index) => {
    const x = history.length === 1
      ? GPU_TELEMETRY_WIDTH
      : (index / (history.length - 1)) * GPU_TELEMETRY_WIDTH;
    const y = percentToSparklineY(value);
    return `${x.toFixed(1)} ${y.toFixed(1)}`;
  });

  return `M 0 ${GPU_TELEMETRY_HEIGHT} L ${points.join(" L ")} L ${GPU_TELEMETRY_WIDTH} ${GPU_TELEMETRY_HEIGHT} Z`;
}

function percentToSparklineY(percent) {
  const clamped = clampPercent(percent);
  const innerHeight = GPU_TELEMETRY_HEIGHT - 6;
  return 3 + ((100 - clamped) / 100) * innerHeight;
}

function clampPercent(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    return 0;
  }
  return Math.max(0, Math.min(100, numeric));
}

function renderModels() {
  const selected = elements.modelSelect.value;
  const visibleModels = getVisibleModels();
  const supportedModels = visibleModels.filter((model) => model.runtime_supported);
  const unsupportedModels = visibleModels.filter((model) => !model.runtime_supported);
  const hiddenModeCount = state.models.length - visibleModels.length;

  if (!state.models.length) {
    elements.modelSelect.innerHTML = `<option value="">No GGUF models found in models/</option>`;
    elements.modelSelect.disabled = true;
    renderModelNotice("Drop one or more .gguf files into models/ and press Refresh Files.");
    renderReferenceIntentControls();
    syncActionState();
    return;
  }

  if (!visibleModels.length) {
    const label = state.generationStyle === "realism" ? "No realism models found" : "No expressive models found";
    elements.modelSelect.innerHTML = `<option value="">${label}</option>`;
    elements.modelSelect.disabled = true;
    renderModelNotice(
      state.generationStyle === "realism"
        ? "Realism mode needs diffusion-style GGUFs plus any companion weights they require in models/. Switch to Expressive to use regular llama.cpp models."
        : "Expressive mode uses regular llama.cpp-compatible models. Switch to Realism for diffusion/video GGUFs."
    );
    renderReferenceIntentControls();
    syncActionState();
    return;
  }

  if (!supportedModels.length) {
    elements.modelSelect.disabled = false;
    elements.modelSelect.innerHTML = `
      <option value="">No ready-to-run ${escapeHtml(state.generationStyle)} models</option>
      ${unsupportedModels
        .map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildDropdownLabel(model))}</option>`)
        .join("")}
    `;
    if (selected && unsupportedModels.some((model) => model.id === selected)) {
      elements.modelSelect.value = selected;
    } else if (unsupportedModels.length) {
      elements.modelSelect.value = unsupportedModels[0].id;
    }
    renderModelSummary(hiddenModeCount);
    renderReferenceIntentControls();
    syncActionState();
    return;
  }

  elements.modelSelect.disabled = false;
  const supportedOptions = supportedModels
    .map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildDropdownLabel(model))}</option>`)
    .join("");
  const unsupportedOptions = unsupportedModels.length
    ? `<optgroup label="Detected but not ready">${unsupportedModels
        .map((model) => `<option value="" disabled>${escapeHtml(buildDropdownLabel(model))}</option>`)
        .join("")}</optgroup>`
    : "";

  elements.modelSelect.innerHTML = `<optgroup label="Ready to run">${supportedOptions}</optgroup>${unsupportedOptions}`;

  if (selected && supportedModels.some((model) => model.id === selected)) {
    elements.modelSelect.value = selected;
  } else {
    elements.modelSelect.value = supportedModels[0].id;
  }

  normalizeAssignedReferencesForCurrentModel();
  renderModelSummary(hiddenModeCount);
  renderReferenceIntentControls();
  syncActionState();
}

function renderModelSummary(hiddenModeCount = null) {
  const model = getSelectedModel();
  if (!model) {
    const invisibleCount = hiddenModeCount ?? state.models.filter((entry) => entry.generation_style !== state.generationStyle).length;
    renderModelNotice(
      invisibleCount > 0
        ? `No compatible ${state.generationStyle} model selected. ${invisibleCount} file(s) belong to the other mode.`
        : `No compatible ${state.generationStyle} model selected.`,
      invisibleCount
    );
    return;
  }

  const invisibleCount = hiddenModeCount ?? state.models.filter((entry) => entry.generation_style !== state.generationStyle).length;
  const stateInfo = describeModelState(model);
  const badges = [
    createModelBadge(stateInfo.label, `state-${stateInfo.tone}`),
    createModelBadge(formatBackendBadge(model.backend), "backend"),
    createModelBadge(model.family, "family"),
  ];

  if ((model.supported_kinds || []).length) {
    badges.push(createModelBadge(`Outputs: ${formatKinds(model.supported_kinds)}`, "outputs"));
  }
  if (model.requires_reference) {
    badges.push(createModelBadge("Reference required", "reference"));
  } else if (model.supports_image_reference) {
    badges.push(createModelBadge("Image refs optional", "reference"));
  }
  if (model.requires_end_image_reference) {
    badges.push(createModelBadge("End frame required", "reference"));
  } else if (model.supports_end_image_reference) {
    badges.push(createModelBadge("End frame optional", "reference"));
  }
  if (model.supports_video_reference) {
    badges.push(createModelBadge("Control video optional", "reference"));
  }
  if (model.supports_audio_reference) {
    badges.push(createModelBadge("Audio refs optional", "reference"));
  }

  const hiddenNote = invisibleCount > 0
    ? `<div class="model-summary-foot">${escapeHtml(`${invisibleCount} file(s) belong to the other mode and are hidden right now.`)}</div>`
    : "";
  const recommendations = buildRecommendedLimitsMarkup(model);

  elements.modelSummary.innerHTML = `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(model.name)}</strong>
      </div>
      <div class="model-badges">${badges.join("")}</div>
      <div class="model-summary-copy">${escapeHtml(model.compatibility_note)}</div>
      ${recommendations}
      ${hiddenNote}
    </div>
  `;
}

function buildRecommendedLimitsMarkup(model) {
  const hardware = state.hardwareProfile;
  if (!hardware || !model) {
    return "";
  }

  const rows = (model.supported_kinds || [])
    .map((kind) => buildKindRecommendation(model, kind, hardware))
    .filter(Boolean);

  if (!rows.length) {
    return "";
  }

  const hardwareBits = [
    hardware.gpu_label || "Local GPU",
    hardware.dedicated_vram_gb ? `${formatOneDecimal(hardware.dedicated_vram_gb)} GB dedicated` : null,
    hardware.shared_memory_gb ? `${formatOneDecimal(hardware.shared_memory_gb)} GB shared` : null,
  ].filter(Boolean);

  return `
    <div class="recommended-limits">
      <div class="recommended-limits-head">
        <strong>Recommended Limits On This Hardware</strong>
        <span>${escapeHtml(hardwareBits.join(" | "))}</span>
      </div>
      <div class="recommended-limits-list">
        ${rows.map((row) => `
          <div class="recommended-limit-row">
            <strong>${escapeHtml(row.kind)}</strong>
            <span><em>Safe:</em> ${escapeHtml(row.safe)}</span>
            <span><em>Stretch:</em> ${escapeHtml(row.stretch)}</span>
            <span><em>Risky:</em> ${escapeHtml(row.risky)}</span>
          </div>
        `).join("")}
      </div>
      <div class="recommended-limits-note">${escapeHtml(hardware.note || "Recommendations are heuristics based on the current machine and selected model.")}</div>
    </div>
  `;
}

function buildKindRecommendation(model, kind, hardware) {
  const family = String(model.family || "").toLowerCase();
  const name = String(model.name || "").toLowerCase();
  const dedicated = Number(hardware.dedicated_vram_gb || 0);
  const lowVram = Boolean(elements.lowVramInput.checked);
  const sizeHint = parseModelSizeHint(model.name);
  const isExpressive = model.backend === "llama_cpp";
  const isWan = family.includes("wan");
  const isFlux = family.includes("flux");
  const isDiffusion = model.backend === "stable_diffusion_cpp";
  const smallWan = isWan && sizeHint <= 20;

  if (isExpressive) {
    switch (kind) {
      case "image":
        return {
          kind: "Image",
          safe: "Square 512 or Square 768",
          stretch: "Landscape 1280x720 if you do not mind slower planning",
          risky: "Very large expressive scenes mainly cost time, not VRAM",
        };
      case "gif":
        return {
          kind: "GIF",
          safe: "512x512 | 2s to 5s | 8 to 16 FPS",
          stretch: "512x512 | 10s | 16 FPS",
          risky: "Long GIFs become slow because planning and local rendering both scale up",
        };
      case "audio":
        return {
          kind: "Audio",
          safe: "Default sliders are fine on this machine",
          stretch: "Longer prompts and more steps are usually okay",
          risky: "Large voice-style models mainly cost time, not GPU memory",
        };
      default:
        return null;
    }
  }

  if (kind === "image") {
    if (isWan) {
      return {
        kind: "Image",
        safe: smallWan ? "Square 512 or Square 768" : "Square 512",
        stretch: smallWan ? "Landscape 1280x720 may work, but start with Square 768" : "Square 768",
        risky: "Higher resolutions can still fail during Vulkan decode on this GPU class",
      };
    }
    if (isFlux) {
      return {
        kind: "Image",
        safe: dedicated >= 8 ? "Square 512 or Square 768" : "Square 512",
        stretch: dedicated >= 8 ? "Landscape 1280x720" : "Square 768",
        risky: "Poster-size renders are more likely to spill into OOM territory than SD1.5/2.1",
      };
    }
    if (isDiffusion) {
      return {
        kind: "Image",
        safe: "Square 512",
        stretch: dedicated >= 8 ? "Square 768" : "Square 512 only",
        risky: "1024-class renders can hit contiguous Vulkan allocation limits",
      };
    }
  }

  if (kind === "gif") {
    if (isWan) {
      return {
        kind: "GIF",
        safe: smallWan ? "256x256 | 2s to 5s | 8 FPS" : "256x256 | 2s | 8 FPS",
        stretch: "512x512 | 2s | 8 FPS",
        risky: "768x768 or long clips can overflow Vulkan buffers even with Low VRAM mode on",
      };
    }
    if (isFlux || isDiffusion) {
      return {
        kind: "GIF",
        safe: "256x256 | 2s to 5s | 8 FPS",
        stretch: "512x512 | 2s | 8 FPS",
        risky: "Longer clips behave more like video memory pressure than still-image pressure",
      };
    }
  }

  if (kind === "video") {
    if (isWan) {
      return {
        kind: "Video",
        safe: smallWan
          ? "256x256 | 2s to 5s | 8 FPS"
          : "256x256 | 2s | 8 FPS",
        stretch: lowVram
          ? "512x512 | 2s | 8 FPS"
          : "512x512 | 2s | 8 FPS only after enabling Low VRAM mode",
        risky: "768x768, long durations, or high FPS are likely to OOM on this GPU",
      };
    }

    return {
      kind: "Video",
      safe: "256x256 | 2s | 8 FPS",
      stretch: "512x512 | 2s | 8 FPS",
      risky: "Large frame counts scale brutally with Vulkan memory use",
    };
  }

  if (kind === "audio") {
    return {
      kind: "Audio",
      safe: "Audio generation is not GPU-limited in the same way as realism video",
      stretch: "Longer prompts and more steps mainly cost time",
      risky: "Very large expressive models can still be slow to plan",
    };
  }

  return null;
}

function parseModelSizeHint(name) {
  const match = String(name || "").match(/(\d+(?:\.\d+)?)\s*[bB]/);
  if (!match) {
    return 9999;
  }
  return Math.round(Number(match[1]) * 10);
}

function formatOneDecimal(value) {
  return Number(value).toFixed(1);
}

function renderAssets() {
  const assets = state.assets.filter((asset) => {
    return state.activeFilter === "all" ? true : asset.kind === state.activeFilter;
  });

  if (!assets.length) {
    elements.assetList.innerHTML = `<div class="tray-empty">No matching files found. Put files into <code>input/images</code>, <code>input/video</code>, or <code>input/audio</code>, then press Refresh Files.</div>`;
  } else {
    elements.assetList.innerHTML = assets
      .map((asset) => {
        const active = state.selectedReference?.id === asset.id ? "active" : "";
        return `
          <button class="asset-card ${active}" type="button" data-asset-id="${escapeHtml(asset.id)}">
            <strong>${escapeHtml(asset.name)}</strong>
            <span>${escapeHtml(formatKind(asset.kind))} reference</span>
            <span>${escapeHtml(asset.relative_path)}</span>
          </button>
        `;
      })
      .join("");

    elements.assetList.querySelectorAll("[data-asset-id]").forEach((button) => {
      button.addEventListener("click", () => {
        const asset = state.assets.find((entry) => entry.id === button.dataset.assetId);
        setSelectedReference(asset || null);
      });
    });
  }

  renderTrayFilters();
  renderReferenceIntentControls();
  renderAssignedReferences();
  renderTrayPreview();
}

function renderTrayFilters() {
  elements.trayFilters.forEach((button) => {
    button.classList.toggle("active", button.dataset.filter === state.activeFilter);
  });
}

function renderTrayPreview() {
  if (!state.selectedReference) {
    elements.trayPreview.innerHTML = `<div class="tray-empty">Choose a file from the Input Tray to use it as a reference or edit source.</div>`;
    return;
  }

  const asset = state.selectedReference;
  const media = createMediaMarkup(asset, "tray-media");
  const assignments = [];
  if (state.primaryReference?.id === asset.id) {
    assignments.push(`Start image | ${referenceIntentLabel(state.referenceIntent)}`);
  }
  if (state.endReference?.id === asset.id) {
    assignments.push("End frame");
  }
  if (state.controlReference?.id === asset.id) {
    assignments.push("Control video");
  }
  elements.trayPreview.innerHTML = `
    <strong>${escapeHtml(asset.name)}</strong>
    <span>${escapeHtml(asset.relative_path)}</span>
    <span>${escapeHtml(assignments.length ? `Assigned as: ${assignments.join(" | ")}` : "Not assigned to a slot yet.")}</span>
    ${media}
  `;
}

function renderReferenceIntentControls() {
  const context = getReferenceAssignmentContext();
  const selectedAssetId = state.selectedReference?.id;

  elements.referenceGuide.classList.toggle(
    "active",
    state.primaryReference?.id === selectedAssetId && state.referenceIntent === "guide"
  );
  elements.referenceEdit.classList.toggle(
    "active",
    state.primaryReference?.id === selectedAssetId && state.referenceIntent === "edit"
  );
  elements.referenceEnd.classList.toggle("active", state.endReference?.id === selectedAssetId);
  elements.referenceControl.classList.toggle("active", state.controlReference?.id === selectedAssetId);

  elements.referenceGuide.disabled = !context.guideEnabled;
  elements.referenceEdit.disabled = !context.editEnabled;
  elements.referenceEnd.disabled = !context.endEnabled;
  elements.referenceControl.disabled = !context.controlEnabled;
  elements.referenceModeNote.textContent = context.message;
  renderAssignedReferences();
}

function getReferenceAssignmentContext() {
  const model = getSelectedModel();
  const asset = state.selectedReference;

  if (!asset) {
    return {
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: "Choose a file first. Guide/Edit assign a start image. End Frame assigns the final still. Control Video assigns a motion guide from input/video/.",
    };
  }

  if (!model) {
    return {
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: "Choose a model first so Chatty-art can match the selected file to a backend.",
    };
  }

  if (state.generationStyle === "expressive") {
    return {
      guideEnabled: true,
      editEnabled: true,
      endEnabled: false,
      controlEnabled: false,
      message:
        asset.kind === "audio"
          ? "Expressive mode can treat the selected audio as either a guide or an edit/source cue for planning."
          : "Expressive mode can treat the selected file as either a guide or an edit/source cue for planning. End-frame and control-video slots are realism-only.",
    };
  }

  if (!model.runtime_supported
      && !model.supports_image_reference
      && !model.requires_reference
      && !model.supports_end_image_reference
      && !model.requires_end_image_reference
      && !model.supports_video_reference) {
    return {
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: "This realism model is not ready yet, so Input Tray assignments are disabled.",
    };
  }

  if (asset.kind === "image") {
    const guideEnabled = model.supports_image_reference || model.requires_reference;
    const endEnabled = model.supports_end_image_reference || model.requires_end_image_reference;
    return {
      guideEnabled,
      editEnabled: guideEnabled,
      endEnabled,
      controlEnabled: false,
      message: endEnabled
        ? "Assign this still image as the start image, edit source, or end frame depending on the selected realism model."
        : "Assign this still image as a guide or edit source for realism generation.",
    };
  }

  if (asset.kind === "video") {
    return {
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: model.supports_video_reference,
      message: model.supports_video_reference
        ? "Assign this video or GIF as the control-video guide for a VACE-style realism run."
        : "This realism model does not use control-video guidance in Chatty-art yet.",
    };
  }

  return {
    guideEnabled: false,
    editEnabled: false,
    endEnabled: false,
    controlEnabled: false,
    message: "Realism uses still images for start/end frames and video or GIF files for control-video guidance.",
  };
}

function renderPreview() {
  const item = state.currentPreview;
  if (!item) {
    elements.previewSurface.classList.add("empty");
    elements.previewSurface.innerHTML = `
      <div class="empty-state">
        <strong>No output yet</strong>
        <p>Your latest image, video, or audio file will appear here with inline preview controls.</p>
      </div>
    `;
    return;
  }

  elements.previewSurface.classList.remove("empty");
  elements.previewSurface.innerHTML = `
    ${createMediaMarkup(item, "preview-media")}
    <div class="preview-meta">
      <h3>${escapeHtml(item.file_name)}</h3>
      <span>${escapeHtml(formatKind(item.kind))} | ${escapeHtml(item.model || "Unknown model")} | ${escapeHtml(item.style || "expressive")} mode</span>
      <p>${escapeHtml(item.prompt || "Saved output")}</p>
      ${item.resolution_label ? `<p><strong>Output settings:</strong> ${escapeHtml(item.resolution_label)}</p>` : ""}
      ${item.negative_prompt ? `<p><strong>Negative prompt:</strong> ${escapeHtml(item.negative_prompt)}</p>` : ""}
      ${item.reference_asset ? `<p><strong>Reference use:</strong> ${escapeHtml(referenceIntentLabel(item.reference_intent || "guide"))} via ${escapeHtml(item.reference_asset)}</p>` : ""}
      ${item.end_reference_asset ? `<p><strong>End frame:</strong> ${escapeHtml(item.end_reference_asset)}</p>` : ""}
      ${item.control_reference_asset ? `<p><strong>Control video:</strong> ${escapeHtml(item.control_reference_asset)}</p>` : ""}
      ${item.compiled_prompt ? `<p><strong>Compiled brief:</strong> ${escapeHtml(item.compiled_prompt)}</p>` : ""}
      ${item.prompt_assist && item.prompt_assist !== "off" ? `<p><strong>Prompt Assist:</strong> ${escapeHtml(item.prompt_assist)}${item.interpreter_model ? ` via ${escapeHtml(item.interpreter_model)}` : ""}</p>` : ""}
      <p>${escapeHtml(item.note || "")}</p>
    </div>
  `;
}

function renderHistory() {
  if (!state.outputs.length) {
    elements.historyGrid.innerHTML = `<div class="history-card"><div class="history-copy"><strong>No saved outputs yet</strong><span>Generated files will appear here as soon as the first job finishes.</span></div></div>`;
    return;
  }

  elements.historyGrid.innerHTML = state.outputs
    .map((output) => {
      const preview = createHistoryPreview(output);
      return `
        <div class="history-card">
          <button type="button" data-output-id="${escapeHtml(output.id)}">
            ${preview}
            <div class="history-copy">
              <strong>${escapeHtml(output.file_name)}</strong>
              <span>${escapeHtml(output.model || "Unknown model")}</span>
            </div>
          </button>
        </div>
      `;
    })
    .join("");

  elements.historyGrid.querySelectorAll("[data-output-id]").forEach((button) => {
    button.addEventListener("click", () => {
      const output = state.outputs.find((entry) => entry.id === button.dataset.outputId);
      if (output) {
        state.currentPreview = output;
        renderPreview();
      }
    });
  });
}

function renderAssignedReferences() {
  const slots = [
    {
      key: "primary",
      label: "Primary input",
      asset: state.primaryReference,
      detail: state.primaryReference
        ? `${referenceIntentLabel(state.referenceIntent)}`
        : "No primary guide/edit input assigned.",
    },
    {
      key: "end",
      label: "End frame",
      asset: state.endReference,
      detail: state.endReference ? "Used as the final still frame for FLF2V-style video generation." : "No end frame assigned.",
    },
    {
      key: "control",
      label: "Control video",
      asset: state.controlReference,
      detail: state.controlReference ? "Used as motion guidance for VACE-style video generation." : "No control video assigned.",
    },
  ];

  elements.referenceAssignments.innerHTML = slots
    .map((slot) => `
      <div class="reference-assignment">
        <div class="reference-assignment-copy">
          <strong>${escapeHtml(slot.label)}</strong>
          <span>${escapeHtml(slot.asset ? `${slot.asset.name} | ${slot.detail}` : slot.detail)}</span>
        </div>
        <button class="ghost-button reference-clear" type="button" data-reference-slot="${escapeHtml(slot.key)}">Clear</button>
      </div>
    `)
    .join("");

  elements.referenceAssignments.querySelectorAll("[data-reference-slot]").forEach((button) => {
    button.addEventListener("click", () => clearReferenceSlot(button.dataset.referenceSlot));
  });

  elements.selectedReferenceName.textContent = formatAssignedReferenceBanner();
}

function setSelectedReference(asset) {
  state.selectedReference = asset;
  renderAssets();
}

function assignSelectedReference(slot, intent = state.referenceIntent) {
  if (!state.selectedReference) {
    return;
  }

  if (slot === "primary") {
    state.primaryReference = state.selectedReference;
    state.referenceIntent = intent;
  } else if (slot === "end") {
    state.endReference = state.selectedReference;
  } else if (slot === "control") {
    state.controlReference = state.selectedReference;
  }

  renderReferenceIntentControls();
  renderTrayPreview();
  syncActionState();
}

function clearReferenceSlot(slot) {
  if (slot === "primary") {
    state.primaryReference = null;
  } else if (slot === "end") {
    state.endReference = null;
  } else if (slot === "control") {
    state.controlReference = null;
  }

  renderReferenceIntentControls();
  renderTrayPreview();
  syncActionState();
}

function clearReferenceSlots() {
  state.primaryReference = null;
  state.endReference = null;
  state.controlReference = null;
  renderReferenceIntentControls();
  renderTrayPreview();
  syncActionState();
}

function formatAssignedReferenceBanner() {
  const primary = state.primaryReference
    ? `${state.primaryReference.name} (${referenceIntentLabel(state.referenceIntent)})`
    : "none";
  const end = state.endReference ? state.endReference.name : "none";
  const control = state.controlReference ? state.controlReference.name : "none";
  return `Primary: ${primary} | End: ${end} | Control: ${control}`;
}

function reconcileAssignedAssets() {
  const keepAsset = (asset) => {
    if (!asset) {
      return null;
    }
    return state.assets.find((entry) => entry.id === asset.id) || null;
  };

  state.selectedReference = keepAsset(state.selectedReference);
  state.primaryReference = keepAsset(state.primaryReference);
  state.endReference = keepAsset(state.endReference);
  state.controlReference = keepAsset(state.controlReference);
}

function normalizeAssignedReferencesForCurrentModel() {
  const model = getSelectedModel();

  if (state.generationStyle !== "realism") {
    state.endReference = null;
    state.controlReference = null;
    return;
  }

  if (!model) {
    return;
  }

  if (state.primaryReference) {
    const primarySupported =
      state.primaryReference.kind === "image" &&
      (model.supports_image_reference || model.requires_reference);
    if (!primarySupported) {
      state.primaryReference = null;
    }
  }

  if (state.endReference) {
    const endSupported =
      state.endReference.kind === "image" &&
      (model.supports_end_image_reference || model.requires_end_image_reference);
    if (!endSupported) {
      state.endReference = null;
    }
  }

  if (state.controlReference) {
    const controlSupported =
      state.controlReference.kind === "video" && model.supports_video_reference;
    if (!controlSupported) {
      state.controlReference = null;
    }
  }
}

function toggleTray(open) {
  elements.tray.classList.toggle("hidden", !open);
  elements.showTray.classList.toggle("hidden", open);
}

function toggleColumn(column, open) {
  if (column === "left") {
    elements.leftColumn.classList.toggle("hidden", !open);
    elements.showLeftColumn.classList.toggle("hidden", open);
    return;
  }

  if (column === "center") {
    elements.centerColumn.classList.toggle("hidden", !open);
    elements.showCenterColumn.classList.toggle("hidden", open);
  }
}

function syncActionState() {
  const model = getSelectedModel();
  const assignedReferencesValid = areAssignedReferencesCompatible(model);
  elements.actionButtons.forEach((button) => {
    const supported = model && model.runtime_supported && kindSupported(model, button.dataset.kind);
    button.disabled = state.generating || !supported || !assignedReferencesValid;
  });
}

function buildAcceptedMessage(model, kind) {
  const assistMode = elements.promptAssistInput.value;
  const assistNote = assistMode === "off"
    ? ""
    : ` Prompt Assist (${assistMode}) will compile a richer local brief first.`;
  const kindLabel = formatKind(kind).toLowerCase();

  if (state.generationStyle === "realism") {
    return `Job accepted. Starting the local realism pipeline for ${kindLabel}. The first realism run can take longer while stable-diffusion.cpp gets ready.${assistNote}`;
  }

  const largePlanner =
    /\b(14b|20b|22b|32b|70b)\b/i.test(model.name) || /\b(gpt-oss|qwq)\b/i.test(model.name);
  if (largePlanner) {
    return `Job accepted. Starting local planning with ${model.name} for ${kindLabel}. Bigger GGUFs can spend a few minutes planning before rendering begins.${assistNote}`;
  }

  return `Job accepted. Starting local planning for ${kindLabel}. The first progress update may take a few seconds.${assistNote}`;
}

async function submitGeneration(kind) {
  const model = getSelectedModel();
  if (!model) {
    setProgress(0, "Model", `Choose a ${state.generationStyle} model first.`);
    return;
  }

  if (!kindSupported(model, kind)) {
    setProgress(0, "Mode", `${model.name} does not currently support ${kind} generation in ${state.generationStyle} mode.`);
    return;
  }

  if (!areAssignedReferencesCompatible(model)) {
    const message = getAssignedReferenceValidationMessage(model);
    setProgress(0, "Reference", message);
    return;
  }

  const prompt = elements.promptInput.value.trim();
  if (!prompt) {
    setProgress(0, "Prompt", "Type a prompt first.");
    elements.promptInput.focus();
    return;
  }

  state.generating = true;
  syncActionState();
  setProgress(
    0.04,
    "Queued",
    state.generationStyle === "realism"
      ? `Submitting ${kind} job to the local stable-diffusion.cpp realism backend.`
      : `Submitting ${kind} job to the bundled expressive backend.`
  );

  let seed = null;
  try {
    seed = parseSeedInput();
  } catch (error) {
    state.generating = false;
    syncActionState();
    setProgress(0, "Seed", error.message);
    elements.seedInput.focus();
    return;
  }

  const negativePrompt = elements.negativePromptInput.value.trim();
  const payload = {
    prompt,
    negative_prompt: negativePrompt ? negativePrompt : null,
    prompt_assist: elements.promptAssistInput.value,
    model: model.id,
    kind,
    style: state.generationStyle,
      settings: {
        temperature: Number(elements.temperatureInput.value),
        steps: Number(elements.stepsInput.value),
        cfg_scale: Number(elements.cfgInput.value),
        resolution: elements.resolutionInput.value,
        video_resolution: elements.videoResolutionInput.value,
        video_duration_seconds: Number(elements.videoDurationInput.value),
        video_fps: Number(elements.videoFpsInput.value),
        low_vram_mode: state.generationStyle === "realism" && elements.lowVramInput.checked,
        seed,
      },
    reference_asset: state.primaryReference ? state.primaryReference.id : null,
    reference_intent: state.referenceIntent,
    end_reference_asset: state.endReference ? state.endReference.id : null,
    control_reference_asset: state.controlReference ? state.controlReference.id : null,
  };

  try {
    const response = await fetch("/api/generate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const message = await response.text();
      throw new Error(message || "Generation request failed.");
    }

    const accepted = await response.json();
    state.currentJobId = accepted.job_id;
    if (accepted.used_seed !== undefined && accepted.used_seed !== null) {
      elements.seedInput.value = String(accepted.used_seed);
    }
    setProgress(0.08, "Starting", buildAcceptedMessage(model, kind));
  } catch (error) {
    state.generating = false;
    syncActionState();
    setProgress(0, "Error", error.message || "Generation request failed.");
  }
}

function connectSocket() {
  const protocol = location.protocol === "https:" ? "wss" : "ws";
  const socket = new WebSocket(`${protocol}://${location.host}/ws`);

  socket.addEventListener("message", (event) => {
    const payload = JSON.parse(event.data);
    handleServerEvent(payload);
  });

  socket.addEventListener("close", () => {
    setTimeout(connectSocket, 1500);
  });
}

function handleServerEvent(event) {
  if (event.type === "progress" && event.job_id === state.currentJobId) {
    setProgress(event.percent, event.phase, event.message);
    return;
  }

  if (event.type === "completed") {
    upsertOutput(event.output);
    state.currentPreview = event.output;
    renderPreview();
    renderHistory();

    if (event.job_id === state.currentJobId) {
      state.generating = false;
      state.currentJobId = null;
      syncActionState();
      setProgress(1, "Complete", `${formatKind(event.output.kind)} saved to outputs/.`);
    }
    return;
  }

  if (event.type === "error" && event.job_id === state.currentJobId) {
    state.generating = false;
    state.currentJobId = null;
    syncActionState();
    setProgress(0, "Error", event.message || "Generation failed.");
  }
}

function upsertOutput(output) {
  const index = state.outputs.findIndex((entry) => entry.id === output.id);
  if (index === -1) {
    state.outputs.unshift(output);
  } else {
    state.outputs.splice(index, 1, output);
  }
}

function getVisibleModels() {
  return state.models.filter((model) => model.generation_style === state.generationStyle);
}

function getSelectedModel() {
  return getVisibleModels().find((model) => model.id === elements.modelSelect.value) || null;
}

function buildDropdownLabel(model) {
  const stateInfo = describeModelState(model);
  const parts = [stateInfo.shortLabel, model.name, model.family];
  if ((model.supported_kinds || []).length) {
    parts.push(formatKinds(model.supported_kinds));
  }
  if (model.requires_reference) {
    parts.push("Reference needed");
  } else if (model.supports_image_reference) {
    parts.push("Image refs");
  }
  if (model.requires_end_image_reference) {
    parts.push("End frame needed");
  } else if (model.supports_end_image_reference) {
    parts.push("End frame");
  }
  if (model.supports_video_reference) {
    parts.push("Control video");
  }
  return parts.join(" | ");
}

function runtimeAccelerationTone(acceleration) {
  switch (acceleration) {
    case "vulkan":
      return "vulkan";
    case "cpu_only":
      return "cpu";
    case "build_pending":
      return "pending";
    case "incomplete_tree":
      return "warning";
    default:
      return "neutral";
  }
}

function describeModelState(model) {
  const note = String(model.compatibility_note || "").toLowerCase();
  const family = String(model.family || "").toLowerCase();

  if (model.runtime_supported) {
    if (model.requires_reference) {
      return { label: "Ready, needs reference", shortLabel: "READY + REF", tone: "ready-ref" };
    }
    return { label: "Ready to run", shortLabel: "READY", tone: "ready" };
  }

  if (family.includes("companion") || note.includes("helper weight")) {
    return { label: "Companion file", shortLabel: "COMPANION", tone: "companion" };
  }

  if (note.includes("missing:")) {
    return { label: "Needs local files", shortLabel: "NEEDS FILES", tone: "needs-files" };
  }

  if (note.includes("not wired")) {
    return { label: "Adapter not wired yet", shortLabel: "NOT WIRED", tone: "unsupported" };
  }

  if (note.includes("not supported") || note.includes("not support") || note.includes("does not recognize") || note.includes("unsupported")) {
    return { label: "Unsupported by current runtime", shortLabel: "UNSUPPORTED", tone: "unsupported" };
  }

  return { label: "Detected, not ready", shortLabel: "DETECTED", tone: "detected" };
}

function createModelBadge(label, tone) {
  return `<span class="model-pill ${escapeHtml(tone)}">${escapeHtml(label)}</span>`;
}

function renderModelNotice(message, hiddenModeCount = 0) {
  const hiddenNote = hiddenModeCount > 0
    ? `<div class="model-summary-foot">${escapeHtml(`${hiddenModeCount} file(s) belong to the other mode and are hidden right now.`)}</div>`
    : "";
  elements.modelSummary.innerHTML = `
    <div class="model-summary-card">
      <div class="model-summary-copy">${escapeHtml(message)}</div>
      ${hiddenNote}
    </div>
  `;
}

function kindSupported(model, kind) {
  return (model.supported_kinds || []).includes(kind);
}

function areAssignedReferencesCompatible(model) {
  if (!model) {
    return true;
  }

  if (state.generationStyle !== "realism") {
    return true;
  }

  if (model.requires_reference && !state.primaryReference) {
    return false;
  }

  if (state.primaryReference) {
    if (state.primaryReference.kind !== "image") {
      return false;
    }
    if (!model.supports_image_reference && !model.requires_reference) {
      return false;
    }
  }

  if (model.requires_end_image_reference && !state.endReference) {
    return false;
  }

  if (state.endReference) {
    if (state.endReference.kind !== "image") {
      return false;
    }
    if (!model.supports_end_image_reference && !model.requires_end_image_reference) {
      return false;
    }
  }

  if (state.controlReference) {
    if (state.controlReference.kind !== "video") {
      return false;
    }
    if (!model.supports_video_reference) {
      return false;
    }
  }

  return true;
}

function getAssignedReferenceValidationMessage(model) {
  if (!model) {
    return "Choose a model first.";
  }

  if (state.generationStyle !== "realism") {
    return "The selected reference setup is not compatible with the current model.";
  }

  if (model.requires_reference && !state.primaryReference) {
    return "This realism model needs a start image in the Input Tray before it can generate.";
  }

  if (state.primaryReference && state.primaryReference.kind !== "image") {
    return "The start image must be a still image from input/images/.";
  }

  if (state.primaryReference && !model.supports_image_reference && !model.requires_reference) {
    return "This realism model does not use a start image in Chatty-art yet.";
  }

  if (model.requires_end_image_reference && !state.endReference) {
    return "This realism model needs an end image in the Input Tray before it can generate.";
  }

  if (state.endReference && state.endReference.kind !== "image") {
    return "The end frame must be a still image from input/images/.";
  }

  if (state.endReference && !model.supports_end_image_reference && !model.requires_end_image_reference) {
    return "This realism model does not use an end-frame image in Chatty-art yet.";
  }

  if (state.controlReference && state.controlReference.kind !== "video") {
    return "Control-video input must come from input/video/. GIFs are supported there too.";
  }

  if (state.controlReference && !model.supports_video_reference) {
    return "This realism model does not use control-video guidance in Chatty-art yet.";
  }

  return "The assigned Input Tray files are not compatible with the current model.";
}

function setProgress(percent, phase, message) {
  elements.progressFill.style.width = `${Math.max(0, Math.min(1, percent)) * 100}%`;
  elements.progressPhase.textContent = phase;
  elements.progressMessage.textContent = message;
}

function bindSettingDisplay(input, label, formatter) {
  const sync = () => {
    label.textContent = formatter(input.value);
  };
  input.addEventListener("input", sync);
  sync();
}

function parseSeedInput() {
  const raw = elements.seedInput.value.trim();
  if (!raw) {
    return null;
  }

  const seed = Number(raw);
  if (!Number.isInteger(seed) || seed < 0 || seed > MAX_RUNTIME_SEED) {
    throw new Error(`Seed must be a whole number between 0 and ${MAX_RUNTIME_SEED}.`);
  }

  return seed;
}

function createMediaMarkup(item, className) {
  const mime = String(item.mime || "");

  if (item.kind === "image") {
    return `<img class="${className}" src="${escapeAttribute(item.url)}" alt="${escapeAttribute(item.file_name || item.name)}">`;
  }

  if (item.kind === "gif") {
    return `<img class="${className}" src="${escapeAttribute(item.url)}" alt="${escapeAttribute(item.file_name || item.name)}">`;
  }

  if (item.kind === "video") {
    if (mime === "image/gif" || String(item.url).toLowerCase().endsWith(".gif")) {
      return `<img class="${className}" src="${escapeAttribute(item.url)}" alt="${escapeAttribute(item.file_name || item.name)}">`;
    }
    if (mime === "video/x-msvideo" || String(item.url).toLowerCase().endsWith(".avi")) {
      return `
        <div class="video-fallback ${className}">
          <strong>AVI video saved locally</strong>
          <span>Your browser may not preview MJPG AVI inline.</span>
          <a href="${escapeAttribute(item.url)}" target="_blank" rel="noreferrer">Open the saved video</a>
        </div>
      `;
    }
    return `<video class="${className}" controls loop src="${escapeAttribute(item.url)}"></video>`;
  }

  return `<audio class="${className}" controls src="${escapeAttribute(item.url)}"></audio>`;
}

function createHistoryPreview(output) {
  if (output.kind === "image") {
    return `<img src="${escapeAttribute(output.url)}" alt="${escapeAttribute(output.file_name)}">`;
  }

  if (output.kind === "gif") {
    return `<img src="${escapeAttribute(output.url)}" alt="${escapeAttribute(output.file_name)}">`;
  }

  if (output.kind === "video") {
    if (String(output.mime || "").toLowerCase() === "image/gif" || String(output.url).toLowerCase().endsWith(".gif")) {
      return `<img src="${escapeAttribute(output.url)}" alt="${escapeAttribute(output.file_name)}">`;
    }
    if (String(output.mime || "").toLowerCase() === "video/x-msvideo" || String(output.url).toLowerCase().endsWith(".avi")) {
      return `<div class="history-thumb">AVI Video</div>`;
    }
    return `<video muted autoplay loop playsinline src="${escapeAttribute(output.url)}"></video>`;
  }

  if (output.kind === "audio") {
    return `<audio controls src="${escapeAttribute(output.url)}"></audio>`;
  }

  return `<div class="history-thumb">${escapeHtml(formatKind(output.kind))}</div>`;
}

function formatKind(kind) {
  if (kind === "gif") return "GIF";
  return kind.charAt(0).toUpperCase() + kind.slice(1);
}

function formatKinds(kinds) {
  const labels = (kinds || []).map((kind) => formatKind(kind));
  return labels.length ? labels.join(", ") : "No direct outputs";
}

function formatBackend(backend) {
  if (backend === "stable_diffusion_cpp") return "stable-diffusion.cpp realism";
  return "llama.cpp expressive";
}

function formatBackendBadge(backend) {
  if (backend === "stable_diffusion_cpp") return "stable-diffusion.cpp";
  return "llama.cpp";
}

function referenceIntentLabel(intent) {
  return intent === "edit" ? "Edit selected" : "Use as guide";
}

async function fetchJson(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Request failed for ${url}`);
  }
  return response.json();
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function escapeAttribute(value) {
  return escapeHtml(value);
}
