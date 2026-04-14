const state = {
  models: [],
  loras: [],
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
  preparing: false,
  generationStyle: "expressive",
  workflowMode: "basic",
  audioSegments: [],
  preparedHandoff: null,
};

const MAX_RUNTIME_SEED = 4294967295;
const MODE_DEFAULTS = {
  expressive: {
    temperature: "0.6",
    steps: "28",
    cfgScale: "7.5",
    sampler: "euler",
    scheduler: "default",
    referenceStrength: "0.8",
    flowShift: "3.0",
    resolution: "square512",
    videoResolution: "square512",
    videoDuration: "5",
    videoFps: "8",
    audioDuration: "10",
    lowVram: false,
  },
  realism: {
    temperature: "0.6",
    steps: "24",
    cfgScale: "6.0",
    sampler: "euler",
    scheduler: "default",
    referenceStrength: "0.8",
    flowShift: "3.0",
    resolution: "square512",
    videoResolution: "square256",
    videoDuration: "2",
    videoFps: "8",
    audioDuration: "10",
    lowVram: true,
  },
};

const elements = {
  promptInput: document.getElementById("promptInput"),
  negativePromptInput: document.getElementById("negativePromptInput"),
  negativePromptBlock: document.getElementById("negativePromptBlock"),
  audioLiteralPromptBlock: document.getElementById("audioLiteralPromptBlock"),
  audioLiteralPromptTitle: document.getElementById("audioLiteralPromptTitle"),
  audioLiteralPromptInput: document.getElementById("audioLiteralPromptInput"),
  manualFocusCuesBlock: document.getElementById("manualFocusCuesBlock"),
  manualFocusCuesInput: document.getElementById("manualFocusCuesInput"),
  manualAssumptionsBlock: document.getElementById("manualAssumptionsBlock"),
  manualAssumptionsInput: document.getElementById("manualAssumptionsInput"),
  audioSegmentsBlock: document.getElementById("audioSegmentsBlock"),
  audioSegmentsTitle: document.getElementById("audioSegmentsTitle"),
  audioSegmentsHelp: document.getElementById("audioSegmentsHelp"),
  audioSegmentsList: document.getElementById("audioSegmentsList"),
  addAudioSegmentButton: document.getElementById("addAudioSegmentButton"),
  prepareKindInput: document.getElementById("prepareKindInput"),
  prepareRequestButton: document.getElementById("prepareRequestButton"),
  clearPreparedButton: document.getElementById("clearPreparedButton"),
  preparedEmpty: document.getElementById("preparedEmpty"),
  preparedPanel: document.getElementById("preparedPanel"),
  preparedMetaChips: document.getElementById("preparedMetaChips"),
  preparedNote: document.getElementById("preparedNote"),
  preparedPromptTitle: document.getElementById("preparedPromptTitle"),
  preparedPromptInput: document.getElementById("preparedPromptInput"),
  preparedSpokenBlock: document.getElementById("preparedSpokenBlock"),
  preparedSpokenInput: document.getElementById("preparedSpokenInput"),
  preparedNegativeBlock: document.getElementById("preparedNegativeBlock"),
  preparedNegativeInput: document.getElementById("preparedNegativeInput"),
  preparedEstimate: document.getElementById("preparedEstimate"),
  preparedFocusTags: document.getElementById("preparedFocusTags"),
  preparedAssumptions: document.getElementById("preparedAssumptions"),
  styleButtons: [...document.querySelectorAll("[data-style]")],
  workflowButtons: [...document.querySelectorAll("[data-workflow]")],
  styleSummary: document.getElementById("styleSummary"),
  workflowSummary: document.getElementById("workflowSummary"),
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
  samplerCard: document.getElementById("samplerCard"),
  samplerInput: document.getElementById("samplerInput"),
  samplerCopy: document.getElementById("samplerCopy"),
  schedulerCard: document.getElementById("schedulerCard"),
  schedulerInput: document.getElementById("schedulerInput"),
  schedulerCopy: document.getElementById("schedulerCopy"),
  loraCard: document.getElementById("loraCard"),
  loraInput: document.getElementById("loraInput"),
  loraCopy: document.getElementById("loraCopy"),
  loraWeightCard: document.getElementById("loraWeightCard"),
  loraWeightInput: document.getElementById("loraWeightInput"),
  loraWeightValue: document.getElementById("loraWeightValue"),
  loraWeightCopy: document.getElementById("loraWeightCopy"),
  referenceStrengthCard: document.getElementById("referenceStrengthCard"),
  referenceStrengthInput: document.getElementById("referenceStrengthInput"),
  referenceStrengthValue: document.getElementById("referenceStrengthValue"),
  referenceStrengthCopy: document.getElementById("referenceStrengthCopy"),
  flowShiftCard: document.getElementById("flowShiftCard"),
  flowShiftInput: document.getElementById("flowShiftInput"),
  flowShiftValue: document.getElementById("flowShiftValue"),
  flowShiftCopy: document.getElementById("flowShiftCopy"),
  resolutionInput: document.getElementById("resolutionInput"),
  videoResolutionInput: document.getElementById("videoResolutionInput"),
  videoDurationInput: document.getElementById("videoDurationInput"),
  videoDurationCopy: document.getElementById("videoDurationCopy"),
  videoFpsInput: document.getElementById("videoFpsInput"),
  videoFpsCopy: document.getElementById("videoFpsCopy"),
  audioDurationInput: document.getElementById("audioDurationInput"),
  audioDurationCopy: document.getElementById("audioDurationCopy"),
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
  inputAssetList: document.getElementById("inputAssetList"),
  outputAssetList: document.getElementById("outputAssetList"),
  trayPreview: document.getElementById("trayPreview"),
  referenceGuide: document.getElementById("referenceGuide"),
  referenceEdit: document.getElementById("referenceEdit"),
  referenceEnd: document.getElementById("referenceEnd"),
  referenceControl: document.getElementById("referenceControl"),
  referenceVoice: document.getElementById("referenceVoice"),
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
  previewHandoffPanel: document.getElementById("previewHandoffPanel"),
};

const GPU_TELEMETRY_WIDTH = 180;
const GPU_TELEMETRY_HEIGHT = 44;

bindSettingDisplay(elements.temperatureInput, elements.temperatureValue, (value) => Number(value).toFixed(1));
bindSettingDisplay(elements.stepsInput, elements.stepsValue, (value) => `${value}`);
bindSettingDisplay(elements.cfgInput, elements.cfgValue, (value) => Number(value).toFixed(1));
bindSettingDisplay(elements.loraWeightInput, elements.loraWeightValue, (value) => Number(value).toFixed(2));
bindSettingDisplay(elements.referenceStrengthInput, elements.referenceStrengthValue, (value) => Number(value).toFixed(2));
bindSettingDisplay(elements.flowShiftInput, elements.flowShiftValue, (value) => Number(value).toFixed(1));

const trackedSettingInputs = [
  elements.temperatureInput,
  elements.stepsInput,
  elements.cfgInput,
  elements.samplerInput,
  elements.schedulerInput,
  elements.loraInput,
  elements.loraWeightInput,
  elements.referenceStrengthInput,
  elements.flowShiftInput,
  elements.resolutionInput,
  elements.videoResolutionInput,
  elements.videoDurationInput,
  elements.videoFpsInput,
  elements.audioDurationInput,
  elements.lowVramInput,
];

elements.refreshAll.addEventListener("click", () => {
  clearPreparedHandoff();
  refreshEverything();
});
elements.clearReference.addEventListener("click", () => clearReferenceSlots());
elements.referenceVoice.addEventListener("click", () => assignSelectedReference("primary", "guide"));
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
  clearPreparedHandoff();
  renderStyleMode();
  renderPrepareKindOptions();
  renderModelSummary();
  refreshAudioSettingCopy();
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
    clearPreparedHandoff();
    renderStyleMode();
    renderModels();
    renderReferenceIntentControls();
    syncActionState();
  });
});
elements.workflowButtons.forEach((button) => {
  button.addEventListener("click", () => {
    const nextMode = button.dataset.workflow;
    if (!nextMode || nextMode === state.workflowMode) {
      return;
    }

    state.workflowMode = nextMode;
    renderPromptWorkflowMode();
    syncActionState();
  });
});

function handleTrackedSettingMutation() {
  state.lastAutoDefaultsStyle = null;
  refreshVideoSettingCopy();
  clearPreparedHandoff();
  renderAdvancedRealismSettings();
  renderModelSummary();
}

trackedSettingInputs.forEach((input) => {
  input.addEventListener("change", handleTrackedSettingMutation);
  input.addEventListener("input", handleTrackedSettingMutation);
});

elements.prepareRequestButton.addEventListener("click", () => prepareGenerationRequest());
elements.clearPreparedButton.addEventListener("click", () => clearPreparedHandoff());
elements.promptInput.addEventListener("input", () => clearPreparedHandoff());
elements.negativePromptInput.addEventListener("input", () => clearPreparedHandoff());
elements.audioLiteralPromptInput.addEventListener("input", () => clearPreparedHandoff());
elements.manualFocusCuesInput.addEventListener("input", () => clearPreparedHandoff());
elements.manualAssumptionsInput.addEventListener("input", () => clearPreparedHandoff());
elements.promptAssistInput.addEventListener("change", () => clearPreparedHandoff());
elements.prepareKindInput.addEventListener("change", () => clearPreparedHandoff());
elements.addAudioSegmentButton.addEventListener("click", () => {
  const model = getSelectedModel();
  if (!isAdvancedAudioSegmentsEnabled(model)) {
    return;
  }
  seedAudioSegmentsFromBasicField(model);
  state.audioSegments.push(createAudioSegment());
  renderAudioPromptInputs();
  clearPreparedHandoff();
  syncActionState();
});
elements.audioSegmentsList.addEventListener("input", (event) => {
  const target = event.target;
  const index = Number(target.dataset.segmentIndex);
  if (!Number.isInteger(index) || !state.audioSegments[index]) {
    return;
  }

  if (target.matches(".audio-segment-label-input")) {
    state.audioSegments[index].label = target.value;
  } else if (target.matches(".audio-segment-literal-input")) {
    state.audioSegments[index].literal = target.value;
  } else {
    return;
  }

  clearPreparedHandoff();
  refreshAudioSettingCopy();
  syncActionState();
});
elements.audioSegmentsList.addEventListener("change", (event) => {
  const target = event.target;
  const index = Number(target.dataset.segmentIndex);
  if (!Number.isInteger(index) || !state.audioSegments[index]) {
    return;
  }

  if (!target.matches(".audio-segment-timing-input")) {
    return;
  }

  state.audioSegments[index].same_time_as_previous = Boolean(target.checked);
  renderAudioPromptInputs();
  clearPreparedHandoff();
  refreshAudioSettingCopy();
  syncActionState();
});
elements.audioSegmentsList.addEventListener("click", (event) => {
  const target = event.target;
  if (!target.matches(".audio-segment-remove")) {
    return;
  }

  const index = Number(target.dataset.segmentIndex);
  if (!Number.isInteger(index)) {
    return;
  }

  state.audioSegments.splice(index, 1);
  renderAudioPromptInputs();
  clearPreparedHandoff();
  refreshAudioSettingCopy();
  syncActionState();
});

connectSocket();
applyModeDefaults(state.generationStyle);
renderStyleMode();
renderPromptWorkflowMode();
renderPrepareKindOptions();
renderPreparedHandoff();
refreshEverything();
startGpuTelemetryPolling();

async function refreshEverything() {
  await Promise.all([
    loadRuntimeStatus(),
    loadHardwareProfile(),
    loadModels(),
    loadLoras(),
    loadAssets(),
    loadOutputs(),
    loadGpuTelemetry(),
  ]);
}

async function loadRuntimeStatus() {
  try {
    state.runtimeStatus = await fetchJson("/api/runtime");
  } catch {
    state.runtimeStatus = null;
  }
  renderStyleMode();
}

async function loadHardwareProfile() {
  try {
    state.hardwareProfile = await fetchJson("/api/hardware");
  } catch {
    state.hardwareProfile = null;
  }

  renderModelSummary();
}

async function loadGpuTelemetry() {
  try {
    state.gpuTelemetry = await fetchJson("/api/telemetry/gpu");
  } catch {
    state.gpuTelemetry = {
      supported: false,
      label: "ECG Window",
      note: "ECG Window is temporarily unavailable.",
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
  elements.samplerInput.value = preset.sampler;
  elements.schedulerInput.value = preset.scheduler;
  elements.referenceStrengthInput.value = preset.referenceStrength;
  elements.flowShiftInput.value = preset.flowShift;
  elements.resolutionInput.value = preset.resolution;
  elements.videoResolutionInput.value = preset.videoResolution;
  elements.videoDurationInput.value = preset.videoDuration;
  elements.videoFpsInput.value = preset.videoFps;
  elements.audioDurationInput.value = preset.audioDuration;
  elements.lowVramInput.checked = Boolean(preset.lowVram);
  refreshSettingDisplays();
}

function refreshSettingDisplays() {
  [
    elements.temperatureInput,
    elements.stepsInput,
    elements.cfgInput,
    elements.referenceStrengthInput,
    elements.flowShiftInput,
  ].forEach((input) => {
    input.dispatchEvent(new Event("input"));
  });
  refreshVideoSettingCopy();
  refreshAudioSettingCopy();
  refreshAdvancedRealismSettingCopy();
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
    && elements.samplerInput.value === preset.sampler
    && elements.schedulerInput.value === preset.scheduler
    && elements.referenceStrengthInput.value === preset.referenceStrength
    && elements.flowShiftInput.value === preset.flowShift
    && elements.resolutionInput.value === preset.resolution
    && elements.videoResolutionInput.value === preset.videoResolution
    && elements.videoDurationInput.value === preset.videoDuration
    && elements.videoFpsInput.value === preset.videoFps
    && elements.audioDurationInput.value === preset.audioDuration
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

function refreshAudioSettingCopy() {
  const selectedModel = getSelectedModel();
  const seconds = Math.max(1, Number(elements.audioDurationInput.value || 0));
  const segmentCount = getNormalizedAudioSegments().length;
  const isStableAudio = selectedModel?.backend === "audio_runtime" && !selectedModel?.supports_voice_output;
  const isSpeechAudio = selectedModel?.backend === "audio_runtime" && selectedModel?.supports_voice_output;

  if (isStableAudio) {
    if (isAdvancedAudioSegmentsEnabled(selectedModel) && segmentCount > 1) {
      elements.audioDurationCopy.textContent = `Target clip length for realism soundscape audio. ${seconds}s is applied per box in advanced mode, with ${segmentCount} boxes currently queued.`;
      return;
    }
    elements.audioDurationCopy.textContent = `Target clip length for realism soundscape audio. ${seconds}s will be handed to the Stable Audio runtime.`;
    return;
  }

  if (isSpeechAudio) {
    if (isAdvancedAudioSegmentsEnabled(selectedModel) && segmentCount > 1) {
      elements.audioDurationCopy.textContent = `Speech-style audio models mainly care about prompt length. In advanced mode, ${segmentCount} script boxes become separate speech segments.`;
      return;
    }
    elements.audioDurationCopy.textContent = "Speech-style audio models mainly care about prompt length and optional voice reference. This duration control is mostly for soundscape/SFX audio.";
    return;
  }

  if (state.generationStyle === "expressive") {
    elements.audioDurationCopy.textContent = "Expressive audio mostly follows the local planner path. This duration control is mainly for realism soundscape audio.";
    return;
  }

  elements.audioDurationCopy.textContent = `Used for Generate Audio. ${seconds}s is the target clip length for realism soundscape audio. Speech models mostly ignore it.`;
}

function refreshAdvancedRealismSettingCopy() {
  const model = getSelectedModel();
  const samplerLabel = elements.samplerInput.options[elements.samplerInput.selectedIndex]?.text || "Euler";
  const samplerValue = elements.samplerInput.value;
  const schedulerValue = elements.schedulerInput.value;
  const schedulerLabel = elements.schedulerInput.options[elements.schedulerInput.selectedIndex]?.text || "Auto / Runtime Default";
  const referenceStrength = Number(elements.referenceStrengthInput.value || 0);
  const flowShift = Number(elements.flowShiftInput.value || 0);
  const isEditIntent = state.referenceIntent === "edit";
  const familyLabel = model?.family || "flow-based";
  const selectedLora = state.loras.find((entry) => entry.id === elements.loraInput.value) || null;
  const hasStillReference = Boolean(state.primaryReference && state.primaryReference.kind === "image");

  elements.samplerCopy.textContent = describeSamplerSetting(samplerValue, samplerLabel);
  elements.schedulerCopy.textContent = describeSchedulerSetting(schedulerValue, schedulerLabel);
  elements.referenceStrengthCopy.textContent = describeReferenceStrengthSetting(
    referenceStrength,
    isEditIntent,
    hasStillReference
  );
  elements.flowShiftCopy.textContent = describeFlowShiftSetting(flowShift, familyLabel);

  if (supportsLoraControl(model)) {
    elements.loraCopy.textContent = describeLoraSelection(model, selectedLora);
    elements.loraWeightCopy.textContent = describeLoraWeightSetting(
      Number(elements.loraWeightInput.value || 1),
      selectedLora
    );
  }
}

function describeSamplerSetting(value, label) {
  const details = {
    euler: "Balanced all-rounder. This is the safest first choice and a good baseline for comparisons.",
    euler_a: "Adds extra randomness and texture. Useful when you want rougher, more chaotic exploration, but it can drift more.",
    heun: "A smoother, more deliberate denoise path. Often worth trying if Euler feels too rough or unstable.",
    dpm2: "A cleaner, more structured sampler than the basic baseline. Good when you want a slightly tidier result.",
    "dpm++2s_a": "Pushes detail harder with a bit more adventurous behaviour. Good for experimentation once the base setup is working.",
    "dpm++2m": "A popular modern sampler for cleaner detail and consistency. Often a good second test after Euler.",
    "dpm++2mv2": "A refined DPM++ variant. Try it when you want cleaner detail without going fully experimental.",
    ipndm: "A lighter, faster-feeling sampler. Handy for quick tests, but it may feel less exact than the safer defaults.",
    ipndm_v: "A variant of IPNDM. Worth testing only if you are already comparing sampler behaviour on purpose.",
    lcm: "Built for fast workflows and lower step counts. Best when the model or LoRA expects LCM-style behaviour.",
    ddim_trailing: "A more traditional denoise feel. Can produce softer, gentler results than the sharper samplers.",
    tcd: "Speed-oriented experimental sampler. Best treated as a deliberate test option rather than a default.",
    res_multistep: "An advanced sampler for deliberate experimentation. Not the best first choice for a new setup.",
    res_2s: "A more experimental sampler variant. Useful for testing, but not usually the best beginner baseline.",
  };

  return `${label} is active. ${details[value] || "This sampler changes how the model walks from noise to the final image. If you are unsure, go back to Euler."}`;
}

function describeSchedulerSetting(value, label) {
  const details = {
    default: "Auto lets the runtime keep its preferred schedule for the selected model family. This is the safest place to start.",
    discrete: "A straightforward traditional schedule. Good as a simple comparison point if Auto is not giving you what you want.",
    karras: "Puts more emphasis on the later denoise stages. Often used when people want cleaner, crisper results.",
    exponential: "Uses a steeper curve across the run. It can feel punchier, but it is more of an experiment than a default.",
    ays: "A schedule aimed at doing more with fewer useful steps. Best for intentional speed-vs-quality testing.",
    gits: "An experimental schedule. Good for side-by-side tests, not usually the first thing to change.",
    sgm_uniform: "Spreads work more evenly across the run. Can be a useful neutral comparison schedule on some families.",
    simple: "A very plain schedule. Best used for debugging or controlled comparisons rather than as a quality preset.",
    smoothstep: "A gentler transition schedule. Useful if other schedules feel too harsh or abrupt.",
    kl_optimal: "An advanced schedule that tries to place denoise effort more efficiently. Worth testing only after the basics feel stable.",
    lcm: "Pairs with LCM-style fast workflows. Most useful when the selected model or LoRA is built for that path.",
    bong_tangent: "A highly experimental schedule. Treat it as a curiosity test rather than a safe everyday option.",
  };

  return `${label} is active. ${details[value] || "This scheduler changes how denoise effort is distributed across the run. Auto is still the safest baseline."}`;
}

function describeReferenceStrengthSetting(value, isEditIntent, hasStillReference) {
  if (!hasStillReference) {
    return "No still-image guide or edit source is assigned right now. This only matters when you use a reference image.";
  }

  const band = value <= 0.35
    ? "very gentle"
    : value <= 0.7
      ? "balanced"
      : "strong";

  if (isEditIntent) {
    if (band === "very gentle") {
      return `Edit mode is active at ${value.toFixed(2)}. This is a very gentle edit setting, so the result should usually stay closer to the source image.`;
    }
    if (band === "balanced") {
      return `Edit mode is active at ${value.toFixed(2)}. This is a balanced edit setting, keeping recognisable structure while still allowing a noticeable rewrite.`;
    }
    return `Edit mode is active at ${value.toFixed(2)}. This is a strong edit setting, so the model is allowed to rewrite the source image more aggressively.`;
  }

  if (band === "very gentle") {
    return `Guide mode is active at ${value.toFixed(2)}. This keeps the reference as a soft hint while still giving the model plenty of freedom.`;
  }
  if (band === "balanced") {
    return `Guide mode is active at ${value.toFixed(2)}. This is a balanced steer, giving the reference a visible say without completely taking over the result.`;
  }
  return `Guide mode is active at ${value.toFixed(2)}. This is a strong steer, so the model should lean much more heavily on the reference image.`;
}

function describeFlowShiftSetting(value, familyLabel) {
  if (value <= 1.5) {
    return `${familyLabel} is using a low flow shift of ${value.toFixed(1)}. That is a conservative setting and usually the safest place to stay if you are troubleshooting.`;
  }
  if (value <= 4.0) {
    return `${familyLabel} is using a flow shift of ${value.toFixed(1)}. This sits close to the normal working range for flow-based families.`;
  }
  return `${familyLabel} is using a high flow shift of ${value.toFixed(1)}. This is an experimental setting and can noticeably change motion or detail behaviour.`;
}

function describeLoraSelection(model, selectedLora) {
  const familyLabel = model?.family || "selected model";
  const familyKey = modelLoraFamilyKey(model) || "family";
  const compatibleLoras = getCompatibleLoras(model);

  if (!selectedLora) {
    return compatibleLoras.length
      ? `${compatibleLoras.length} compatible LoRA${compatibleLoras.length === 1 ? "" : "s"} found for ${familyLabel}. Choose one if you want to bolt a more specific style or concept on top of the base model.`
      : `No compatible LoRAs detected for ${familyLabel}. Put matching files in models/loras/${familyKey}/.`;
  }

  return `${selectedLora.name} is active. Think of it as a small style or concept add-on sitting on top of the ${familyLabel} base model.`;
}

function describeLoraWeightSetting(weight, selectedLora) {
  if (!selectedLora) {
    return "Choose a LoRA first. Lower weights keep it subtle, higher weights push the style harder.";
  }

  if (weight <= 0.35) {
    return `${weight.toFixed(2)} is a very light touch. The LoRA should act more like a hint than a takeover.`;
  }
  if (weight <= 0.75) {
    return `${weight.toFixed(2)} is a gentle LoRA setting. Good when you want the base model to stay in charge.`;
  }
  if (weight <= 1.15) {
    return `${weight.toFixed(2)} is a balanced LoRA setting. This is the best neutral starting point for most tests.`;
  }
  if (weight <= 1.5) {
    return `${weight.toFixed(2)} is a strong LoRA setting. Useful when the LoRA effect feels too weak, but it can start to overpower the base model.`;
  }
  return `${weight.toFixed(2)} is a heavy LoRA setting. Treat this as experimental, because it can distort the base model if the match is poor.`;
}

function createAudioSegment(seed = {}) {
  return {
    label: seed.label || "",
    literal: seed.literal || "",
    same_time_as_previous: Boolean(seed.same_time_as_previous),
  };
}

function getNormalizedAudioSegments() {
  return state.audioSegments
    .map((segment) => ({
      label: String(segment.label || "").trim(),
      literal: String(segment.literal || "").trim(),
      same_time_as_previous: Boolean(segment.same_time_as_previous),
    }))
    .filter((segment) => segment.literal)
    .map((segment) => ({
      label: segment.label || null,
      literal: segment.literal,
      same_time_as_previous: segment.same_time_as_previous,
    }));
}

function isAdvancedAudioSegmentsEnabled(model = getSelectedModel()) {
  return Boolean(
    model
    && model.backend === "audio_runtime"
    && state.workflowMode === "advanced"
  );
}

function seedAudioSegmentsFromBasicField(model = getSelectedModel()) {
  if (!isAdvancedAudioSegmentsEnabled(model) || state.audioSegments.length) {
    return;
  }

  const literal = elements.audioLiteralPromptInput.value.trim();
  state.audioSegments = [createAudioSegment({ literal })];
}

function renderAudioPromptInputs() {
  const selectedModel = getSelectedModel();
  const isDedicatedAudioModel = selectedModel && selectedModel.backend === "audio_runtime";
  const isSpeechAudio = isDedicatedAudioModel && selectedModel.supports_voice_output;
  const advancedAudio = isAdvancedAudioSegmentsEnabled(selectedModel);

  if (advancedAudio) {
    seedAudioSegmentsFromBasicField(selectedModel);
  }

  elements.audioLiteralPromptBlock.classList.toggle("hidden", !isDedicatedAudioModel || advancedAudio);
  elements.audioSegmentsBlock.classList.toggle("hidden", !advancedAudio);

  if (!isDedicatedAudioModel) {
    elements.audioLiteralPromptTitle.textContent = "Words / Sounds";
    elements.audioLiteralPromptInput.placeholder = "Optional verbatim words or literal sound cues to preserve exactly.";
    elements.audioSegmentsList.innerHTML = "";
    return;
  }

  elements.audioLiteralPromptTitle.textContent = isSpeechAudio ? "Words / Script" : "Words / Sounds";
  elements.audioLiteralPromptInput.placeholder = isSpeechAudio
    ? "Optional exact words to be spoken aloud. Leave the main Prompt field for delivery and style notes."
    : "Optional literal sound cues to preserve exactly, like dripping water, distant thunder, crackling fire.";

  if (!advancedAudio) {
    elements.audioSegmentsList.innerHTML = "";
    return;
  }

  elements.audioSegmentsTitle.textContent = isSpeechAudio ? "Script Sequence" : "Sound Sequence";
  elements.audioSegmentsHelp.textContent = isSpeechAudio
    ? "Each box becomes its own spoken segment. Use the main Prompt field for delivery direction, these boxes for exact lines, and reuse the same Voice Name when you want the same character voice to stay consistent."
    : "Each box becomes its own sound event. Use the main Prompt field for the overall scene, these boxes for literal sound cues, and reuse the same Layer Name when you want the same sound identity to stay consistent.";

  if (!state.audioSegments.length) {
    elements.audioSegmentsList.innerHTML = `
      <div class="selection-summary audio-segments-empty">
        No advanced audio boxes yet. Add one to start building a sequence.
      </div>
    `;
    return;
  }

  elements.audioSegmentsList.innerHTML = state.audioSegments
    .map((segment, index) => {
      const segmentName = isSpeechAudio ? `Voice ${index + 1}` : `Sound ${index + 1}`;
      const roleLabel = isSpeechAudio ? "Voice Name / Character Note" : "Layer Name / Sound Note";
      const rolePlaceholder = isSpeechAudio
        ? "Same name = same voice, like Narrator, Caller, Child, Robot"
        : "Same name = same sound identity, like Rain Bed, Footsteps, Crowd, Thunder";
      const literalLabel = isSpeechAudio ? "Words / Script" : "Words / Sounds";
      const literalPlaceholder = isSpeechAudio
        ? "Type the exact line to be spoken in this segment."
        : "Type the exact sound cues or sound description for this segment.";
      const timingMarkup = index === 0
        ? `<div class="audio-segment-timing-note">This box starts first.</div>`
        : `
          <label class="setting-toggle audio-segment-timing-toggle">
            <input
              class="audio-segment-timing-input"
              data-segment-index="${index}"
              type="checkbox"
              ${segment.same_time_as_previous ? "checked" : ""}
            >
            <span>Occurring at the same time as last box</span>
          </label>
          <div class="audio-segment-timing-note">${segment.same_time_as_previous ? "This box will start alongside the last box." : "This box will start after the last box ends."}</div>
        `;

      return `
        <section class="audio-segment-card">
          <div class="audio-segment-header">
            <strong>${escapeHtml(segmentName)}</strong>
            <button
              class="audio-segment-remove"
              data-segment-index="${index}"
              type="button"
              aria-label="Remove ${escapeHtml(segmentName)}"
              title="Remove this box"
            >×</button>
          </div>
          <label class="field-block compact-segment-field">
            <span class="field-title">${escapeHtml(roleLabel)}</span>
            <input
              class="audio-segment-label-input"
              data-segment-index="${index}"
              type="text"
              value="${escapeHtml(segment.label || "")}"
              placeholder="${escapeHtml(rolePlaceholder)}"
            >
          </label>
          <label class="field-block compact-segment-field">
            <span class="field-title">${escapeHtml(literalLabel)}</span>
            <textarea
              class="audio-segment-literal-input"
              data-segment-index="${index}"
              rows="3"
              placeholder="${escapeHtml(literalPlaceholder)}"
            >${escapeHtml(segment.literal || "")}</textarea>
          </label>
          ${timingMarkup}
        </section>
      `;
    })
    .join("");
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

async function loadLoras() {
  try {
    state.loras = await fetchJson("/api/loras");
  } catch {
    state.loras = [];
  }

  renderAdvancedRealismSettings();
  renderModelSummary();
  renderPreparedHandoff();
}

async function loadAssets() {
  try {
    state.assets = await fetchJson("/api/assets");
  } catch {
    state.assets = [];
  }
  reconcileAssignedAssets();
  renderAssets();
}

async function loadOutputs() {
  try {
    state.outputs = await fetchJson("/api/outputs");
  } catch {
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
  const selectedModel = getSelectedModel();
  elements.styleSummary.textContent = realism
    ? (selectedModel && selectedModel.backend === "audio_runtime"
        ? (selectedModel.supports_voice_output
            ? "Realism speech uses a separate specialist audio-runtime lane. OuteTTS-style models focus on spoken voice output rather than image/video diffusion."
            : "Realism soundscape audio uses a separate specialist audio-runtime lane. Stable Audio style packages focus on ambience, effects, and texture-driven clips rather than speech.")
        : "Realism uses local specialist backends. Today that means stable-diffusion.cpp for image, GIF, and supported video jobs, with realism-audio families detected separately as they are wired.")
    : "Expressive uses the bundled llama.cpp planner plus Chatty-art's local renderer for fast image, GIF, and audio output.";
  renderRuntimeBadges();
  refreshAudioSettingCopy();

  elements.negativePromptBlock.classList.toggle("hidden", !realism);
  renderAudioPromptInputs();
  renderManualPromptAssistInputs();
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
  renderAdvancedRealismSettings();
  renderReferenceIntentControls();
}

function renderPromptWorkflowMode() {
  const advanced = state.workflowMode === "advanced";

  elements.workflowButtons.forEach((button) => {
    button.classList.toggle("active", button.dataset.workflow === state.workflowMode);
  });

  elements.workflowSummary.textContent = advanced
    ? "Advanced keeps Preview Handoff visible and adds deeper controls for power users."
    : "Basic keeps the workflow simple while still letting you review the handoff before you generate.";

  elements.previewHandoffPanel.classList.remove("hidden");
  renderAudioPromptInputs();
  renderManualPromptAssistInputs();
  renderAdvancedRealismSettings();
}

function supportsManualPromptAssistInputs(model = getSelectedModel()) {
  return Boolean(
    state.generationStyle === "realism"
    && state.workflowMode === "advanced"
    && model
    && model.backend === "stable_diffusion_cpp"
    && kindSupported(model, "image")
  );
}

function renderManualPromptAssistInputs() {
  const show = supportsManualPromptAssistInputs();
  elements.manualFocusCuesBlock.classList.toggle("hidden", !show);
  elements.manualAssumptionsBlock.classList.toggle("hidden", !show);
}

function supportsAdvancedRealismSettings(model = getSelectedModel()) {
  return Boolean(
    state.generationStyle === "realism"
    && state.workflowMode === "advanced"
    && model
    && model.backend === "stable_diffusion_cpp"
  );
}

function supportsReferenceStrengthControl(model = getSelectedModel()) {
  return Boolean(
    supportsAdvancedRealismSettings(model)
    && model.supports_reference_strength
  );
}

function supportsFlowShiftControl(model = getSelectedModel()) {
  if (!supportsAdvancedRealismSettings(model)) {
    return false;
  }

  const family = String(model.family || "").toLowerCase();
  return family.includes("wan") || family.includes("qwen");
}

function modelLoraFamilyKey(model = getSelectedModel()) {
  if (!model || model.backend !== "stable_diffusion_cpp") {
    return null;
  }

  const family = String(model.family || "").toLowerCase();
  if (family.includes("flux")) return "flux";
  if (family.includes("sd3")) return "sd3";
  if (family.includes("wan")) return "wan";
  if (family.includes("qwen")) return "qwen";
  if (
    family.includes("stable diffusion")
    || family.includes("self-contained diffusion")
    || family.includes("diffusion gguf")
  ) {
    return "sd";
  }

  return null;
}

function getCompatibleLoras(model = getSelectedModel()) {
  const familyKey = modelLoraFamilyKey(model);
  if (!familyKey) {
    return [];
  }

  return state.loras.filter((lora) => lora.runtime_supported && lora.family_key === familyKey);
}

function supportsLoraControl(model = getSelectedModel()) {
  return Boolean(
    supportsAdvancedRealismSettings(model)
    && getCompatibleLoras(model).length
  );
}

function renderAdvancedRealismSettings() {
  const model = getSelectedModel();
  const showAdvancedRealism = supportsAdvancedRealismSettings(model);
  const showLora = supportsLoraControl(model);
  const showReferenceStrength = supportsReferenceStrengthControl(model);
  const showFlowShift = supportsFlowShiftControl(model);

  elements.samplerCard.classList.toggle("hidden", !showAdvancedRealism);
  elements.schedulerCard.classList.toggle("hidden", !showAdvancedRealism);
  elements.loraCard.classList.toggle("hidden", !showLora);
  elements.loraWeightCard.classList.toggle("hidden", !showLora);
  elements.referenceStrengthCard.classList.toggle("hidden", !showReferenceStrength);
  elements.flowShiftCard.classList.toggle("hidden", !showFlowShift);

  if (showLora) {
    const compatibleLoras = getCompatibleLoras(model);
    const currentSelection = elements.loraInput.value;
    const options = [
      `<option value="">No LoRA</option>`,
      ...compatibleLoras.map((lora) => `<option value="${escapeHtml(lora.id)}">${escapeHtml(lora.name)} | ${escapeHtml(lora.family)}</option>`),
    ];
    elements.loraInput.innerHTML = options.join("");
    if (compatibleLoras.some((lora) => lora.id === currentSelection)) {
      elements.loraInput.value = currentSelection;
    } else {
      elements.loraInput.value = "";
    }

    const familyLabel = model.family || "selected";
    elements.loraCopy.textContent = compatibleLoras.length
      ? `${compatibleLoras.length} compatible LoRA${compatibleLoras.length === 1 ? "" : "s"} found for ${familyLabel}. Put more in models/loras/${compatibleLoras[0].family_key}/.`
      : `No compatible LoRAs detected for ${familyLabel}. Put them in models/loras/${modelLoraFamilyKey(model) || "family"}/.`;
    const weight = Number(elements.loraWeightInput.value || 1);
    elements.loraWeightCopy.textContent = elements.loraInput.value
      ? `${weight.toFixed(2)} will be used as the LoRA strength for this run. 1.00 is the safest neutral starting point.`
      : "Choose a LoRA first. Lower weights keep it subtle, higher weights push the style harder.";
  } else {
    elements.loraInput.innerHTML = `<option value="">No LoRA</option>`;
    elements.loraInput.value = "";
  }

  refreshAdvancedRealismSettingCopy();
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

  const label = String(telemetry.label || "ECG Window").trim() || "ECG Window";
  const note = String(telemetry.note || "ECG-style view of the busiest local GPU engine.").trim();
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
    elements.modelSelect.innerHTML = `<option value="">No local models found in models/</option>`;
    elements.modelSelect.disabled = true;
    renderStyleMode();
    renderModelNotice("Drop one or more GGUF models or supported local model packages into models/ and press Refresh Files.");
    renderPrepareKindOptions();
    renderPreparedHandoff();
    renderReferenceIntentControls();
    syncActionState();
    return;
  }

  if (!visibleModels.length) {
    const label = state.generationStyle === "realism" ? "No realism models found" : "No expressive models found";
    elements.modelSelect.innerHTML = `<option value="">${label}</option>`;
    elements.modelSelect.disabled = true;
    renderStyleMode();
    renderModelNotice(
      state.generationStyle === "realism"
        ? "Realism mode needs diffusion-style GGUFs or supported local model packages, plus any companion weights they require in models/. Switch to Expressive to use regular llama.cpp models."
        : "Expressive mode uses regular llama.cpp-compatible models. Switch to Realism for diffusion/video GGUFs."
    );
    renderPrepareKindOptions();
    renderPreparedHandoff();
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
    renderStyleMode();
    renderModelSummary(hiddenModeCount);
    renderPrepareKindOptions();
    renderPreparedHandoff();
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
        .map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildDropdownLabel(model))}</option>`)
        .join("")}</optgroup>`
    : "";

  elements.modelSelect.innerHTML = `<optgroup label="Ready to run">${supportedOptions}</optgroup>${unsupportedOptions}`;

  if (selected && visibleModels.some((model) => model.id === selected)) {
    elements.modelSelect.value = selected;
  } else {
    elements.modelSelect.value = supportedModels[0].id;
  }

  normalizeAssignedReferencesForCurrentModel();
  renderStyleMode();
  renderModelSummary(hiddenModeCount);
  renderPrepareKindOptions();
  renderPreparedHandoff();
  renderReferenceIntentControls();
  syncActionState();
}

function renderPrepareKindOptions() {
  const model = getSelectedModel();
  const fallbackKinds = state.generationStyle === "realism"
    ? ["image", "gif", "video"]
    : ["image", "gif", "audio"];
  const supportedKinds = model
    ? ((model.supported_kinds || []).length ? model.supported_kinds : fallbackKinds)
    : fallbackKinds;
  const current = elements.prepareKindInput.value;

  elements.prepareKindInput.innerHTML = supportedKinds
    .map((kind) => `<option value="${escapeHtml(kind)}">${escapeHtml(formatKind(kind))}</option>`)
    .join("");

  if (supportedKinds.includes(current)) {
    elements.prepareKindInput.value = current;
  } else {
    elements.prepareKindInput.value = supportedKinds[0] || "image";
  }
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
  if ((model.supported_kinds || []).includes("audio")) {
    badges.push(
      createModelBadge(
        model.supports_voice_output ? "Speech / Voice" : "Soundscape / SFX",
        "reference"
      )
    );
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
  if (supportsLoraControl(model) && elements.loraInput.value) {
    const selectedLora = state.loras.find((entry) => entry.id === elements.loraInput.value);
    if (selectedLora) {
      badges.push(
        createModelBadge(
          `LoRA: ${selectedLora.name} @ ${Number(elements.loraWeightInput.value || 1).toFixed(2)}`,
          "reference"
        )
      );
    }
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
  refreshAudioSettingCopy();
}

function buildRecommendedLimitsMarkup(model) {
  const hardware = state.hardwareProfile;
  if (!hardware || !model) {
    return "";
  }

  const rows = (model.supported_kinds || [])
    .map((kind) => {
      const recommendation = buildKindRecommendation(model, kind, hardware);
      if (!recommendation) {
        return null;
      }

      return {
        ...recommendation,
        current: assessCurrentKindPressure(model, kind, hardware),
      };
    })
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
          <div class="recommended-limit-row current-${escapeHtml(row.current.tone)}">
            <strong>${escapeHtml(row.kind)}</strong>
            <span><em>Safe:</em> ${escapeHtml(row.safe)}</span>
            <span><em>Stretch:</em> ${escapeHtml(row.stretch)}</span>
            <span><em>Risky:</em> ${escapeHtml(row.risky)}</span>
            <span class="recommended-current ${escapeHtml(`current-${row.current.tone}`)}"><em>Current:</em> ${escapeHtml(row.current.summary)} -> ${escapeHtml(row.current.label)}</span>
            <span class="recommended-current-note">${escapeHtml(row.current.note)}</span>
          </div>
        `).join("")}
      </div>
      <div class="recommended-limits-note">${escapeHtml(hardware.note || "Recommendations are heuristics based on the current machine and selected model.")}</div>
    </div>
  `;
}

function buildKindRecommendation(model, kind, hardware) {
  const family = String(model.family || "").toLowerCase();
  const dedicated = Number(hardware.dedicated_vram_gb || 0);
  const lowVram = Boolean(elements.lowVramInput.checked);
  const sizeHint = parseModelSizeHint(model.name);
  const isExpressive = model.backend === "llama_cpp";
  const isAudioRuntime = model.backend === "audio_runtime";
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
    if (isAudioRuntime) {
      if (model.supports_voice_output) {
        return {
          kind: "Audio",
          safe: "Short to medium speech lines with the default voice",
          stretch: "Longer narration or optional voice-reference cloning",
          risky: "Very long passages mainly cost CPU/RAM time rather than VRAM, especially on local speech runtimes",
        };
      }
      return {
        kind: "Audio",
        safe: "5s to 10s soundscape clips at the default steps",
        stretch: "20s soundscape clips or higher steps",
        risky: "Long ambience/SFX clips mainly become CPU/RAM heavy rather than GPU-VRAM heavy",
      };
    }
    return {
      kind: "Audio",
      safe: "Audio generation is not GPU-limited in the same way as realism video",
      stretch: "Longer prompts and more steps mainly cost time",
      risky: "Very large expressive models can still be slow to plan",
    };
  }

  return null;
}

function assessCurrentKindPressure(model, kind, hardware) {
  const family = String(model.family || "").toLowerCase();
  const dedicated = Math.max(1, Number(hardware.dedicated_vram_gb || 8));
  const lowVram = state.generationStyle === "realism" && Boolean(elements.lowVramInput.checked);
  const sizeHint = parseModelSizeHint(model.name);
  const isExpressive = model.backend === "llama_cpp";
  const isWan = family.includes("wan");
  const isFlux = family.includes("flux");
  const isAmd = /amd|radeon/i.test(String(hardware.gpu_label || ""));
  const smallWan = isWan && sizeHint <= 20;
  const current = currentSettingsForKind(model, kind);

  if (!current) {
    return {
      tone: "safe",
      label: "No current setting",
      summary: "Unavailable",
      note: "Select a supported output to see a live hardware assessment.",
    };
  }

  if (kind === "audio") {
    if (isExpressive) {
      const promptWords = current.promptWords || 0;
      if (promptWords <= 40) {
        return {
          tone: "safe",
          label: "Comfortable",
          summary: current.summary,
          note: "Expressive audio is mostly planner time, not VRAM pressure, at this prompt length.",
        };
      }
      if (promptWords <= 120) {
        return {
          tone: "stretch",
          label: "Heavy but reasonable",
          summary: current.summary,
          note: "Longer expressive audio prompts mainly make the local planner slower rather than causing GPU OOMs.",
        };
      }
      return {
        tone: "risky",
        label: "Very long prompt",
        summary: current.summary,
        note: "This is more likely to become a slow expressive-planning job than a hard hardware failure.",
      };
    }

    const isSpeechAudio = model.backend === "audio_runtime" && model.supports_voice_output;
    if (isSpeechAudio) {
      const promptWords = current.promptWords || 0;
      const hasVoiceReference = Boolean(state.primaryReference && state.primaryReference.kind === "audio");
      if (promptWords <= 40 && !hasVoiceReference) {
        return {
          tone: "safe",
          label: "Comfortable",
          summary: current.summary,
          note: "This speech request is well inside the easy local range. Prompt length matters more than VRAM here.",
        };
      }
      if (promptWords <= 120) {
        return {
          tone: "stretch",
          label: "Heavy but reasonable",
          summary: current.summary,
          note: hasVoiceReference
            ? "Voice-reference cloning adds extra local runtime work, but this should still be reasonable."
            : "Longer narration pushes runtime and memory, but it should still be manageable.",
        };
      }
      return {
        tone: "risky",
        label: "Very long speech request",
        summary: current.summary,
        note: "Very long narration is more likely to feel slow or RAM-heavy than to trip a GPU OOM.",
      };
    }

    const duration = current.audioDurationSeconds || 0;
    const pressure = duration * Math.max(0.7, Number(elements.stepsInput.value || 24) / 24);
    if (pressure <= 12) {
      return {
        tone: "safe",
        label: "Comfortable",
        summary: current.summary,
        note: "This soundscape/audio length is comfortably inside the local Stable Audio range for this machine.",
      };
    }
    if (pressure <= 24) {
      return {
        tone: "stretch",
        label: "Heavy but reasonable",
        summary: current.summary,
        note: "Longer soundscape clips mainly push CPU and RAM time rather than dedicated VRAM on the current audio runtime.",
      };
    }
    return {
      tone: "risky",
      label: "Long audio job",
      summary: current.summary,
      note: "This is more likely to be a very slow local audio render than a classic GPU OOM, but it is beyond the comfortable range.",
    };
  }

  if (isExpressive) {
    const hintedSize = parseModelSizeHint(model.name);
    const sizeScale = hintedSize >= 9999 ? 1.0 : Math.max(0.6, hintedSize / 80);
    const pressure = current.pixelScale * Math.max(1, current.frameCount / 16) * Math.max(0.7, sizeScale);
    if (pressure <= 2.5) {
      return {
        tone: "safe",
        label: "Comfortable",
        summary: current.summary,
        note: "Expressive mode is more likely to get slower than to hit a hard GPU memory wall.",
      };
    }
    if (pressure <= 5.5) {
      return {
        tone: "stretch",
        label: "Heavy but reasonable",
        summary: current.summary,
        note: "This should still run, but longer clips or higher steps may feel slow.",
      };
    }
    return {
      tone: "risky",
      label: "Very heavy",
      summary: current.summary,
      note: "This is more likely to cost a lot of time than to hard-fail, but it is beyond the comfortable range for local expressive output.",
    };
  }

  if (kind === "image") {
    const familyScale = isWan ? 1.2 : isFlux ? 1.25 : 0.9;
    const pressure = current.pixelScale * familyScale * (lowVram ? 0.88 : 1.0);
    const safeThreshold = dedicated >= 12 ? 2.2 : dedicated >= 8 ? 1.5 : 1.1;
    const stretchThreshold = dedicated >= 12 ? 3.8 : dedicated >= 8 ? 2.6 : 1.7;

    if (pressure <= safeThreshold) {
      return {
        tone: "safe",
        label: "Safe now",
        summary: current.summary,
        note: "This image size sits inside the comfortable range for the selected model on this hardware.",
      };
    }
    if (pressure <= stretchThreshold) {
      return {
        tone: "stretch",
        label: "Stretch",
        summary: current.summary,
        note: lowVram
          ? "This is above the easy range, but Low VRAM mode is giving the runtime a safer decode path."
          : "This is above the easy range. Low VRAM mode or a smaller still size would be safer.",
      };
    }
    return {
      tone: "risky",
      label: "Likely OOM",
      summary: current.summary,
      note: "This image size is large enough that Vulkan decode can fail even when Windows still reports shared GPU memory available.",
    };
  }

  const familyScale = isWan ? (smallWan ? 1.6 : 1.85) : isFlux ? 1.15 : 1.0;
  const pressure =
    current.pixelScale
    * Math.max(1, current.frameCount / 16)
    * familyScale
    * (lowVram ? 0.82 : 1.0);
  const safeThreshold = dedicated >= 12 ? 1.6 : dedicated >= 8 ? 1.0 : 0.75;
  const stretchThreshold = dedicated >= 12 ? 3.0 : dedicated >= 8 ? 2.0 : 1.25;
  const frameStress = current.frameCount > 80;
  const resolutionStress = current.maxDimension >= 768;
  const baseNote = isAmd
    ? "On AMD/Windows, shared GPU memory can be in use and Task Manager can still look roomy while Vulkan fails one large contiguous allocation."
    : "Shared GPU memory can help a little, but Vulkan video jobs still fail when a single large allocation cannot be satisfied.";

  if (pressure <= safeThreshold) {
    return {
      tone: "safe",
      label: "Safe now",
      summary: current.summary,
      note: "This clip sits inside the comfortable range for the selected model on this hardware.",
    };
  }
  if (pressure <= stretchThreshold) {
    return {
      tone: "stretch",
      label: "Stretch",
      summary: current.summary,
      note: lowVram
        ? "Low VRAM mode is helping here, but clip length and resolution are already pushing past the easy range."
        : "This should be treated as a stretch setting. Low VRAM mode and a shorter clip would be safer.",
    };
  }
  return {
    tone: "risky",
    label: "Likely OOM",
    summary: current.summary,
    note: frameStress
      ? `Frame count is the biggest multiplier here. ${baseNote}`
      : resolutionStress
      ? `Resolution is the biggest multiplier here. ${baseNote}`
      : baseNote,
  };
}

function currentSettingsForKind(model, kind) {
  if (kind === "gif" || kind === "video") {
    const resolution = elements.videoResolutionInput;
    const summary = `${selectedOptionLabel(resolution)} | ${elements.videoDurationInput.value}s | ${elements.videoFpsInput.value} FPS (${currentVideoFrameCount()} frames)`;
    const [width, height] = parseDimensionPair(resolution.value);
    return {
      summary,
      width,
      height,
      pixelScale: Math.max(0.25, (width * height) / (512 * 512)),
      maxDimension: Math.max(width, height),
      frameCount: currentVideoFrameCount(),
    };
  }

  if (kind === "audio") {
    const promptWords = Math.max(1, elements.promptInput.value.trim().split(/\s+/).filter(Boolean).length);
    const audioDurationSeconds = Math.max(1, Number(elements.audioDurationInput.value || 0));
    if (model?.backend === "llama_cpp") {
      return {
        summary: `${promptWords} words | ${elements.stepsInput.value} steps | temp ${elements.temperatureInput.value}`,
        width: 1,
        height: 1,
        pixelScale: 1,
        maxDimension: 1,
        frameCount: 1,
        promptWords,
        audioDurationSeconds,
      };
    }
    if (model?.backend === "audio_runtime" && model.supports_voice_output) {
      return {
        summary: `${promptWords} words | ${elements.stepsInput.value} steps${state.primaryReference?.kind === "audio" ? " | voice reference" : ""}`,
        width: 1,
        height: 1,
        pixelScale: 1,
        maxDimension: 1,
        frameCount: 1,
        promptWords,
        audioDurationSeconds,
      };
    }
    return {
      summary: `${audioDurationSeconds}s audio | ${elements.stepsInput.value} steps | CFG ${elements.cfgInput.value}`,
      width: 1,
      height: 1,
      pixelScale: 1,
      maxDimension: 1,
      frameCount: 1,
      promptWords,
      audioDurationSeconds,
    };
  }

  if (kind === "image") {
    const resolution = elements.resolutionInput;
    const summary = `${selectedOptionLabel(resolution)} | ${elements.stepsInput.value} steps`;
    const [width, height] = parseDimensionPair(resolution.value);
    return {
      summary,
      width,
      height,
      pixelScale: Math.max(0.35, (width * height) / (512 * 512)),
      maxDimension: Math.max(width, height),
      frameCount: 1,
    };
  }

  return null;
}

function currentVideoFrameCount() {
  const seconds = Math.max(1, Number(elements.videoDurationInput.value || 0));
  const fps = Math.max(1, Number(elements.videoFpsInput.value || 0));
  return seconds * fps;
}

function selectedOptionLabel(select) {
  return select.options[select.selectedIndex]?.text || select.value;
}

function parseDimensionPair(value) {
  switch (value) {
    case "square256":
      return [256, 256];
    case "square512":
      return [512, 512];
    case "square768":
      return [768, 768];
    case "landscape720":
      return [1280, 720];
    case "portrait768":
      return [768, 1024];
    case "landscape1024":
      return [1024, 768];
    case "poster1024":
      return [1024, 1280];
    default:
      return [512, 512];
  }
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
  const assets = state.assets.filter((asset) => assetMatchesActiveFilter(asset));
  const inputAssets = assets.filter((asset) => asset.source === "input");
  const outputAssets = assets.filter((asset) => asset.source === "output");

  renderAssetSection(elements.inputAssetList, inputAssets, {
    empty: "No matching files found in <code>input/</code>. Put files into <code>input/images</code>, <code>input/video</code>, or <code>input/audio</code>, then press Refresh Files.",
  });
  renderAssetSection(elements.outputAssetList, outputAssets, {
    empty: "No matching files found in <code>outputs/</code> yet. Generated images, GIFs, video, and audio will show up here after a run finishes.",
  });

  renderTrayFilters();
  renderReferenceIntentControls();
  renderAssignedReferences();
  renderTrayPreview();
}

function assetMatchesActiveFilter(asset) {
  if (state.activeFilter === "all") {
    return true;
  }

  if (state.activeFilter === "video") {
    return asset.kind === "video" || asset.kind === "gif";
  }

  return asset.kind === state.activeFilter;
}

function renderAssetSection(container, assets, { empty }) {
  if (!assets.length) {
    container.innerHTML = `<div class="tray-empty">${empty}</div>`;
    return;
  }

  container.innerHTML = assets
    .map((asset) => {
      const active = state.selectedReference?.id === asset.id ? "active" : "";
      return `
        <button class="asset-card ${active}" type="button" data-asset-id="${escapeHtml(asset.id)}">
          <strong>${escapeHtml(asset.name)}</strong>
          <span>${escapeHtml(formatKind(asset.kind))} ${escapeHtml(asset.source)} reference</span>
          <span>${escapeHtml(asset.relative_path)}</span>
        </button>
      `;
    })
    .join("");

  container.querySelectorAll("[data-asset-id]").forEach((button) => {
    button.addEventListener("click", () => {
      const asset = state.assets.find((entry) => entry.id === button.dataset.assetId);
      setSelectedReference(asset || null);
    });
  });
}

function renderTrayFilters() {
  elements.trayFilters.forEach((button) => {
    button.classList.toggle("active", button.dataset.filter === state.activeFilter);
  });
}

function renderTrayPreview() {
  if (!state.selectedReference) {
    elements.trayPreview.innerHTML = `<div class="tray-empty">Choose a file from the tray to use it as a reference or edit source.</div>`;
    return;
  }

  const asset = state.selectedReference;
  const media = createMediaMarkup(asset, "tray-media");
  const assignments = [];
  if (state.primaryReference?.id === asset.id) {
    assignments.push(primaryReferenceAssignmentLabel());
  }
  if (state.endReference?.id === asset.id) {
    assignments.push("End frame");
  }
  if (state.controlReference?.id === asset.id) {
    assignments.push("Control video");
  }
  elements.trayPreview.innerHTML = `
    <strong>${escapeHtml(asset.name)}</strong>
    <span>${escapeHtml(asset.source === "output" ? "Output Folder" : "Input Folder")}</span>
    <span>${escapeHtml(asset.relative_path)}</span>
    <span>${escapeHtml(assignments.length ? `Assigned as: ${assignments.join(" | ")}` : "Not assigned to a slot yet.")}</span>
    ${media}
  `;
}

function isSpeechVoiceReferenceModel(model = getSelectedModel()) {
  return Boolean(
    state.generationStyle === "realism"
    && model
    && model.backend === "audio_runtime"
    && model.supports_voice_output
  );
}

function primaryReferenceSlotLabel(model = getSelectedModel()) {
  return isSpeechVoiceReferenceModel(model) ? "Voice reference" : "Primary input";
}

function primaryReferenceAssignmentLabel(model = getSelectedModel()) {
  return isSpeechVoiceReferenceModel(model)
    ? "Voice reference"
    : `Start image | ${referenceIntentLabel(state.referenceIntent)}`;
}

function primaryReferenceEmptyDetail(model = getSelectedModel()) {
  return isSpeechVoiceReferenceModel(model)
    ? "No voice reference assigned."
    : "No primary guide/edit input assigned.";
}

function primaryReferenceFilledDetail(model = getSelectedModel()) {
  return isSpeechVoiceReferenceModel(model)
    ? "Used to clone the speaker voice for realism speech generation."
    : `${referenceIntentLabel(state.referenceIntent)}`;
}

function renderReferenceIntentControls() {
  const context = getReferenceAssignmentContext();
  const selectedAssetId = state.selectedReference?.id;
  const speechVoiceModel = isSpeechVoiceReferenceModel();

  elements.referenceVoice.classList.toggle(
    "active",
    speechVoiceModel && state.primaryReference?.id === selectedAssetId
  );
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

  elements.referenceVoice.classList.toggle("hidden", !speechVoiceModel);
  elements.referenceGuide.classList.toggle("hidden", speechVoiceModel);
  elements.referenceEdit.classList.toggle("hidden", speechVoiceModel);
  elements.referenceEnd.classList.toggle("hidden", speechVoiceModel);
  elements.referenceControl.classList.toggle("hidden", speechVoiceModel);

  elements.referenceVoice.disabled = !context.voiceEnabled;
  elements.referenceGuide.disabled = !context.guideEnabled;
  elements.referenceEdit.disabled = !context.editEnabled;
  elements.referenceEnd.disabled = !context.endEnabled;
  elements.referenceControl.disabled = !context.controlEnabled;
  elements.referenceModeNote.textContent = context.message;
  refreshAdvancedRealismSettingCopy();
  renderAssignedReferences();
}

function getReferenceAssignmentContext() {
  const model = getSelectedModel();
  const asset = state.selectedReference;

  if (!asset) {
    if (isSpeechVoiceReferenceModel(model)) {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: "Choose an audio file first. Voice Reference assigns a prerecorded clip for speech cloning.",
      };
    }

    return {
      voiceEnabled: false,
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: "Choose a file first. Guide/Edit assign a start image. End Frame assigns the final still. Control Video assigns a motion guide from the tray.",
    };
  }

  if (!model) {
    return {
      voiceEnabled: false,
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: "Choose a model first so Chatty-art can match the selected file to a backend.",
    };
  }

  if (isSpeechVoiceReferenceModel(model)) {
    if (!model.runtime_supported) {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: "This speech model is not ready yet, so voice-reference assignment is disabled.",
      };
    }

    if (!model.supports_audio_reference) {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: "This speech model does not use voice-reference cloning in Chatty-art yet.",
      };
    }

    return {
      voiceEnabled: asset.kind === "audio",
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: asset.kind === "audio"
        ? "Assign this audio file as the voice reference for realism speech generation."
        : "Speech voice cloning uses an audio file from the tray as the voice reference.",
    };
  }

  if (state.generationStyle === "expressive") {
    return {
      voiceEnabled: false,
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
      voiceEnabled: false,
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
      voiceEnabled: false,
      guideEnabled,
      editEnabled: guideEnabled,
      endEnabled,
      controlEnabled: false,
      message: endEnabled
        ? "Assign this still image as the start image, edit source, or end frame depending on the selected realism model."
        : "Assign this still image as a guide or edit source for realism generation.",
    };
  }

  if (asset.kind === "video" || asset.kind === "gif") {
    return {
      voiceEnabled: false,
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
    voiceEnabled: false,
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
      ${item.reference_asset ? `<p><strong>${escapeHtml(item.kind === "audio" && item.spoken_text ? "Voice reference" : "Reference use")}:</strong> ${escapeHtml(item.kind === "audio" && item.spoken_text ? item.reference_asset : `${referenceIntentLabel(item.reference_intent || "guide")} via ${item.reference_asset}`)}</p>` : ""}
      ${item.end_reference_asset ? `<p><strong>End frame:</strong> ${escapeHtml(item.end_reference_asset)}</p>` : ""}
      ${item.control_reference_asset ? `<p><strong>Control video:</strong> ${escapeHtml(item.control_reference_asset)}</p>` : ""}
      ${item.spoken_text ? `<p><strong>Spoken text:</strong> ${escapeHtml(item.spoken_text)}</p>` : ""}
      ${item.compiled_prompt ? `<p><strong>${item.spoken_text ? "Speech direction:" : "Compiled brief:"}</strong> ${escapeHtml(item.compiled_prompt)}</p>` : ""}
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
  const model = getSelectedModel();
  const slots = isSpeechVoiceReferenceModel(model)
    ? [
        {
          key: "primary",
          label: primaryReferenceSlotLabel(model),
          asset: state.primaryReference,
          detail: state.primaryReference
            ? primaryReferenceFilledDetail(model)
            : primaryReferenceEmptyDetail(model),
        },
      ]
    : [
        {
          key: "primary",
          label: primaryReferenceSlotLabel(model),
          asset: state.primaryReference,
          detail: state.primaryReference
            ? primaryReferenceFilledDetail(model)
            : primaryReferenceEmptyDetail(model),
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

  clearPreparedHandoff();
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

  clearPreparedHandoff();
  renderReferenceIntentControls();
  renderTrayPreview();
  syncActionState();
}

function clearReferenceSlots() {
  state.primaryReference = null;
  state.endReference = null;
  state.controlReference = null;
  clearPreparedHandoff();
  renderReferenceIntentControls();
  renderTrayPreview();
  syncActionState();
}

function formatAssignedReferenceBanner() {
  const primaryLabel = primaryReferenceSlotLabel();
  const primary = state.primaryReference
    ? `${state.primaryReference.name} (${primaryReferenceFilledDetail()})`
    : "none";
  if (isSpeechVoiceReferenceModel()) {
    return `${primaryLabel}: ${primary}`;
  }
  const end = state.endReference ? state.endReference.name : "none";
  const control = state.controlReference ? state.controlReference.name : "none";
  return `${primaryLabel}: ${primary} | End: ${end} | Control: ${control}`;
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

  if (isSpeechVoiceReferenceModel(model)) {
    if (state.primaryReference) {
      const primarySupported =
        state.primaryReference.kind === "audio" && model.supports_audio_reference;
      if (!primarySupported) {
        state.primaryReference = null;
      }
    }

    state.endReference = null;
    state.controlReference = null;
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
      (state.controlReference.kind === "video" || state.controlReference.kind === "gif")
      && model.supports_video_reference;
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
  const prepareKind = elements.prepareKindInput.value;
  const prepareSupported = model && model.runtime_supported && kindSupported(model, prepareKind);
  elements.prepareRequestButton.disabled = state.preparing || state.generating || !prepareSupported || !assignedReferencesValid;
  elements.clearPreparedButton.disabled = !state.preparedHandoff;
  elements.actionButtons.forEach((button) => {
    const supported = model && model.runtime_supported && kindSupported(model, button.dataset.kind);
    button.disabled = state.generating || state.preparing || !supported || !assignedReferencesValid;
  });
}

function buildAcceptedMessage(model, kind) {
  const assistMode = elements.promptAssistInput.value;
  const usingPreparedHandoff = state.preparedHandoff && state.preparedHandoff.kind === kind;
  const assistNote = usingPreparedHandoff
    ? " Preview Handoff was locked in for this run."
    : assistMode === "off"
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

function clearPreparedHandoff() {
  if (!state.preparedHandoff) {
    renderPreparedHandoff();
    syncActionState();
    return;
  }

  state.preparedHandoff = null;
  renderPreparedHandoff();
  syncActionState();
}

function renderPreparedHandoff() {
  const handoff = state.preparedHandoff;
  if (!handoff) {
    elements.preparedEmpty.classList.remove("hidden");
    elements.preparedPanel.classList.add("hidden");
    elements.preparedMetaChips.innerHTML = "";
    elements.preparedNote.textContent = "";
    elements.preparedPromptTitle.textContent = "Prepared Prompt";
    elements.preparedPromptInput.value = "";
    elements.preparedPromptInput.placeholder = "The compiled handoff prompt will appear here after Preview Handoff runs.";
    elements.preparedSpokenBlock.classList.add("hidden");
    elements.preparedSpokenInput.value = "";
    elements.preparedNegativeBlock.classList.remove("hidden");
    elements.preparedNegativeInput.value = "";
    elements.preparedEstimate.innerHTML = "";
    elements.preparedFocusTags.innerHTML = "";
    elements.preparedAssumptions.textContent = "";
    return;
  }

  const isSpeechAudio = handoff.kind === "audio" && handoff.supports_voice_output;
  const isSoundAudio = handoff.kind === "audio" && !handoff.supports_voice_output;
  elements.preparedEmpty.classList.add("hidden");
  elements.preparedPanel.classList.remove("hidden");
  elements.preparedPromptTitle.textContent = isSpeechAudio
    ? "Speech Direction"
    : isSoundAudio
      ? "Prepared Description"
      : "Prepared Prompt";
  elements.preparedPromptInput.placeholder = isSpeechAudio
    ? "Optional delivery direction, tone, pacing, or voice feel. This field is not spoken aloud."
    : isSoundAudio
      ? "Only the descriptive sound direction appears here. Your Words / Sounds boxes stay separate and verbatim."
      : "The compiled handoff prompt will appear here after Preview Handoff runs.";
  elements.preparedPromptInput.value = handoff.prepared_prompt || "";
  elements.preparedSpokenBlock.classList.toggle("hidden", !isSpeechAudio);
  elements.preparedSpokenInput.value = handoff.prepared_spoken_text || "";
  elements.preparedNegativeBlock.classList.toggle("hidden", isSpeechAudio);
  elements.preparedNegativeInput.value = handoff.effective_negative_prompt || "";

  const chips = [
    createPreparedChip(`For ${formatKind(handoff.kind)}`),
    createPreparedChip(handoff.resolution_label || "Current settings"),
    createPreparedChip(`Estimate ${formatDurationRange(handoff.estimated_time)}`),
    handoff.interpreter_model ? createPreparedChip(`Interpreter ${handoff.interpreter_model}`) : "",
    handoff.selected_lora_name
      ? createPreparedChip(
          `LoRA ${handoff.selected_lora_name} @ ${Number(handoff.selected_lora_weight || 1).toFixed(2)}`
        )
      : "",
    handoff.used_original_prompt ? createPreparedChip("Using original wording") : "",
    isSpeechAudio ? createPreparedChip("Speech handoff") : "",
    isSoundAudio ? createPreparedChip("Literal sound lane kept separate") : "",
  ].filter(Boolean);
  elements.preparedMetaChips.innerHTML = chips.join("");

  const noteParts = [
    handoff.note,
    handoff.reference_note,
    handoff.hardware_note,
  ].filter(Boolean);
  elements.preparedNote.textContent = noteParts.join(" ");

  const estimateParts = [
    `<strong>${escapeHtml(formatDurationRange(handoff.estimated_time))}</strong>`,
    `<span>${escapeHtml(handoff.estimated_time.note || "")}</span>`,
    handoff.estimated_frames
      ? `<span>${escapeHtml(`${handoff.estimated_frames} frame(s) estimated for this ${formatKind(handoff.kind).toLowerCase()} run.`)}</span>`
      : "",
    `<span>${escapeHtml(`Confidence: ${formatEstimateConfidence(handoff.estimated_time.confidence)}.`)}</span>`,
  ].filter(Boolean);
  elements.preparedEstimate.innerHTML = estimateParts.join("");

  elements.preparedFocusTags.innerHTML = (handoff.focus_tags || []).length
    ? handoff.focus_tags.map((tag) => createPreparedChip(tag)).join("")
    : `<span class="prepared-copy">No extra focus cues were added.</span>`;
  elements.preparedAssumptions.textContent = (handoff.assumptions || []).length
    ? handoff.assumptions.join(" | ")
    : "No assumptions were needed for this handoff.";
}

function createPreparedChip(label) {
  return `<span class="prepared-chip">${escapeHtml(label)}</span>`;
}

function formatDurationRange(estimate) {
  const min = Number(estimate?.min_seconds || 0);
  const max = Number(estimate?.max_seconds || 0);
  if (!min && !max) {
    return "Unknown time";
  }
  return `${formatSeconds(min)} to ${formatSeconds(Math.max(min, max))}`;
}

function formatSeconds(totalSeconds) {
  const seconds = Math.max(0, Math.round(totalSeconds));
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  if (minutes <= 0) {
    return `${remainder}s`;
  }
  if (remainder === 0) {
    return `${minutes}m`;
  }
  return `${minutes}m ${remainder}s`;
}

function formatEstimateConfidence(confidence) {
  if (confidence === "high") return "high";
  if (confidence === "medium") return "medium";
  return "low";
}

function buildBasePayload(kind) {
  const model = getSelectedModel();
  const prompt = elements.promptInput.value.trim();
  const negativePrompt = elements.negativePromptInput.value.trim();
  const audioLiteralPrompt = elements.audioLiteralPromptInput.value.trim();
  const includeAudioLiteral =
    kind === "audio"
    && model
    && model.backend === "audio_runtime";
  const audioSegments = includeAudioLiteral && state.workflowMode === "advanced"
    ? getNormalizedAudioSegments()
    : [];
  const includeManualPromptControls = supportsManualPromptAssistInputs(model);
  const includeLora = supportsLoraControl(model) && elements.loraInput.value;
  return {
    prompt,
    negative_prompt: negativePrompt ? negativePrompt : null,
    audio_literal_prompt:
      includeAudioLiteral && !audioSegments.length && audioLiteralPrompt ? audioLiteralPrompt : null,
    audio_segments: audioSegments,
    manual_focus_tags: includeManualPromptControls ? parsePromptListInput(elements.manualFocusCuesInput.value) : [],
    manual_assumptions: includeManualPromptControls ? parsePromptListInput(elements.manualAssumptionsInput.value) : [],
    prompt_assist: elements.promptAssistInput.value,
    model: model.id,
    kind,
    style: state.generationStyle,
    settings: {
      temperature: Number(elements.temperatureInput.value),
      steps: Number(elements.stepsInput.value),
      cfg_scale: Number(elements.cfgInput.value),
      sampler: elements.samplerInput.value,
      scheduler: elements.schedulerInput.value,
      reference_strength: Number(elements.referenceStrengthInput.value),
      flow_shift: Number(elements.flowShiftInput.value),
      resolution: elements.resolutionInput.value,
      video_resolution: elements.videoResolutionInput.value,
      video_duration_seconds: Number(elements.videoDurationInput.value),
      video_fps: Number(elements.videoFpsInput.value),
      audio_duration_seconds: Number(elements.audioDurationInput.value),
      low_vram_mode: state.generationStyle === "realism" && elements.lowVramInput.checked,
      seed: null,
    },
    reference_asset: state.primaryReference ? state.primaryReference.id : null,
    reference_intent: state.referenceIntent,
    end_reference_asset: state.endReference ? state.endReference.id : null,
    control_reference_asset: state.controlReference ? state.controlReference.id : null,
    selected_lora: includeLora ? elements.loraInput.value : null,
    selected_lora_weight: includeLora ? Number(elements.loraWeightInput.value) : null,
    prepared_prompt: null,
    prepared_negative_prompt: null,
    prepared_note: null,
    prepared_interpreter_model: null,
    prepared_spoken_text: null,
  };
}

function parsePromptListInput(value) {
  return String(value || "")
    .split(/[\n,|]+/)
    .map((part) => part.trim())
    .filter(Boolean);
}

async function prepareGenerationRequest() {
  const model = getSelectedModel();
  const kind = elements.prepareKindInput.value;

  if (!model) {
    setProgress(0, "Model", `Choose a ${state.generationStyle} model first.`);
    return;
  }

  if (!kindSupported(model, kind)) {
    setProgress(0, "Mode", `${model.name} does not currently support ${kind} generation in ${state.generationStyle} mode.`);
    return;
  }

  if (!areAssignedReferencesCompatible(model)) {
    setProgress(0, "Reference", getAssignedReferenceValidationMessage(model));
    return;
  }

  const prompt = elements.promptInput.value.trim();
  const audioLiteralPrompt = elements.audioLiteralPromptInput.value.trim();
  const canUseAudioLiteral = model.backend === "audio_runtime" && kind === "audio";
  const audioSegments = canUseAudioLiteral && state.workflowMode === "advanced"
    ? getNormalizedAudioSegments()
    : [];
  if (!prompt && !(canUseAudioLiteral && (audioLiteralPrompt || audioSegments.length))) {
    setProgress(0, "Prompt", "Type a prompt or fill in the audio Words / Script / Sounds area first.");
    elements.promptInput.focus();
    return;
  }

  let seed = null;
  try {
    seed = parseSeedInput();
  } catch (error) {
    setProgress(0, "Seed", error.message);
    elements.seedInput.focus();
    return;
  }

  const payload = buildBasePayload(kind);
  payload.settings.seed = seed;

  state.preparing = true;
  syncActionState();
  setProgress(0.06, "Previewing", `Preparing a handoff preview for ${formatKind(kind).toLowerCase()} generation.`);

  try {
    const response = await fetch("/api/prepare", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const message = await response.text();
      throw new Error(message || "Preview Handoff failed.");
    }

    state.preparedHandoff = await response.json();
    renderPreparedHandoff();
    setProgress(0.12, "Preview Ready", `Preview Handoff is ready for ${formatKind(kind).toLowerCase()} generation. Review and edit it before you lock in.`);
  } catch (error) {
    state.preparedHandoff = null;
    renderPreparedHandoff();
    setProgress(0, "Error", error.message || "Preview Handoff failed.");
  } finally {
    state.preparing = false;
    syncActionState();
  }
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
  const audioLiteralPrompt = elements.audioLiteralPromptInput.value.trim();
  const canUseAudioLiteral = model.backend === "audio_runtime" && kind === "audio";
  const audioSegments = canUseAudioLiteral && state.workflowMode === "advanced"
    ? getNormalizedAudioSegments()
    : [];
  if (!prompt && !(canUseAudioLiteral && (audioLiteralPrompt || audioSegments.length))) {
    setProgress(0, "Prompt", "Type a prompt or fill in the audio Words / Script / Sounds area first.");
    elements.promptInput.focus();
    return;
  }

  const currentRisk = state.hardwareProfile
    ? assessCurrentKindPressure(model, kind, state.hardwareProfile)
    : null;

  state.generating = true;
  syncActionState();
  setProgress(
    0.04,
    "Queued",
    state.generationStyle === "realism"
      ? `Submitting ${kind} job to the local stable-diffusion.cpp realism backend.${currentRisk?.tone === "risky" ? ` Warning: ${currentRisk.note}` : ""}`
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

  const payload = buildBasePayload(kind);
  payload.settings.seed = seed;
  if (state.preparedHandoff && state.preparedHandoff.kind === kind) {
    const preparedPrompt = elements.preparedPromptInput.value.trim();
    payload.prepared_prompt = preparedPrompt ? preparedPrompt : null;
    const preparedNegative = elements.preparedNegativeInput.value.trim();
    payload.prepared_negative_prompt = preparedNegative ? preparedNegative : null;
    const preparedSpokenText = elements.preparedSpokenInput.value.trim();
    payload.prepared_spoken_text = preparedSpokenText ? preparedSpokenText : null;
    payload.prepared_note = state.preparedHandoff.note || "Preview Handoff was reviewed before generation.";
    payload.prepared_interpreter_model = state.preparedHandoff.interpreter_model || null;
  }

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
    if (state.controlReference.kind !== "video" && state.controlReference.kind !== "gif") {
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
    return "The start image must be a still image from the tray.";
  }

  if (state.primaryReference && !model.supports_image_reference && !model.requires_reference) {
    return "This realism model does not use a start image in Chatty-art yet.";
  }

  if (model.requires_end_image_reference && !state.endReference) {
    return "This realism model needs an end image in the Input Tray before it can generate.";
  }

  if (state.endReference && state.endReference.kind !== "image") {
    return "The end frame must be a still image from the tray.";
  }

  if (state.endReference && !model.supports_end_image_reference && !model.requires_end_image_reference) {
    return "This realism model does not use an end-frame image in Chatty-art yet.";
  }

  if (state.controlReference
      && state.controlReference.kind !== "video"
      && state.controlReference.kind !== "gif") {
    return "Control-video input must be a video or GIF from the tray.";
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

function formatBackendBadge(backend) {
  if (backend === "stable_diffusion_cpp") return "stable-diffusion.cpp";
  if (backend === "audio_runtime") return "audio runtime";
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
