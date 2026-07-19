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
  currentBatchTotal: 1,
  currentBatchCompleted: 0,
  activeFilter: "all",
  generating: false,
  canceling: false,
  preparing: false,
  generationStyle: "expressive",
  workflowMode: "basic",
  audioSegments: [],
  loraSelections: [],
  cloudProviders: [],
  cloudApiKeyLanesEnabled: false,
  cloudLaneAssignments: {
    prompt_assist: "local_auto",
    vision_assist: "local_auto",
    media_generation: "local_only",
  },
  preparedHandoff: null,
  handoffTargets: [],
  selectedOutputIds: new Set(),
  handoffSending: false,
  handoffStatusMessage: "",
  loraInbox: {
    loading: false,
    importing: false,
    assets: [],
    selectedAssetIds: new Set(),
    statusMessage: "",
  },
};

let lastBridgeStatusFingerprint = "";
let startupSplashDismissed = false;
let startupSplashTimerHandle = null;

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
  manualPreserveBlock: document.getElementById("manualPreserveBlock"),
  manualPreserveInput: document.getElementById("manualPreserveInput"),
  manualChangeBlock: document.getElementById("manualChangeBlock"),
  manualChangeInput: document.getElementById("manualChangeInput"),
  manualAvoidBlock: document.getElementById("manualAvoidBlock"),
  manualAvoidInput: document.getElementById("manualAvoidInput"),
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
  preparedVisionBlock: document.getElementById("preparedVisionBlock"),
  preparedVisionTitle: document.getElementById("preparedVisionTitle"),
  preparedVisionSummary: document.getElementById("preparedVisionSummary"),
  preparedFocusTags: document.getElementById("preparedFocusTags"),
  preparedAssumptions: document.getElementById("preparedAssumptions"),
  styleButtons: [...document.querySelectorAll("[data-style]")],
  workflowButtons: [...document.querySelectorAll("[data-workflow]")],
  styleSummary: document.getElementById("styleSummary"),
  workflowSummary: document.getElementById("workflowSummary"),
  runtimeBadges: document.getElementById("runtimeBadges"),
  modelSelect: document.getElementById("modelSelect"),
  modelSummary: document.getElementById("modelSummary"),
  mediaGenerationCloudBlock: document.getElementById("mediaGenerationCloudBlock"),
  mediaGenerationLaneSelect: document.getElementById("mediaGenerationLaneSelect"),
  mediaGenerationLaneSummary: document.getElementById("mediaGenerationLaneSummary"),
  mediaGenerationCloudEntrySelect: document.getElementById("mediaGenerationCloudEntrySelect"),
  mediaGenerationCloudStatus: document.getElementById("mediaGenerationCloudStatus"),
  mediaGenerationCloudNameInput: document.getElementById("mediaGenerationCloudNameInput"),
  mediaGenerationCloudProviderInput: document.getElementById("mediaGenerationCloudProviderInput"),
  mediaGenerationCloudBaseUrlInput: document.getElementById("mediaGenerationCloudBaseUrlInput"),
  mediaGenerationCloudImageModelInput: document.getElementById("mediaGenerationCloudImageModelInput"),
  mediaGenerationCloudVideoModelInput: document.getElementById("mediaGenerationCloudVideoModelInput"),
  mediaGenerationCloudAudioModelInput: document.getElementById("mediaGenerationCloudAudioModelInput"),
  mediaGenerationCloudVoiceInput: document.getElementById("mediaGenerationCloudVoiceInput"),
  mediaGenerationCloudApiKeyInput: document.getElementById("mediaGenerationCloudApiKeyInput"),
  mediaGenerationCloudEnabledInput: document.getElementById("mediaGenerationCloudEnabledInput"),
  saveMediaGenerationCloudButton: document.getElementById("saveMediaGenerationCloudButton"),
  verifyMediaGenerationCloudButton: document.getElementById("verifyMediaGenerationCloudButton"),
  deleteMediaGenerationCloudButton: document.getElementById("deleteMediaGenerationCloudButton"),
  promptModelBlock: document.getElementById("promptModelBlock"),
  promptModelSelect: document.getElementById("promptModelSelect"),
  promptModelSummary: document.getElementById("promptModelSummary"),
  promptAssistCloudBlock: document.getElementById("promptAssistCloudBlock"),
  promptAssistLaneSelect: document.getElementById("promptAssistLaneSelect"),
  promptAssistLaneSummary: document.getElementById("promptAssistLaneSummary"),
  promptAssistCloudEntrySelect: document.getElementById("promptAssistCloudEntrySelect"),
  promptAssistCloudStatus: document.getElementById("promptAssistCloudStatus"),
  promptAssistCloudNameInput: document.getElementById("promptAssistCloudNameInput"),
  promptAssistCloudProviderInput: document.getElementById("promptAssistCloudProviderInput"),
  promptAssistCloudBaseUrlInput: document.getElementById("promptAssistCloudBaseUrlInput"),
  promptAssistCloudModelInput: document.getElementById("promptAssistCloudModelInput"),
  promptAssistCloudApiKeyInput: document.getElementById("promptAssistCloudApiKeyInput"),
  promptAssistCloudEnabledInput: document.getElementById("promptAssistCloudEnabledInput"),
  savePromptAssistCloudButton: document.getElementById("savePromptAssistCloudButton"),
  verifyPromptAssistCloudButton: document.getElementById("verifyPromptAssistCloudButton"),
  deletePromptAssistCloudButton: document.getElementById("deletePromptAssistCloudButton"),
  visionModelBlock: document.getElementById("visionModelBlock"),
  visionModelSelect: document.getElementById("visionModelSelect"),
  visionModelSummary: document.getElementById("visionModelSummary"),
  visionAssistCloudBlock: document.getElementById("visionAssistCloudBlock"),
  visionAssistLaneSelect: document.getElementById("visionAssistLaneSelect"),
  visionAssistLaneSummary: document.getElementById("visionAssistLaneSummary"),
  visionAssistCloudEntrySelect: document.getElementById("visionAssistCloudEntrySelect"),
  visionAssistCloudStatus: document.getElementById("visionAssistCloudStatus"),
  visionAssistCloudNameInput: document.getElementById("visionAssistCloudNameInput"),
  visionAssistCloudProviderInput: document.getElementById("visionAssistCloudProviderInput"),
  visionAssistCloudBaseUrlInput: document.getElementById("visionAssistCloudBaseUrlInput"),
  visionAssistCloudModelInput: document.getElementById("visionAssistCloudModelInput"),
  visionAssistCloudApiKeyInput: document.getElementById("visionAssistCloudApiKeyInput"),
  visionAssistCloudEnabledInput: document.getElementById("visionAssistCloudEnabledInput"),
  saveVisionAssistCloudButton: document.getElementById("saveVisionAssistCloudButton"),
  verifyVisionAssistCloudButton: document.getElementById("verifyVisionAssistCloudButton"),
  deleteVisionAssistCloudButton: document.getElementById("deleteVisionAssistCloudButton"),
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
  loraList: document.getElementById("loraList"),
  addLoraButton: document.getElementById("addLoraButton"),
  loraDetectedCount: document.getElementById("loraDetectedCount"),
  loraCopy: document.getElementById("loraCopy"),
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
  batchCountInput: document.getElementById("batchCountInput"),
  batchCountCopy: document.getElementById("batchCountCopy"),
  cancelGenerateButton: document.getElementById("cancelGenerateButton"),
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
  outputHandoffPanel: document.getElementById("outputHandoffPanel"),
  outputHandoffSummary: document.getElementById("outputHandoffSummary"),
  outputHandoffNote: document.getElementById("outputHandoffNote"),
  outputHandoffPreview: document.getElementById("outputHandoffPreview"),
  sendOutputsToLoraButton: document.getElementById("sendOutputsToLoraButton"),
  sendOutputsToSandboxButton: document.getElementById("sendOutputsToSandboxButton"),
  deleteSelectedOutputsButton: document.getElementById("deleteSelectedOutputsButton"),
  loraInboxPanel: document.getElementById("loraInboxPanel"),
  loraInboxSummary: document.getElementById("loraInboxSummary"),
  loraInboxFamilySelect: document.getElementById("loraInboxFamilySelect"),
  loraInboxList: document.getElementById("loraInboxList"),
  importLoraInboxButton: document.getElementById("importLoraInboxButton"),
  clearLoraInboxSelectionButton: document.getElementById("clearLoraInboxSelectionButton"),
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
  startupSplash: document.getElementById("startupSplash"),
};

const GPU_TELEMETRY_WIDTH = 180;
const GPU_TELEMETRY_HEIGHT = 44;

bindSettingDisplay(elements.temperatureInput, elements.temperatureValue, (value) => Number(value).toFixed(1));
bindSettingDisplay(elements.stepsInput, elements.stepsValue, (value) => `${value}`);
bindSettingDisplay(elements.cfgInput, elements.cfgValue, (value) => Number(value).toFixed(1));
bindSettingDisplay(elements.referenceStrengthInput, elements.referenceStrengthValue, (value) => Number(value).toFixed(2));
bindSettingDisplay(elements.flowShiftInput, elements.flowShiftValue, (value) => Number(value).toFixed(1));

const trackedSettingInputs = [
  elements.temperatureInput,
  elements.stepsInput,
  elements.cfgInput,
  elements.samplerInput,
  elements.schedulerInput,
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
elements.batchCountInput.addEventListener("input", () => {
  refreshBatchCountCopy();
  clearPreparedHandoff();
});
elements.addLoraButton.addEventListener("click", () => {
  addLoraSelection();
  renderAdvancedRealismSettings();
  syncActionState();
});
elements.modelSelect.addEventListener("change", async () => {
  clearPreparedHandoff();
  const selection = elements.modelSelect.value || "";
  const currentLane = state.cloudLaneAssignments?.media_generation || "local_only";
  if (selection.startsWith("cloud-route:")) {
    const providerRoute = mediaGenerationLaneSelectionFromRouteValue(selection);
    if (providerRoute && currentLane !== providerRoute) {
      elements.mediaGenerationLaneSelect.value = providerRoute;
      await saveMediaGenerationLaneSelection();
      return;
    }
  } else if (selection.startsWith("cloud:")) {
    if (currentLane !== selection) {
      elements.mediaGenerationLaneSelect.value = selection;
      await saveMediaGenerationLaneSelection();
      return;
    }
  } else if (currentLane.startsWith("cloud:")) {
    elements.mediaGenerationLaneSelect.value = "local_only";
    await saveMediaGenerationLaneSelection();
    return;
  }
  renderStyleMode();
  renderPrepareKindOptions();
  renderModelSummary();
  renderPromptModelSelector();
  renderVisionModelSelector();
  refreshAudioSettingCopy();
  refreshBatchCountCopy();
  renderReferenceIntentControls();
  syncActionState();
});
elements.promptModelSelect.addEventListener("change", async () => {
  clearPreparedHandoff();
  const selection = elements.promptModelSelect.value || "";
  const currentLane = state.cloudLaneAssignments?.prompt_assist || "local_auto";
  if (selection.startsWith("cloud:")) {
    if (currentLane !== selection) {
      elements.promptAssistLaneSelect.value = selection;
      await savePromptAssistLaneSelection();
      return;
    }
  } else if (currentLane.startsWith("cloud:")) {
    elements.promptAssistLaneSelect.value = "local_auto";
    await savePromptAssistLaneSelection();
    return;
  }
  renderPromptModelSelector();
  syncActionState();
});
elements.promptAssistLaneSelect.addEventListener("change", () => {
  savePromptAssistLaneSelection();
});
elements.promptAssistCloudEntrySelect.addEventListener("change", () => {
  populatePromptAssistCloudForm();
  renderPromptAssistCloudControls();
});
elements.promptAssistCloudProviderInput.addEventListener("change", () => {
  const defaults = cloudProviderDefaults(elements.promptAssistCloudProviderInput.value);
  elements.promptAssistCloudBaseUrlInput.value = defaults.baseUrl;
  elements.promptAssistCloudModelInput.value = defaults.model;
});
elements.savePromptAssistCloudButton.addEventListener("click", () => {
  savePromptAssistCloudProvider();
});
elements.verifyPromptAssistCloudButton.addEventListener("click", () => {
  verifyPromptAssistCloudProvider();
});
elements.deletePromptAssistCloudButton.addEventListener("click", () => {
  deletePromptAssistCloudProvider();
});
elements.mediaGenerationLaneSelect.addEventListener("change", () => {
  saveMediaGenerationLaneSelection();
});
elements.mediaGenerationCloudEntrySelect.addEventListener("change", () => {
  populateMediaGenerationCloudForm();
  renderMediaGenerationCloudControls();
});
elements.mediaGenerationCloudProviderInput.addEventListener("change", () => {
  const defaults = cloudProviderDefaults(elements.mediaGenerationCloudProviderInput.value);
  elements.mediaGenerationCloudBaseUrlInput.value = defaults.baseUrl;
  elements.mediaGenerationCloudImageModelInput.value = defaults.imageModel || defaults.model;
  elements.mediaGenerationCloudVideoModelInput.value = defaults.videoModel || "";
  elements.mediaGenerationCloudAudioModelInput.value = defaults.audioModel || "";
  elements.mediaGenerationCloudVoiceInput.value = defaults.audioVoice || "";
});
elements.saveMediaGenerationCloudButton.addEventListener("click", () => {
  saveMediaGenerationCloudProvider();
});
elements.verifyMediaGenerationCloudButton.addEventListener("click", () => {
  verifyMediaGenerationCloudProvider();
});
elements.deleteMediaGenerationCloudButton.addEventListener("click", () => {
  deleteMediaGenerationCloudProvider();
});
elements.visionAssistLaneSelect.addEventListener("change", () => {
  saveVisionAssistLaneSelection();
});
elements.visionAssistCloudEntrySelect.addEventListener("change", () => {
  populateVisionAssistCloudForm();
  renderVisionAssistCloudControls();
});
elements.visionAssistCloudProviderInput.addEventListener("change", () => {
  const defaults = cloudProviderDefaults(elements.visionAssistCloudProviderInput.value);
  elements.visionAssistCloudBaseUrlInput.value = defaults.baseUrl;
  elements.visionAssistCloudModelInput.value = defaults.model;
});
elements.saveVisionAssistCloudButton.addEventListener("click", () => {
  saveVisionAssistCloudProvider();
});
elements.verifyVisionAssistCloudButton.addEventListener("click", () => {
  verifyVisionAssistCloudProvider();
});
elements.deleteVisionAssistCloudButton.addEventListener("click", () => {
  deleteVisionAssistCloudProvider();
});
elements.visionModelSelect.addEventListener("change", async () => {
  clearPreparedHandoff();
  const selection = elements.visionModelSelect.value || "";
  const currentLane = state.cloudLaneAssignments?.vision_assist || "local_auto";
  if (selection.startsWith("cloud:")) {
    if (currentLane !== selection) {
      elements.visionAssistLaneSelect.value = selection;
      await saveVisionAssistLaneSelection();
      return;
    }
  } else if (currentLane.startsWith("cloud:")) {
    elements.visionAssistLaneSelect.value = "local_auto";
    await saveVisionAssistLaneSelection();
    return;
  }
  renderVisionModelSelector();
  syncActionState();
});
elements.actionButtons.forEach((button) => {
  button.addEventListener("click", () => submitGeneration(button.dataset.kind));
});
elements.cancelGenerateButton.addEventListener("click", () => cancelCurrentGeneration());
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
    refreshBatchCountCopy();
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
    refreshBatchCountCopy();
    syncActionState();
  });
});

function handleTrackedSettingMutation() {
  state.lastAutoDefaultsStyle = null;
  refreshVideoSettingCopy();
  refreshBatchCountCopy();
  clearPreparedHandoff();
  renderAdvancedRealismSettings();
  renderModelSummary();
}

function dismissStartupSplash() {
  if (startupSplashDismissed || !elements.startupSplash) {
    return;
  }
  startupSplashDismissed = true;
  if (startupSplashTimerHandle) {
    window.clearTimeout(startupSplashTimerHandle);
    startupSplashTimerHandle = null;
  }
  elements.startupSplash.classList.add("hidden");
  document.body.classList.remove("splash-active");
  window.removeEventListener("keydown", handleStartupSplashKeydown);
}

function handleStartupSplashKeydown(event) {
  if (event.key === " " || event.key === "Enter" || event.key === "Escape") {
    event.preventDefault();
    dismissStartupSplash();
  }
}

function initStartupSplash() {
  if (!elements.startupSplash) {
    return;
  }
  document.body.classList.add("splash-active");
  elements.startupSplash.addEventListener("click", dismissStartupSplash);
  window.addEventListener("keydown", handleStartupSplashKeydown);
  startupSplashTimerHandle = window.setTimeout(dismissStartupSplash, 3000);
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
elements.promptAssistInput.addEventListener("change", () => renderPromptModelSelector());
elements.promptAssistInput.addEventListener("change", () => renderVisionModelSelector());
elements.prepareKindInput.addEventListener("change", () => {
  clearPreparedHandoff();
  renderModels();
  normalizeAssignedReferencesForCurrentModel();
  renderReferenceIntentControls();
  renderVisionModelSelector();
  renderTrayPreview();
  refreshBatchCountCopy();
  syncActionState();
});
elements.outputHandoffNote.addEventListener("input", () => renderOutputHandoffPanel());
elements.sendOutputsToLoraButton.addEventListener("click", () => sendSelectedOutputsToLora());
elements.sendOutputsToSandboxButton.addEventListener("click", () => sendSelectedOutputsToSandbox());
elements.deleteSelectedOutputsButton.addEventListener("click", () => deleteSelectedOutputs());
elements.loraInboxFamilySelect.addEventListener("change", () => renderLoraInboxPanel());
elements.importLoraInboxButton.addEventListener("click", () => importSelectedLoraInboxAssets());
elements.clearLoraInboxSelectionButton.addEventListener("click", () => {
  state.loraInbox.selectedAssetIds.clear();
  renderLoraInboxPanel();
});
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
initStartupSplash();
applyModeDefaults(state.generationStyle);
renderStyleMode();
renderPromptWorkflowMode();
renderPrepareKindOptions();
renderPreparedHandoff();
refreshBatchCountCopy();
refreshEverything();
loadAvailableHandoffTargets();
loadLoraInboxAssets();
startGpuTelemetryPolling();
syncChattyCogBridgeStatus();
window.setInterval(syncChattyCogBridgeStatus, 2000);
window.setInterval(() => {
  loadLoraInboxAssets({ silent: true });
}, 3000);

async function refreshEverything() {
  await Promise.all([
    loadRuntimeStatus(),
    loadHardwareProfile(),
    loadCloudProviders(),
    loadCloudLaneAssignments(),
    loadModels(),
    loadLoras(),
    loadAssets(),
    loadOutputs(),
    loadGpuTelemetry(),
    loadAvailableHandoffTargets(),
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

async function loadCloudProviders() {
  try {
    const response = await fetchJson("/api/cloud/providers");
    state.cloudProviders = Array.isArray(response.providers) ? response.providers : [];
    state.cloudApiKeyLanesEnabled = Boolean(response.api_key_lanes_enabled);
  } catch {
    state.cloudProviders = [];
    state.cloudApiKeyLanesEnabled = false;
  }
  renderMediaGenerationCloudControls();
  renderPromptAssistCloudControls();
  renderVisionAssistCloudControls();
}

async function loadCloudLaneAssignments() {
  try {
    const response = await fetchJson("/api/cloud/lanes");
    state.cloudLaneAssignments = response.lane_assignments || {
      prompt_assist: "local_auto",
      vision_assist: "local_auto",
      media_generation: "local_only",
    };
  } catch {
    state.cloudLaneAssignments = {
      prompt_assist: "local_auto",
      vision_assist: "local_auto",
      media_generation: "local_only",
    };
  }
  renderMediaGenerationCloudControls();
  renderPromptAssistCloudControls();
  renderVisionAssistCloudControls();
}

async function loadHardwareProfile() {
  try {
    state.hardwareProfile = await fetchJson("/api/hardware");
  } catch {
    state.hardwareProfile = null;
  }

  refreshBatchCountCopy();
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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const seconds = Math.max(1, Number(elements.audioDurationInput.value || 0));
  const segmentCount = getNormalizedAudioSegments().length;
  const isStableAudio = selectedModel?.backend === "audio_runtime" && !selectedModel?.supports_voice_output;
  const isCloudSpeechAudio = Boolean(
    activeMediaCloudProvider
    && elements.prepareKindInput.value === "audio"
    && mediaGenerationSupportedKinds(activeMediaCloudProvider).includes("audio")
  );
  const isSpeechAudio = (selectedModel?.backend === "audio_runtime" && selectedModel?.supports_voice_output)
    || isCloudSpeechAudio;

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
    if (isCloudSpeechAudio) {
      elements.audioDurationCopy.textContent = "Cloud speech output mainly cares about spoken text length and delivery notes. This duration control is mostly for local soundscape/SFX audio.";
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
  const selectedLoras = getSelectedLoraDetails(model);
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
    elements.loraCopy.textContent = describeLoraSelection(model, selectedLoras);
    elements.loraWeightCopy.textContent = describeLoraWeightSetting(selectedLoras);
  }
}

function createLoraSelection(seed = {}) {
  return {
    id: seed.id || "",
    weight: Number.isFinite(Number(seed.weight)) ? Number(seed.weight) : 1.0,
  };
}

function getSelectedLoraDetails(model = getSelectedModel()) {
  return getNormalizedLoraSelections()
    .map((selection) => {
      const lora = getCompatibleLoras(model).find((entry) => entry.id === selection.id) || null;
      return lora
        ? { ...selection, lora }
        : null;
    })
    .filter(Boolean);
}

function getNormalizedLoraSelections() {
  const seen = new Set();
  return state.loraSelections
    .map((selection) => ({
      id: String(selection.id || "").trim(),
      weight: clampLoraWeight(selection.weight),
    }))
    .filter((selection) => selection.id)
    .filter((selection) => {
      const key = selection.id.toLowerCase();
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });
}

function clampLoraWeight(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) {
    return 1;
  }
  return Math.min(2, Math.max(0, numeric));
}

function ensureLoraSelectionSlots() {
  if (!state.loraSelections.length) {
    state.loraSelections = [createLoraSelection()];
  }
}

function addLoraSelection(seed = {}) {
  if (state.loraSelections.length >= 6) {
    return;
  }
  state.loraSelections.push(createLoraSelection(seed));
}

function removeLoraSelection(index) {
  state.loraSelections = state.loraSelections.filter((_, currentIndex) => currentIndex !== index);
  ensureLoraSelectionSlots();
}

function updateLoraSelection(index, nextValue) {
  const current = state.loraSelections[index];
  if (!current) {
    return;
  }
  state.loraSelections[index] = {
    ...current,
    ...nextValue,
  };
}

function renderLoraSelections(model) {
  const compatibleLoras = getCompatibleLoras(model);
  ensureLoraSelectionSlots();

  elements.loraList.innerHTML = state.loraSelections.map((selection, index) => {
    const options = [
      `<option value="">No LoRA</option>`,
      ...compatibleLoras.map((lora) => `<option value="${escapeHtml(lora.id)}" ${lora.id === selection.id ? "selected" : ""}>${escapeHtml(lora.name)} | ${escapeHtml(lora.family)}</option>`),
    ];
    const canRemove = state.loraSelections.length > 1;
    return `
      <div class="lora-stack-row" data-lora-index="${index}">
        <div class="lora-stack-row-head">
          <span class="lora-stack-row-title">LoRA ${index + 1}</span>
          ${canRemove ? `<button class="ghost-button mini-ghost-button" type="button" data-remove-lora="${index}">Remove</button>` : ""}
        </div>
        <div class="lora-stack-row-grid">
          <label>
            <span>Adapter</span>
            <select data-lora-select="${index}" ${compatibleLoras.length ? "" : "disabled"}>
              ${options.join("")}
            </select>
            <span class="lora-stack-field-note">${compatibleLoras.length ? "Choose the specific LoRA file to layer into this stack slot." : "No compatible LoRAs are currently available for this model."}</span>
          </label>
          <label>
            <span>Strength</span>
            <input data-lora-weight="${index}" type="range" min="0" max="2" step="0.05" value="${clampLoraWeight(selection.weight).toFixed(2)}" ${compatibleLoras.length ? "" : "disabled"}>
            <span class="lora-stack-weight-value">${clampLoraWeight(selection.weight).toFixed(2)}</span>
            <span class="lora-stack-field-note">This slider only affects LoRA ${index + 1}. Lower keeps it subtle, higher pushes it harder.</span>
          </label>
        </div>
      </div>
    `;
  }).join("");

  elements.loraList.querySelectorAll("[data-lora-select]").forEach((select) => {
    select.addEventListener("change", (event) => {
      const index = Number(event.currentTarget.dataset.loraSelect);
      updateLoraSelection(index, { id: event.currentTarget.value });
      clearPreparedHandoff();
      renderAdvancedRealismSettings();
      syncActionState();
    });
  });

  elements.loraList.querySelectorAll("[data-lora-weight]").forEach((input) => {
    input.addEventListener("input", (event) => {
      const index = Number(event.currentTarget.dataset.loraWeight);
      updateLoraSelection(index, { weight: clampLoraWeight(event.currentTarget.value) });
      clearPreparedHandoff();
      renderAdvancedRealismSettings();
      syncActionState();
    });
  });

  elements.loraList.querySelectorAll("[data-remove-lora]").forEach((button) => {
    button.addEventListener("click", (event) => {
      const index = Number(event.currentTarget.dataset.removeLora);
      removeLoraSelection(index);
      clearPreparedHandoff();
      renderAdvancedRealismSettings();
      syncActionState();
    });
  });
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

function describeLoraSelection(model, selectedLoras) {
  const familyLabel = model?.family || "selected model";
  const familyKey = modelLoraFamilyKey(model) || "family";
  const compatibleLoras = getCompatibleLoras(model);

  if (!selectedLoras.length) {
    return compatibleLoras.length
      ? `${compatibleLoras.length} compatible LoRA${compatibleLoras.length === 1 ? "" : "s"} found for ${familyLabel}. Stack one or more if you want to bolt specific style or concept adapters on top of the base model.`
      : `No compatible LoRAs detected for ${familyLabel}. Put matching files in models/loras/${familyKey}/ or models/lora/${familyKey}/.`;
  }

  if (selectedLoras.length === 1) {
    return `${selectedLoras[0].lora.name} is active. Think of it as a small style or concept add-on sitting on top of the ${familyLabel} base model.`;
  }

  return `${selectedLoras.length} LoRAs are stacked on top of ${familyLabel}. Layer them carefully and add one adapter at a time when you are testing, because combined weights can overpower the base model quickly.`;
}

function describeLoraWeightSetting(selectedLoras) {
  if (!selectedLoras.length) {
    return "Choose at least one LoRA first. Lower weights keep the stack subtle, higher combined weights push the adapters much harder.";
  }

  const totalWeight = selectedLoras.reduce((sum, selection) => sum + selection.weight, 0);
  const strongest = selectedLoras.reduce((max, selection) => Math.max(max, selection.weight), 0);

  if (selectedLoras.length === 1) {
    const weight = strongest;
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

  if (totalWeight <= 1.5 && strongest <= 0.9) {
    return `This stack totals ${totalWeight.toFixed(2)} across ${selectedLoras.length} LoRAs. That is still fairly restrained, and usually a good place to start when layering adapters.`;
  }
  if (totalWeight <= 2.4 && strongest <= 1.2) {
    return `This stack totals ${totalWeight.toFixed(2)} across ${selectedLoras.length} LoRAs. That is a balanced multi-LoRA range, but the adapters can still interact in surprising ways.`;
  }
  return `This stack totals ${totalWeight.toFixed(2)} across ${selectedLoras.length} LoRAs. Treat that as experimental territory, because stacked adapters can overpower the base model faster than a single strong LoRA.`;
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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudSpeechAudio = Boolean(
    activeMediaCloudProvider
    && elements.prepareKindInput.value === "audio"
    && mediaGenerationSupportedKinds(activeMediaCloudProvider).includes("audio")
  );
  return Boolean(
    ((model && model.backend === "audio_runtime") || cloudSpeechAudio)
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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudSpeechAudio = Boolean(
    activeMediaCloudProvider
    && elements.prepareKindInput.value === "audio"
    && mediaGenerationSupportedKinds(activeMediaCloudProvider).includes("audio")
  );
  const isDedicatedAudioModel = (selectedModel && selectedModel.backend === "audio_runtime") || cloudSpeechAudio;
  const isSpeechAudio = cloudSpeechAudio || (selectedModel && selectedModel.backend === "audio_runtime" && selectedModel.supports_voice_output);
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
  renderVisionModelSelector();
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

  pruneSelectedOutputs();
  renderPreview();
  renderHistory();
}

async function loadAvailableHandoffTargets() {
  if (!window.chattyCogBridge?.available || typeof window.chattyCogBridge.getAvailableHandoffTargets !== "function") {
    state.handoffTargets = [];
    renderOutputHandoffPanel();
    return;
  }

  try {
    const payload = await window.chattyCogBridge.getAvailableHandoffTargets();
    state.handoffTargets = Array.isArray(payload?.targets) ? payload.targets : [];
  } catch {
    state.handoffTargets = [];
  }

  renderOutputHandoffPanel();
}

async function loadLoraInboxAssets({ silent = false } = {}) {
  if (!window.chattyCogBridge?.available || typeof window.chattyCogBridge.readIncomingAssets !== "function") {
    state.loraInbox.assets = [];
    state.loraInbox.selectedAssetIds.clear();
    renderLoraInboxPanel();
    return;
  }

  if (!silent) {
    state.loraInbox.loading = true;
    renderLoraInboxPanel();
  }

  try {
    const assets = await window.chattyCogBridge.readIncomingAssets("lora_imports");
    state.loraInbox.assets = Array.isArray(assets) ? assets : [];
    pruneLoraInboxSelection();
  } catch {
    state.loraInbox.assets = [];
    state.loraInbox.selectedAssetIds.clear();
  } finally {
    state.loraInbox.loading = false;
    renderLoraInboxPanel();
  }
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
  renderVisionModelSelector();

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
  elements.manualPreserveBlock.classList.toggle("hidden", !show);
  elements.manualChangeBlock.classList.toggle("hidden", !show);
  elements.manualAvoidBlock.classList.toggle("hidden", !show);
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
  elements.referenceStrengthCard.classList.toggle("hidden", !showReferenceStrength);
  elements.flowShiftCard.classList.toggle("hidden", !showFlowShift);
  if (showLora) {
    const familyKey = modelLoraFamilyKey(model);
    if (familyKey) {
      elements.loraInboxFamilySelect.value = familyKey;
    }
  }
  renderLoraInboxPanel();

  if (showLora) {
    const compatibleLoras = getCompatibleLoras(model);
    state.loraSelections = state.loraSelections.map((selection) => (
      compatibleLoras.some((lora) => lora.id === selection.id)
        ? selection
        : createLoraSelection({ weight: selection.weight })
    ));
    renderLoraSelections(model);
    elements.addLoraButton.disabled = !compatibleLoras.length || state.loraSelections.length >= 6;
    const familyKey = modelLoraFamilyKey(model) || "family";
    elements.loraDetectedCount.textContent = compatibleLoras.length
      ? `LoRAs detected for this model: ${compatibleLoras.length} compatible file${compatibleLoras.length === 1 ? "" : "s"} in ${familyKey}.`
      : `LoRAs detected for this model: 0 compatible files in ${familyKey}.`;

    const familyLabel = model.family || "selected";
    elements.loraCopy.textContent = compatibleLoras.length
      ? `${compatibleLoras.length} compatible LoRA${compatibleLoras.length === 1 ? "" : "s"} found for ${familyLabel}. Add one or more rows below and give each row its own strength. Put more in models/loras/${compatibleLoras[0].family_key}/ or models/lora/${compatibleLoras[0].family_key}/.`
      : `No compatible LoRAs detected for ${familyLabel}. Put them in models/loras/${familyKey}/ or models/lora/${familyKey}/. The LoRA stack panel stays visible here so you can tell this model still supports the Advanced realism LoRA path.`;
    const selectedLoras = getSelectedLoraDetails(model);
    elements.loraWeightCopy.textContent = selectedLoras.length
      ? describeLoraWeightSetting(selectedLoras)
      : compatibleLoras.length
        ? "Choose at least one LoRA first. Each row has its own strength slider, and the combined stack can get strong quickly."
        : "No compatible LoRAs are loaded for this model yet, so the stack rows stay visible as a hint but remain effectively empty.";
  } else {
    state.loraSelections = [];
    elements.loraList.innerHTML = "";
    elements.addLoraButton.disabled = true;
    elements.loraDetectedCount.textContent = "LoRA detection is waiting for an Advanced realism stable-diffusion model.";
  }

  refreshAdvancedRealismSettingCopy();
}

function renderLoraInboxPanel() {
  const hasBridge = Boolean(window.chattyCogBridge?.available);
  elements.loraInboxPanel.classList.toggle("hidden", !hasBridge);
  if (!hasBridge) {
    return;
  }

  const selectedCount = state.loraInbox.selectedAssetIds.size;
  if (state.loraInbox.loading) {
    elements.loraInboxSummary.textContent = "Checking ChattyCog for incoming LoRA files...";
  } else if (!state.loraInbox.assets.length) {
    elements.loraInboxSummary.textContent = "No LoRA handoffs are waiting right now.";
  } else {
    elements.loraInboxSummary.textContent = `${state.loraInbox.assets.length} LoRA handoff file${state.loraInbox.assets.length === 1 ? "" : "s"} waiting for explicit import into models/loras/${elements.loraInboxFamilySelect.value || "family"}/.`;
  }

  if (state.loraInbox.statusMessage) {
    elements.loraInboxSummary.textContent = `${elements.loraInboxSummary.textContent} ${state.loraInbox.statusMessage}`;
  }

  elements.importLoraInboxButton.disabled =
    state.loraInbox.importing || selectedCount === 0 || !elements.loraInboxFamilySelect.value;
  elements.importLoraInboxButton.textContent = state.loraInbox.importing
    ? "Importing..."
    : `Import selected LoRAs${selectedCount ? ` (${selectedCount})` : ""}`;
  elements.clearLoraInboxSelectionButton.disabled =
    state.loraInbox.importing || selectedCount === 0;

  if (!state.loraInbox.assets.length) {
    elements.loraInboxList.innerHTML = `<div class="empty-state">No LoRA handoffs are waiting in this bridge lane.</div>`;
    return;
  }

  elements.loraInboxList.innerHTML = state.loraInbox.assets
    .map((asset) => {
      const selected = state.loraInbox.selectedAssetIds.has(asset.asset_id);
      return `
        <article class="lora-inbox-item ${selected ? "selected" : ""}">
          <label class="lora-inbox-select">
            <input type="checkbox" data-lora-inbox-asset="${escapeAttribute(asset.asset_id)}" ${selected ? "checked" : ""}>
            <span>${escapeHtml(asset.label || asset.file_name || asset.asset_id || "LoRA handoff")}</span>
          </label>
          <div class="lora-inbox-copy">
            <strong>${escapeHtml(asset.file_name || asset.payload_file_name || asset.asset_id)}</strong>
            <p>${escapeHtml(asset.summary || "LoRA file waiting for import.")}</p>
            <div class="lora-inbox-meta">${renderLoraInboxMeta(asset)}</div>
            <p class="lora-inbox-action-note">Import will copy this into <code>models/loras/${escapeHtml(elements.loraInboxFamilySelect.value || "family")}/</code>. The sending module keeps its original output.</p>
          </div>
        </article>
      `;
    })
    .join("");

  elements.loraInboxList.querySelectorAll("[data-lora-inbox-asset]").forEach((checkbox) => {
    checkbox.addEventListener("change", () => {
      const assetId = checkbox.dataset.loraInboxAsset;
      if (!assetId) {
        return;
      }
      if (checkbox.checked) {
        state.loraInbox.selectedAssetIds.add(assetId);
      } else {
        state.loraInbox.selectedAssetIds.delete(assetId);
      }
      state.loraInbox.statusMessage = "";
      renderLoraInboxPanel();
    });
  });
}

function renderLoraInboxMeta(asset) {
  const chips = [];
  const sender = String(asset.from_device_name || asset.from_device_id || "").trim();
  const laneLabel = String(asset.lane_label || asset.lane_id || "lora_imports").trim();
  const delivered = formatBridgeDeliveredTime(asset.delivered_at_unix_ms);
  const byteLabel = Number.isFinite(Number(asset.byte_len)) && Number(asset.byte_len) > 0
    ? `${formatOneDecimal(Number(asset.byte_len) / (1024 * 1024))} MB`
    : "";
  const contentType = String(asset.content_type || "").trim();

  if (sender) {
    chips.push(`From ${sender}`);
  }
  if (laneLabel) {
    chips.push(`Lane ${laneLabel}`);
  }
  if (delivered) {
    chips.push(`Delivered ${delivered}`);
  }
  if (byteLabel) {
    chips.push(byteLabel);
  }
  if (contentType) {
    chips.push(contentType);
  }

  return chips.length
    ? chips.map((chip) => `<span class="runtime-pill">${escapeHtml(chip)}</span>`).join("")
    : `<span class="runtime-pill">Bridge handoff</span>`;
}

function pruneLoraInboxSelection() {
  const validIds = new Set(state.loraInbox.assets.map((asset) => asset.asset_id));
  for (const assetId of [...state.loraInbox.selectedAssetIds]) {
    if (!validIds.has(assetId)) {
      state.loraInbox.selectedAssetIds.delete(assetId);
    }
  }
}

async function importSelectedLoraInboxAssets() {
  const assetIds = [...state.loraInbox.selectedAssetIds];
  const familyKey = elements.loraInboxFamilySelect.value;
  if (!assetIds.length || !familyKey) {
    renderLoraInboxPanel();
    return;
  }

  state.loraInbox.importing = true;
  state.loraInbox.statusMessage = "";
  renderLoraInboxPanel();

  try {
    const response = await fetch("/api/loras/import-bridge", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        lane_id: "lora_imports",
        asset_ids: assetIds,
        family_key: familyKey,
      }),
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || `LoRA import failed with ${response.status}`);
    }

    if (typeof window.chattyCogBridge?.consumeIncomingAsset === "function") {
      for (const assetId of assetIds) {
        window.chattyCogBridge.consumeIncomingAsset("lora_imports", assetId);
      }
    }

    state.loraInbox.selectedAssetIds.clear();
    state.loraInbox.statusMessage = Array.isArray(payload.notes) ? payload.notes.join(" ") : "LoRA import complete.";
    await loadLoras();
    await loadLoraInboxAssets({ silent: true });
  } catch (error) {
    state.loraInbox.statusMessage = `Could not import bridge LoRAs yet: ${String(error.message || error)}`;
  } finally {
    state.loraInbox.importing = false;
    renderLoraInboxPanel();
  }
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
  const toolingTone = backendStatus.tooling_ready ? "vulkan" : "warning";
  const toolingMarkup = backendStatus.tooling_label
    ? `<span class="runtime-pill runtime-${escapeHtml(toolingTone)}">${escapeHtml(backendStatus.tooling_label)}</span>`
    : "";
  const toolingNoteMarkup = backendStatus.tooling_note
    ? `<span class="runtime-note">${escapeHtml(backendStatus.tooling_note)}</span>`
    : "";
  elements.runtimeBadges.innerHTML = `
    <span class="runtime-pill">${escapeHtml(formatBackendBadge(backendStatus.backend))}</span>
    <span class="runtime-pill runtime-${escapeHtml(tone)}">${escapeHtml(backendStatus.label)}</span>
    <span class="runtime-note">${escapeHtml(backendStatus.note)}</span>
    ${toolingMarkup}
    ${toolingNoteMarkup}
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
  const cloudRoutes = getMediaGenerationCloudRoutes();
  const hiddenModeCount = state.models.length - visibleModels.length;

  if (!state.models.length && !cloudRoutes.length) {
    elements.modelSelect.innerHTML = `<option value="">No local or cloud models found</option>`;
    elements.modelSelect.disabled = true;
    renderStyleMode();
    renderModelNotice("Drop one or more GGUF models or supported local model packages into models/ and press Refresh Files.");
    renderPromptModelSelector();
    renderVisionModelSelector();
    renderPrepareKindOptions();
    renderPreparedHandoff();
    renderReferenceIntentControls();
    syncActionState();
    return;
  }

  if (!visibleModels.length && !cloudRoutes.length) {
    const label = state.generationStyle === "realism" ? "No realism models found" : "No expressive models found";
    elements.modelSelect.innerHTML = `<option value="">${label}</option>`;
    elements.modelSelect.disabled = true;
    renderStyleMode();
    renderModelNotice(
      state.generationStyle === "realism"
        ? "Realism mode needs diffusion-style GGUFs or supported local model packages, plus any companion weights they require in models/. Switch to Expressive to use regular llama.cpp models."
        : "Expressive mode uses regular llama.cpp-compatible models. Switch to Realism for diffusion/video GGUFs."
    );
    renderPromptModelSelector();
    renderVisionModelSelector();
    renderPrepareKindOptions();
    renderPreparedHandoff();
    renderReferenceIntentControls();
    syncActionState();
    return;
  }

  if (!supportedModels.length && !cloudRoutes.length) {
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
    renderPromptModelSelector();
    renderVisionModelSelector();
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
  const cloudOptions = cloudRoutes.length
    ? `<optgroup label="Cloud">${cloudRoutes
        .map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildMediaCloudDropdownLabel(model))}</option>`)
        .join("")}</optgroup>`
    : "";
  const unsupportedOptions = unsupportedModels.length
    ? `<optgroup label="Detected but not ready">${unsupportedModels
        .map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildDropdownLabel(model))}</option>`)
        .join("")}</optgroup>`
    : "";

  elements.modelSelect.innerHTML = `${supportedOptions ? `<optgroup label="Ready to run">${supportedOptions}</optgroup>` : ""}${cloudOptions}${unsupportedOptions}`;

  const activeCloudRoute = activeMediaGenerationCloudRoute();
  const activeCloudValue = activeCloudRoute ? activeCloudRoute.id : "";
  if (selected && (visibleModels.some((model) => model.id === selected) || cloudRoutes.some((model) => model.id === selected))) {
    elements.modelSelect.value = selected;
  } else if (activeCloudValue && cloudRoutes.some((model) => model.id === activeCloudValue)) {
    elements.modelSelect.value = activeCloudValue;
  } else if (supportedModels.length) {
    elements.modelSelect.value = supportedModels[0].id;
  } else if (cloudRoutes.length) {
    elements.modelSelect.value = cloudRoutes[0].id;
  } else {
    elements.modelSelect.value = unsupportedModels[0]?.id || "";
  }

  normalizeAssignedReferencesForCurrentModel();
  renderStyleMode();
  renderModelSummary(hiddenModeCount);
  renderPromptModelSelector();
  renderVisionModelSelector();
  renderPrepareKindOptions();
  renderPreparedHandoff();
  renderReferenceIntentControls();
  syncActionState();
}

function renderPrepareKindOptions() {
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const current = elements.prepareKindInput.value;
  if (activeMediaCloudProvider) {
    const supportedKinds = mediaGenerationSupportedKinds(activeMediaCloudProvider);
    elements.prepareKindInput.innerHTML = supportedKinds
      .map((kind) => `<option value="${escapeHtml(kind)}">${escapeHtml(formatKind(kind))}</option>`)
      .join("");
    elements.prepareKindInput.value = supportedKinds.includes(current)
      ? current
      : (supportedKinds[0] || "image");
    normalizeAssignedReferencesForCurrentModel();
    return;
  }

  const model = getSelectedModel();
  const fallbackKinds = state.generationStyle === "realism"
    ? ["image", "gif", "video"]
    : ["image", "gif", "audio"];
  const supportedKinds = model
    ? ((model.supported_kinds || []).length ? model.supported_kinds : fallbackKinds)
    : fallbackKinds;

  elements.prepareKindInput.innerHTML = supportedKinds
    .map((kind) => `<option value="${escapeHtml(kind)}">${escapeHtml(formatKind(kind))}</option>`)
    .join("");

  if (supportedKinds.includes(current)) {
    elements.prepareKindInput.value = current;
  } else {
    elements.prepareKindInput.value = supportedKinds[0] || "image";
  }
  normalizeAssignedReferencesForCurrentModel();
}

function renderModelSummary(hiddenModeCount = null) {
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  if (activeMediaCloudProvider) {
    const cloudRoute = activeMediaGenerationCloudRoute();
    if (cloudRoute) {
      elements.modelSummary.innerHTML = buildCloudGenerationSummary(
        cloudRoute,
        `Final output is currently routed through ${cloudRoute.providerName} using ${cloudRoute.modelName} for ${formatKind(cloudRoute.outputKind).toLowerCase()} generation. This selection reuses the saved cloud media lane instead of a local GGUF runtime.`
      );
      refreshAudioSettingCopy();
      return;
    }
    const kinds = mediaGenerationSupportedKinds(activeMediaCloudProvider)
      .map((kind) => formatKind(kind).toLowerCase())
      .join(" + ");
    renderModelNotice(
      `Cloud output route is active via ${activeMediaCloudProvider.display_name}. Local model selection is bypassed for final ${kinds || "media"} generation on this lane.`
    );
    return;
  }

  const model = getSelectedModel();
  if (!model) {
    const invisibleCount = hiddenModeCount ?? state.models.filter((entry) => entry.generation_style !== state.generationStyle).length;
    renderModelNotice(
      invisibleCount > 0
        ? `No compatible ${state.generationStyle} model selected. ${invisibleCount} file(s) are hidden from this generator picker right now.`
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
  const compatibilityNote = String(model.compatibility_note || "").toLowerCase();
  if (compatibilityNote.includes("clip_vision_h")) {
    badges.push(
      createModelBadge(
        model.requires_reference ? "Clip Vision needed" : "Clip Vision for I2V",
        "reference"
      )
    );
  }
  const selectedLoras = getSelectedLoraDetails(model);
  if (supportsLoraControl(model) && selectedLoras.length) {
    badges.push(
      createModelBadge(
        selectedLoras.length === 1
          ? `LoRA: ${selectedLoras[0].lora.name} @ ${selectedLoras[0].weight.toFixed(2)}`
          : `${selectedLoras.length} LoRAs stacked`,
        "reference"
      )
    );
  }

  const hiddenNote = invisibleCount > 0
    ? `<div class="model-summary-foot">${escapeHtml(`${invisibleCount} file(s) are hidden from this generator picker right now.`)}</div>`
    : "";
  const runtimeLine = buildExplicitRuntimeLine(model);
  const recommendations = buildRecommendedLimitsMarkup(model);

  elements.modelSummary.innerHTML = `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(model.name)}</strong>
      </div>
      <div class="model-badges">${badges.join("")}</div>
      ${runtimeLine ? `<div class="model-summary-runtime">${escapeHtml(runtimeLine)}</div>` : ""}
      <div class="model-summary-copy">${escapeHtml(model.compatibility_note)}</div>
      ${recommendations}
      ${hiddenNote}
    </div>
  `;
  refreshAudioSettingCopy();
}

function buildMediaCloudDropdownLabel(model) {
  const provider = providerKindLabel(model.providerKind);
  const kindLabel = formatKind(model.outputKind);
  return `${kindLabel} | ${model.modelName} | ${provider} | ${model.providerName} [Cloud Route]`;
}

function cloudRouteIdentity(model, laneLabel) {
  return `${laneLabel} | ${model.modelName} | ${providerKindLabel(model.providerKind)} | ${model.providerName}`;
}

function cloudGenerationRouteUsageDetails(model) {
  if (model.outputKind === "video") {
    if (model.providerKind === "open_ai") {
      return {
        bestFit: "short MP4 video clips on the saved OpenAI video route",
        watchFor: "duration buckets of 4, 8, 12, 16, or 20 seconds; 1080p is reserved for sora-2-pro; still-image guide only",
        note: "This route currently uses the deprecated OpenAI Videos API path and must be replaced before September 24, 2026.",
      };
    }
    if (model.providerKind === "gemini") {
      return {
        bestFit: "short MP4 video clips on the saved Gemini Veo route",
        watchFor: "duration buckets of 4, 6, or 8 seconds; still-image guide only; guide-image runs require 8 seconds",
        note: "This route currently uses Gemini's compatible video bridge and only surfaces the main still-image guide reference.",
      };
    }
    return {
      bestFit: "short MP4 video clips on the saved cloud video route",
      watchFor: "provider-specific size, duration, and still-image guide limits",
      note: "Cloud video currently accepts only the main still-image guide reference on supported routes.",
    };
  }

  if (model.outputKind === "audio") {
    return {
      bestFit: "speech-style audio output on the saved cloud route",
      watchFor: "provider-specific voice or speaker ids and speech pacing differences",
      note: "Cloud speech currently does not upload guide, end-frame, control-video, or voice-reference tray files.",
    };
  }

  return {
    bestFit: "final still-image output on the saved cloud route",
    watchFor: "provider-specific latency, moderation checks, and image-edit/reference limits",
    note: "Cloud image output currently accepts only still-image guide references on supported routes.",
  };
}

function buildCloudGenerationSummary(model, message) {
  const verificationStatus = model.verification?.status ? model.verification.status : "Not verified yet.";
  const providerBits = [
    providerKindLabel(model.providerKind),
    model.providerName,
    model.hasApiKey ? "API key saved" : "No API key saved",
  ];
  const routeLabel = formatKind(model.outputKind);
  const routeDetail = model.outputKind === "audio"
    ? `${model.modelName}${model.audioVoice ? ` | Voice: ${model.audioVoice}` : " | Provider default voice"}`
    : model.modelName;
  const usage = cloudGenerationRouteUsageDetails(model);
  return `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(model.modelName)}</strong>
      </div>
      <div class="model-badges">
        ${createModelBadge("Cloud", "backend")}
        ${createModelBadge(providerKindLabel(model.providerKind), "family")}
        ${createModelBadge(routeLabel, "outputs")}
      </div>
      <div class="model-summary-runtime">${escapeHtml(providerBits.join(" | "))}</div>
      <div class="model-summary-copy">${escapeHtml(message)}</div>
      <div class="recommended-limits">
        <div class="recommended-limits-head">
          <strong>Cloud Route Details</strong>
          <span>${escapeHtml(model.baseUrl || "Saved route endpoint")}</span>
        </div>
        <div class="recommended-limits-list">
          <div class="recommended-limit-row current-safe">
            <strong>${escapeHtml(`${routeLabel} Route`)}</strong>
            <span>${escapeHtml(`Provider account: ${model.providerName}`)}</span>
            <span>${escapeHtml(`Configured model: ${routeDetail}`)}</span>
            <span class="recommended-current current-safe"><em>Verification:</em> ${escapeHtml(verificationStatus)}</span>
            <span class="recommended-current-note">This route is route-aware rather than hardware-aware. Latency, queue time, moderation checks, and policy behavior can vary by endpoint family and output kind.</span>
          </div>
          <div class="recommended-limit-row current-stretch">
            <strong>Privacy</strong>
            <span>Prompts leave this machine only when this cloud route stays selected.</span>
            <span>Only references supported by the current cloud output kind can be uploaded.</span>
            <span class="recommended-current current-stretch"><em>Route:</em> ${escapeHtml(cloudRouteIdentity(model, routeLabel))}</span>
            <span class="recommended-current-note">Prompt Assist and Vision Assist remain separate role selectors and can still stay local or use their own cloud routes.</span>
          </div>
          <div class="recommended-limit-row current-risky">
            <strong>Usage shape</strong>
            <span><em>Best fit:</em> ${escapeHtml(usage.bestFit)}</span>
            <span><em>Watch for:</em> ${escapeHtml(usage.watchFor)}</span>
            <span class="recommended-current current-risky"><em>Route note:</em> ${escapeHtml(usage.note)}</span>
            <span class="recommended-current-note">This keeps the existing selector workflow, but makes the active cloud route's real constraints visible where hardware guidance would normally appear.</span>
          </div>
        </div>
        <div class="recommended-limits-note">This uses the same selector workflow as local models, but swaps hardware guidance for account, verification, privacy, and route-specific details.</div>
      </div>
    </div>
  `;
}

function buildExplicitRuntimeLine(model) {
  if (!model) {
    return "";
  }

  const runtime = state.runtimeStatus
    ? (model.generation_style === "realism" ? state.runtimeStatus.realism : state.runtimeStatus.expressive)
    : null;

  if (model.backend === "stable_diffusion_cpp") {
    if (!runtime) {
      return "Realism backend: stable-diffusion.cpp";
    }
    return `Realism backend: stable-diffusion.cpp ${runtime.label} build`;
  }

  if (model.backend === "audio_runtime") {
    return "Realism backend: specialist local audio runtime";
  }

  if (model.backend === "llama_cpp") {
    if (!runtime) {
      return "Expressive backend: llama.cpp local runtime";
    }
    return `Expressive backend: llama.cpp ${runtime.label} runtime`;
  }

  return "";
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
    case "video_landscape720":
      return [1280, 720];
    case "video_portrait720":
      return [720, 1280];
    case "video_landscape1024":
      return [1792, 1024];
    case "video_portrait1024":
      return [1024, 1792];
    case "video_landscape1080":
      return [1920, 1080];
    case "video_portrait1080":
      return [1080, 1920];
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
    : cloudPrimaryReferenceUsesGuideOnly()
      ? "No primary guide image assigned."
      : "No primary guide/edit input assigned.";
}

function primaryReferenceFilledDetail(model = getSelectedModel()) {
  return isSpeechVoiceReferenceModel(model)
    ? "Used to clone the speaker voice for realism speech generation. Short WAV clips work best; keep them under 20 seconds."
    : `${referenceIntentLabel(state.referenceIntent)}`;
}

function cloudPrimaryReferenceUsesGuideOnly() {
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudOutputKind = elements.prepareKindInput.value;
  return Boolean(
    activeMediaCloudProvider
    && (cloudOutputKind === "image" || cloudOutputKind === "video")
  );
}

function renderReferenceIntentControls() {
  const context = getReferenceAssignmentContext();
  const selectedAssetId = state.selectedReference?.id;
  const speechVoiceModel = isSpeechVoiceReferenceModel();
  const cloudGuideOnly = cloudPrimaryReferenceUsesGuideOnly();

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
  elements.referenceEdit.classList.toggle("hidden", speechVoiceModel || cloudGuideOnly);
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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudOutputKind = elements.prepareKindInput.value;

  if (!asset) {
    if (activeMediaCloudProvider && cloudOutputKind === "audio") {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: "Cloud speech output does not accept guide, end-frame, control-video, or voice-reference tray files, so nothing from the tray will be uploaded on this output lane.",
      };
    }

    if (activeMediaCloudProvider && cloudOutputKind === "image") {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: "Choose a still image first if you want to attach one guide image to the cloud image lane. End-frame and control-video references stay local-only.",
      };
    }

    if (activeMediaCloudProvider && cloudOutputKind === "video") {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: isGeminiCloudVideoProvider(activeMediaCloudProvider)
          ? "Choose a still image first if you want to attach one guide image to the Gemini Veo lane. Gemini requires an 8-second duration when a still-image guide is attached. End-frame and control-video references stay local-only."
          : "Choose a still image first if you want to attach one guide image to the cloud video lane. End-frame and control-video references stay local-only.",
      };
    }

    if (isSpeechVoiceReferenceModel(model)) {
      return {
        voiceEnabled: false,
        guideEnabled: false,
        editEnabled: false,
        endEnabled: false,
        controlEnabled: false,
        message: "Choose an audio file first. Voice Reference assigns a prerecorded clip for speech cloning. Short WAV clips work best, and OuteTTS expects 20 seconds or less.",
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

  if (activeMediaCloudProvider && cloudOutputKind === "audio") {
    return {
      voiceEnabled: false,
      guideEnabled: false,
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: "Cloud speech output does not use tray references yet, so this file will stay local unless you switch back to a compatible local lane.",
    };
  }

  if (activeMediaCloudProvider && cloudOutputKind === "image") {
    return {
      voiceEnabled: false,
      guideEnabled: asset.kind === "image",
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: asset.kind === "image"
        ? `Assign this still image as the single guide reference for cloud image output via ${activeMediaCloudProvider.display_name}. End-frame and control-video slots stay local-only.`
        : "Cloud image output only accepts a still-image guide from the tray. End-frame and control-video references stay local-only.",
    };
  }

  if (activeMediaCloudProvider && cloudOutputKind === "video") {
    return {
      voiceEnabled: false,
      guideEnabled: asset.kind === "image",
      editEnabled: false,
      endEnabled: false,
      controlEnabled: false,
      message: asset.kind === "image"
        ? isGeminiCloudVideoProvider(activeMediaCloudProvider)
          ? `Assign this still image as the single guide reference for Gemini Veo output via ${activeMediaCloudProvider.display_name}. Gemini requires an 8-second duration when a still-image guide is attached. End-frame and control-video slots stay local-only.`
          : `Assign this still image as the single guide reference for cloud video output via ${activeMediaCloudProvider.display_name}. End-frame and control-video slots stay local-only.`
        : "Cloud video output only accepts a still-image guide from the tray. End-frame and control-video references stay local-only.",
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
        ? "Assign this audio file as the voice reference for realism speech generation. Short WAV clips work best, and OuteTTS expects 20 seconds or less."
        : "Speech voice cloning uses an audio file from the tray as the voice reference. Short WAV clips work best, and OuteTTS expects 20 seconds or less.",
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
  const outputRouteLabel = formatOutputRouteLabel(item);
  const promptAssistLabel = formatLaneRouteLabel(item.prompt_assist_route);
  const visionAssistLabel = formatLaneRouteLabel(item.vision_assist_route);
  const outputSummaryLabel = summarizeOutputSurface(item);
  elements.previewSurface.innerHTML = `
    ${createMediaMarkup(item, "preview-media")}
    <div class="preview-meta">
      <h3>${escapeHtml(item.file_name)}</h3>
      <span>${escapeHtml(outputSummaryLabel)}</span>
      <p>${escapeHtml(item.prompt || "Saved output")}</p>
      ${item.resolution_label ? `<p><strong>Output settings:</strong> ${escapeHtml(item.resolution_label)}</p>` : ""}
      ${item.negative_prompt ? `<p><strong>Negative prompt:</strong> ${escapeHtml(item.negative_prompt)}</p>` : ""}
      ${item.reference_asset ? `<p><strong>${escapeHtml(item.kind === "audio" && item.spoken_text ? "Voice reference" : "Reference use")}:</strong> ${escapeHtml(item.kind === "audio" && item.spoken_text ? item.reference_asset : `${referenceIntentLabel(item.reference_intent || "guide")} via ${item.reference_asset}`)}</p>` : ""}
      ${item.end_reference_asset ? `<p><strong>End frame:</strong> ${escapeHtml(item.end_reference_asset)}</p>` : ""}
      ${item.control_reference_asset ? `<p><strong>Control video:</strong> ${escapeHtml(item.control_reference_asset)}</p>` : ""}
      ${outputRouteLabel ? `<p><strong>Output route:</strong> ${escapeHtml(outputRouteLabel)}</p>` : ""}
      ${item.spoken_text ? `<p><strong>Spoken text:</strong> ${escapeHtml(item.spoken_text)}</p>` : ""}
      ${item.compiled_prompt ? `<p><strong>${item.spoken_text ? "Speech direction:" : "Compiled brief:"}</strong> ${escapeHtml(item.compiled_prompt)}</p>` : ""}
      ${item.prompt_assist && item.prompt_assist !== "off" ? `<p><strong>Prompt Assist:</strong> ${escapeHtml(item.prompt_assist)}${item.interpreter_model ? ` via ${escapeHtml(item.interpreter_model)}` : ""}${promptAssistLabel ? ` (${escapeHtml(promptAssistLabel)})` : ""}</p>` : ""}
      ${item.vision_model ? `<p><strong>Vision Assist:</strong> ${escapeHtml(item.vision_model)}${visionAssistLabel ? ` (${escapeHtml(visionAssistLabel)})` : ""}</p>` : ""}
      <p>${escapeHtml(item.note || "")}</p>
    </div>
  `;
}

function renderHistory() {
  renderOutputHandoffPanel();

  if (!state.outputs.length) {
    elements.historyGrid.innerHTML = `<div class="history-card"><div class="history-copy"><strong>No saved outputs yet</strong><span>Generated files will appear here as soon as the first job finishes.</span></div></div>`;
    return;
  }

  elements.historyGrid.innerHTML = state.outputs
    .map((output) => {
      const preview = createHistoryPreview(output);
      const selected = state.selectedOutputIds.has(output.id);
      return `
        <div class="history-card ${selected ? "selected" : ""}">
          <div class="history-card-header">
            <label class="history-select">
              <input type="checkbox" data-output-select="${escapeHtml(output.id)}" ${selected ? "checked" : ""}>
              <span>Select</span>
            </label>
          </div>
          <button class="history-preview-button" type="button" data-output-id="${escapeHtml(output.id)}">
            ${preview}
            <div class="history-copy">
              <strong>${escapeHtml(output.file_name)}</strong>
              <span>${escapeHtml(buildHistoryCardSubtitle(output))}</span>
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

  elements.historyGrid.querySelectorAll("[data-output-select]").forEach((checkbox) => {
    checkbox.addEventListener("change", () => {
      toggleOutputSelection(checkbox.dataset.outputSelect, checkbox.checked);
    });
  });
}

function renderOutputHandoffPanel() {
  const sandboxTarget = getSandboxHandoffTarget();
  const loraTarget = getLoraHandoffTarget();
  const selectedOutputs = getSelectedOutputs();
  const selectedCount = selectedOutputs.length;

  elements.outputHandoffPanel.classList.toggle("hidden", !state.outputs.length);
  if (!state.outputs.length) {
    return;
  }

  let summary = "Select one or more saved outputs to share a copy through ChattyCog.";
  if (sandboxTarget?.supported) {
    summary = selectedCount
      ? `${selectedCount} output${selectedCount === 1 ? "" : "s"} selected. ChattyCog will copy them into the sandbox handoff tray and keep the originals here in Chatty-art.`
      : "Select one or more saved outputs to share a copy through ChattyCog.";
  } else if (sandboxTarget?.note) {
    summary = sandboxTarget.note;
  } else {
    summary = "ChattyCog Sandbox is not available in this hosting context yet.";
  }

  if (state.handoffStatusMessage) {
    summary = `${summary} ${state.handoffStatusMessage}`;
  }

  elements.outputHandoffSummary.textContent = summary;
  elements.sendOutputsToLoraButton.disabled = !loraTarget?.supported || !selectedCount || state.handoffSending;
  elements.sendOutputsToLoraButton.textContent = state.handoffSending
    ? "Sending to Chatty-lora..."
    : `Send Selected to Chatty-lora${selectedCount ? ` (${selectedCount})` : ""}`;
  elements.sendOutputsToSandboxButton.disabled = !sandboxTarget?.supported || !selectedCount || state.handoffSending;
  elements.sendOutputsToSandboxButton.textContent = state.handoffSending
    ? "Sending to ChattyCog Sandbox..."
    : `Send Selected to ChattyCog Sandbox${selectedCount ? ` (${selectedCount})` : ""}`;
  elements.deleteSelectedOutputsButton.disabled = !selectedCount || state.handoffSending;
  elements.deleteSelectedOutputsButton.textContent = state.handoffSending
    ? "Working..."
    : `Delete Selected${selectedCount ? ` (${selectedCount})` : ""}`;
  renderOutputHandoffPreview(selectedOutputs, sandboxTarget, loraTarget);
}

function renderOutputHandoffPreview(selectedOutputs, sandboxTarget, loraTarget) {
  if (!elements.outputHandoffPreview) {
    return;
  }

  if (!selectedOutputs.length) {
    elements.outputHandoffPreview.innerHTML = `<div class="empty-state compact">Select one or more outputs to preview the mediated handoff details.</div>`;
    return;
  }

  const first = selectedOutputs[0];
  const note = String(elements.outputHandoffNote?.value || "").trim();
  const tags = buildOutputHandoffTags(first).slice(0, 6);
  const readyTargets = [loraTarget, sandboxTarget].filter((target) => target?.supported);
  const targetText = readyTargets.length
    ? readyTargets.map((target) => escapeHtml(`${target.label} via ${target.kind === "module" ? "dataset_candidates" : "sandbox"}`)).join(" or ")
    : "No approved handoff targets are available right now.";
  const previewLines = [
    `<strong>${escapeHtml(`${selectedOutputs.length} output${selectedOutputs.length === 1 ? "" : "s"} queued for mediated copy handoff`)}</strong>`,
    `<p>${escapeHtml(hasBridgeRouteSummary(readyTargets.length))}</p>`,
    `<div class="output-handoff-meta">
      <span class="runtime-pill">Source chatty_art</span>
      <span class="runtime-pill">${escapeHtml(`${selectedOutputs.length} file${selectedOutputs.length === 1 ? "" : "s"}`)}</span>
      <span class="runtime-pill">${escapeHtml(first.kind || "media output")}</span>
      ${first.model ? `<span class="runtime-pill">${escapeHtml(first.model)}</span>` : ""}
    </div>`,
    `<p>${escapeHtml(`Available route: ${readyTargets.length ? readyTargets.map((target) => target.label).join(" or ") : "none"}.`)}</p>`,
    `<p>${escapeHtml(`First item summary: ${buildOutputHandoffSummary(first)}`)}</p>`,
    tags.length ? `<div class="output-handoff-meta">${tags.map((tag) => `<span class="runtime-pill">${escapeHtml(tag)}</span>`).join("")}</div>` : "",
    note ? `<p>${escapeHtml(`User note to include: ${note}`)}</p>` : `<p>No extra user note will be attached unless you add one above.</p>`,
    `<p>${targetText}</p>`,
  ].filter(Boolean);

  elements.outputHandoffPreview.innerHTML = previewLines.join("");
}

function hasBridgeRouteSummary(readyTargetCount) {
  if (readyTargetCount > 0) {
    return "ChattyCog will keep the originals in Chatty-art, attach lightweight context metadata, and route copies only to approved targets.";
  }
  return "Selection also powers local cleanup here. You can still delete saved outputs even when no ChattyCog handoff targets are available.";
}

function toggleOutputSelection(outputId, checked) {
  if (!outputId) {
    return;
  }

  if (checked) {
    state.selectedOutputIds.add(outputId);
  } else {
    state.selectedOutputIds.delete(outputId);
  }

  state.handoffStatusMessage = "";
  renderHistory();
}

function pruneSelectedOutputs() {
  const validIds = new Set(state.outputs.map((output) => output.id));
  for (const outputId of [...state.selectedOutputIds]) {
    if (!validIds.has(outputId)) {
      state.selectedOutputIds.delete(outputId);
    }
  }
}

function getSandboxHandoffTarget() {
  return state.handoffTargets.find((target) => target.target_id === "chattycog_sandbox") || null;
}

function getLoraHandoffTarget() {
  return state.handoffTargets.find((target) => target.target_id === "chatty_lora") || null;
}

function getSelectedOutputs() {
  return state.outputs.filter((output) => state.selectedOutputIds.has(output.id));
}

function buildOutputHandoffSummary(output) {
  const parts = [
    summarizeOutputSurface(output),
    output.spoken_text ? `Spoken: ${truncateBridgeText(output.spoken_text, 90)}` : "",
    output.prompt ? `Prompt: ${truncateBridgeText(output.prompt, 140)}` : "",
    output.model ? `Model: ${output.model}` : "",
    output.relative_path ? `Path: ${output.relative_path}` : "",
  ].filter(Boolean);
  return parts.join(" | ") || "Saved Chatty-art output.";
}

function buildOutputHandoffTags(output) {
  const tags = ["chatty-art", "generated-output"];
  if (output.kind) {
    tags.push(`kind:${String(output.kind).toLowerCase()}`);
  }
  if (output.output_route) {
    tags.push(`output-route:${String(output.output_route).toLowerCase()}`);
  }
  if (output.prompt_assist_route) {
    tags.push(`prompt-assist-route:${String(output.prompt_assist_route).toLowerCase()}`);
  }
  if (output.vision_assist_route) {
    tags.push(`vision-assist-route:${String(output.vision_assist_route).toLowerCase()}`);
  }
  if (output.kind === "audio" && output.spoken_text) {
    tags.push("audio:speech");
  }
  if (output.model) {
    const lower = String(output.model).toLowerCase();
    if (lower.includes("wan")) tags.push("family:wan");
    if (lower.includes("qwen")) tags.push("family:qwen");
    if (lower.includes("flux")) tags.push("family:flux");
    if (lower.includes("sd3")) tags.push("family:sd3");
  }
  return [...new Set(tags)];
}

async function sendSelectedOutputsToLora() {
  return sendSelectedOutputsToTarget({
    target: getLoraHandoffTarget(),
    kind: "module",
    targetId: "chatty_lora",
    laneId: "dataset_candidates",
    buttonLabel: "Chatty-lora",
    successLabel: "Chatty-lora",
  });
}

async function sendSelectedOutputsToSandbox() {
  return sendSelectedOutputsToTarget({
    target: getSandboxHandoffTarget(),
    kind: "sandbox",
    targetId: "chattycog_sandbox",
    laneId: "sandbox_export",
    buttonLabel: "ChattyCog Sandbox",
    successLabel: "ChattyCog Sandbox",
  });
}

async function sendSelectedOutputsToTarget({ target, kind, targetId, laneId, buttonLabel, successLabel }) {
  const selectedOutputs = getSelectedOutputs();
  if (!target?.supported || !selectedOutputs.length) {
    renderOutputHandoffPanel();
    return;
  }

  if (!window.chattyCogBridge?.available || typeof window.chattyCogBridge.requestHandoff !== "function") {
    state.handoffStatusMessage = "ChattyCog handoff bridge is unavailable right now.";
    renderOutputHandoffPanel();
    return;
  }

  const confirmed = window.confirm(
    `Send ${selectedOutputs.length} selected output${selectedOutputs.length === 1 ? "" : "s"} to ${buttonLabel}? This will copy them and keep the originals in Chatty-art.`
  );
  if (!confirmed) {
    return;
  }

  const payload = {
    source_module_id: "chatty_art",
    destination: {
      kind,
      target_id: targetId,
      lane_id: laneId,
    },
    artifacts: selectedOutputs.map((output) => buildOutputHandoffArtifact(output)),
    user_note: elements.outputHandoffNote.value.trim(),
  };

  state.handoffSending = true;
  state.handoffStatusMessage = "";
  renderOutputHandoffPanel();

  try {
    const accepted = window.chattyCogBridge.requestHandoff(payload);
    if (!accepted) {
      throw new Error("ChattyCog did not accept the handoff request.");
    }

    state.handoffStatusMessage = `Sent ${selectedOutputs.length} output${selectedOutputs.length === 1 ? "" : "s"} to ${successLabel}.`;
    state.selectedOutputIds.clear();
    elements.outputHandoffNote.value = "";
    renderHistory();
  } catch (error) {
    state.handoffStatusMessage = error?.message || "ChattyCog handoff request failed.";
    renderOutputHandoffPanel();
  } finally {
    state.handoffSending = false;
    renderOutputHandoffPanel();
  }
}

async function deleteSelectedOutputs() {
  const selectedOutputs = getSelectedOutputs();
  if (!selectedOutputs.length) {
    renderOutputHandoffPanel();
    return;
  }

  const confirmed = window.confirm(
    `Delete ${selectedOutputs.length} selected output${selectedOutputs.length === 1 ? "" : "s"} from Chatty-art? This removes the saved files from outputs/ and also deletes any metadata sidecars tied to them.`
  );
  if (!confirmed) {
    return;
  }

  state.handoffSending = true;
  state.handoffStatusMessage = "";
  renderOutputHandoffPanel();

  try {
    const response = await fetch("/api/outputs/delete", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        relative_paths: selectedOutputs.map((output) => output.relative_path),
      }),
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || `Delete failed with ${response.status}`);
    }
    state.selectedOutputIds.clear();
    state.handoffStatusMessage = Array.isArray(payload.notes) ? payload.notes.join(" ") : "Selected outputs deleted.";
    await loadOutputs();
    await loadAssets();
  } catch (error) {
    state.handoffStatusMessage = error?.message || "Could not delete selected outputs yet.";
  } finally {
    state.handoffSending = false;
    renderHistory();
  }
}

function buildOutputHandoffArtifact(output) {
  const outputRelativePath = normalizeOutputBridgePath(output?.relative_path);
  return {
    artifact_id: output.id || output.file_name || output.relative_path || "",
    label: output.file_name || output.id || "Chatty-art output",
    artifact_kind: "module_asset_file",
    media_kind: output.kind || "",
    source_relative_path: outputRelativePath,
    summary: summarizeOutputForHandoff(output),
    tags: buildOutputHandoffTags(output),
  };
}

function normalizeOutputBridgePath(relativePath) {
  const trimmed = String(relativePath || "").trim().replace(/\\/g, "/");
  if (!trimmed) {
    return "";
  }
  if (trimmed.startsWith("outputs/")) {
    return trimmed;
  }
  return `outputs/${trimmed.replace(/^\/+/, "")}`;
}

function summarizeOutputForHandoff(output) {
  const parts = [
    summarizeOutputSurface(output),
    output.spoken_text ? `Spoken: ${truncateBridgeText(output.spoken_text, 100)}` : "",
    output.prompt ? truncateBridgeText(output.prompt, 180) : "",
    output.model ? `Model: ${output.model}` : "",
    output.style ? `Mode: ${output.style}` : "",
    output.resolution_label ? `Settings: ${output.resolution_label}` : "",
  ].filter(Boolean);
  return parts.join(" | ");
}

function buildHistoryCardSubtitle(output) {
  const route = formatOutputRouteLabel(output);
  const kind = formatKind(output.kind);
  const model = output.model || "Unknown model";
  if (output.kind === "audio" && output.spoken_text) {
    return `${kind} speech | ${route || "Saved output"} | ${model}`;
  }
  return `${kind} | ${route || "Saved output"} | ${model}`;
}

function summarizeOutputSurface(output) {
  const kind = formatKind(output.kind);
  const route = formatOutputRouteLabel(output);
  const style = output.style || "expressive";
  const model = output.model || "Unknown model";
  if (output.kind === "audio" && output.spoken_text) {
    return `${kind} speech | ${model} | ${route || `${style} mode`}`;
  }
  return `${kind} | ${model} | ${route || `${style} mode`}`;
}

function formatLaneRouteLabel(route) {
  if (route === "cloud") return "Cloud lane";
  if (route === "local") return "Local lane";
  return "";
}

function formatOutputRouteLabel(output) {
  if (!output?.output_route) {
    return "";
  }
  if (output.output_route === "cloud") {
    return output.kind === "audio" && output.spoken_text
      ? "Cloud speech lane"
      : "Cloud media lane";
  }
  if (output.output_route === "local") {
    if (output.backend === "audio_runtime") {
      return output.spoken_text ? "Local speech lane" : "Local audio lane";
    }
    return "Local lane";
  }
  return String(output.output_route);
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
  const context = getReferenceAssignmentContext();
  const slotAllowed = slot === "primary"
    ? (intent === "edit" ? context.editEnabled : (context.guideEnabled || context.voiceEnabled))
    : slot === "end"
      ? context.endEnabled
      : context.controlEnabled;
  if (!slotAllowed) {
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
  renderVisionModelSelector();
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
  renderVisionModelSelector();
  renderTrayPreview();
  syncActionState();
}

function clearReferenceSlots() {
  state.primaryReference = null;
  state.endReference = null;
  state.controlReference = null;
  clearPreparedHandoff();
  renderReferenceIntentControls();
  renderVisionModelSelector();
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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudOutputKind = elements.prepareKindInput.value;

  if (activeMediaCloudProvider) {
    if (cloudOutputKind === "audio") {
      state.referenceIntent = "guide";
      state.primaryReference = null;
      state.endReference = null;
      state.controlReference = null;
      return;
    }

    if (cloudOutputKind === "image") {
      state.referenceIntent = "guide";
      if (state.primaryReference && state.primaryReference.kind !== "image") {
        state.primaryReference = null;
      }
      state.endReference = null;
      state.controlReference = null;
      return;
    }

    if (cloudOutputKind === "video") {
      state.referenceIntent = "guide";
      if (state.primaryReference && state.primaryReference.kind !== "image") {
        state.primaryReference = null;
      }
      state.endReference = null;
      state.controlReference = null;
      return;
    }
  }

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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const assignedReferencesValid = activeMediaCloudProvider
    ? areCloudMediaReferencesCompatible()
    : areAssignedReferencesCompatible(model);
  const prepareKind = elements.prepareKindInput.value;
  const cloudKinds = activeMediaCloudProvider ? mediaGenerationSupportedKinds(activeMediaCloudProvider) : [];
  const prepareSupported = activeMediaCloudProvider
    ? cloudKinds.includes(prepareKind)
    : model && model.runtime_supported && kindSupported(model, prepareKind);
  elements.prepareRequestButton.disabled = state.preparing || state.generating || !prepareSupported || !assignedReferencesValid;
  elements.clearPreparedButton.disabled = !state.preparedHandoff;
  elements.actionButtons.forEach((button) => {
    const supported = activeMediaCloudProvider
      ? cloudKinds.includes(button.dataset.kind)
      : model && model.runtime_supported && kindSupported(model, button.dataset.kind);
    button.disabled = state.generating || state.preparing || !supported || !assignedReferencesValid;
  });
  elements.cancelGenerateButton.disabled = !state.generating || !state.currentJobId || state.canceling;
}

function areCloudMediaReferencesCompatible() {
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudAudio = elements.prepareKindInput.value === "audio";
  if (cloudAudio) {
    return !state.primaryReference && !state.endReference && !state.controlReference;
  }
  if (state.endReference) {
    return false;
  }
  if (state.controlReference) {
    return false;
  }
  if (state.primaryReference && state.primaryReference.kind !== "image") {
    return false;
  }
  if (elements.prepareKindInput.value === "video" && isGeminiCloudVideoProvider(activeMediaCloudProvider)) {
    const requestedSeconds = Number(elements.videoDurationInput.value || 0);
    if (![4, 6, 8].includes(requestedSeconds)) {
      return false;
    }
    if (state.primaryReference && requestedSeconds !== 8) {
      return false;
    }
  }
  return true;
}

function cloudMediaReferenceValidationMessage() {
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const cloudAudio = elements.prepareKindInput.value === "audio";
  const cloudVideo = elements.prepareKindInput.value === "video";
  if (cloudAudio) {
    if (state.primaryReference) {
      return "Cloud speech generation does not support guide-image or voice-reference uploads yet.";
    }
    if (state.endReference) {
      return "Cloud speech generation does not support end-frame references.";
    }
    if (state.controlReference) {
      return "Cloud speech generation does not support control-video references.";
    }
    return "";
  }
  if (state.endReference) {
    return cloudVideo
      ? "Cloud video generation does not support end-frame references yet."
      : "Cloud image generation does not support end-frame references yet.";
  }
  if (state.controlReference) {
    return cloudVideo
      ? "Cloud video generation does not support control-video references yet."
      : "Cloud image generation does not support control-video references yet.";
  }
  if (state.primaryReference && state.primaryReference.kind !== "image") {
    return cloudVideo
      ? "Cloud video generation currently only supports one still-image guide reference."
      : "Cloud image generation currently only supports still-image guide references.";
  }
  if (cloudVideo) {
    const requestedSeconds = Number(elements.videoDurationInput.value || 0);
    if (isGeminiCloudVideoProvider(activeMediaCloudProvider) && ![4, 6, 8].includes(requestedSeconds)) {
      return "Gemini cloud video requires a duration of 4, 6, or 8 seconds.";
    }
    if (isGeminiCloudVideoProvider(activeMediaCloudProvider) && state.primaryReference && requestedSeconds !== 8) {
      return "Gemini cloud video requires an 8-second duration when using a still-image guide reference.";
    }
    const [width, height] = parseDimensionPair(elements.videoResolutionInput.value);
    const current = `${width}x${height}`;
    const allowed = ["1280x720", "720x1280", "1792x1024", "1024x1792", "1920x1080", "1080x1920"];
    if (!allowed.includes(current)) {
      return "Cloud video generation currently expects one of these video sizes: 1280x720, 720x1280, 1792x1024, 1024x1792, 1920x1080, or 1080x1920.";
    }
    if (isOpenAiCloudVideoProvider(activeMediaCloudProvider) && ![4, 8, 12, 16, 20].includes(requestedSeconds)) {
      return "OpenAI cloud video currently expects a duration of 4, 8, 12, 16, or 20 seconds.";
    }
    if (isOpenAiCloudVideoProvider(activeMediaCloudProvider) && !isOpenAiProVideoModel(activeMediaCloudProvider) && ["1920x1080", "1080x1920"].includes(current)) {
      return "OpenAI cloud video reserves 1920x1080 and 1080x1920 for sora-2-pro. Use 1280x720, 720x1280, 1792x1024, or 1024x1792 on sora-2.";
    }
  }
  return "";
}

function buildAcceptedMessage(model, kind, batchTotal = 1) {
  const assistMode = elements.promptAssistInput.value;
  const usingPreparedHandoff = state.preparedHandoff && state.preparedHandoff.kind === kind;
  const activeCloudProvider = activePromptAssistCloudProvider();
  const activeVisionCloudProvider = activeVisionAssistCloudProvider();
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const kindLabel = formatKind(kind).toLowerCase();
  const assistNote = usingPreparedHandoff
    ? " Preview Handoff was locked in for this run."
    : assistMode === "off"
    ? ""
    : activeCloudProvider
    ? ` Prompt Assist (${assistMode}) will compile a richer cloud brief first via ${activeCloudProvider.display_name}.`
    : ` Prompt Assist (${assistMode}) will compile a richer local brief first.`;
  const visionNote = activeVisionCloudProvider && state.primaryReference?.kind === "image" && assistMode !== "off"
    ? ` Vision Assist will inspect the reference image through ${activeVisionCloudProvider.display_name}.`
    : "";
  const mediaNote = activeMediaCloudProvider
    ? ` Final ${kindLabel} output will be generated through ${activeMediaCloudProvider.display_name}.`
    : "";
  const batchNote = batchTotal > 1
    ? ` Sequential batch mode will run ${batchTotal} end-to-end generations with a fresh random seed each time.`
    : "";

  if (activeMediaCloudProvider) {
    const videoRouteNote = kind === "video"
      ? mediaGenerationVideoRouteNote(activeMediaCloudProvider)
      : "";
    return `Job accepted. Starting cloud ${kind === "audio" ? "speech" : kindLabel} generation.${batchNote}${assistNote}${visionNote}${mediaNote}${videoRouteNote}`;
  }

  if (state.generationStyle === "realism") {
    return `Job accepted. Starting the local realism pipeline for ${kindLabel}. The first realism run can take longer while stable-diffusion.cpp gets ready.${batchNote}${assistNote}${visionNote}`;
  }

  const largePlanner =
    /\b(14b|20b|22b|32b|70b)\b/i.test(model.name) || /\b(gpt-oss|qwq)\b/i.test(model.name);
  if (largePlanner) {
    return `Job accepted. Starting local planning with ${model.name} for ${kindLabel}. Bigger GGUFs can spend a few minutes planning before rendering begins.${batchNote}${assistNote}${visionNote}`;
  }

  return `Job accepted. Starting local planning for ${kindLabel}. The first progress update may take a few seconds.${batchNote}${assistNote}${visionNote}`;
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
    elements.preparedVisionBlock.classList.add("hidden");
    elements.preparedVisionTitle.textContent = "Vision Summary";
    elements.preparedVisionSummary.textContent = "";
    elements.preparedFocusTags.innerHTML = "";
    elements.preparedAssumptions.textContent = "";
    return;
  }

  const isSpeechAudio = handoff.kind === "audio" && handoff.supports_voice_output;
  const isSoundAudio = handoff.kind === "audio" && !handoff.supports_voice_output;
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
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
  const hasVisionDetails = Boolean(handoff.vision_summary || handoff.vision_error || handoff.vision_model);
  elements.preparedVisionBlock.classList.toggle("hidden", !hasVisionDetails);
  elements.preparedVisionTitle.textContent = handoff.vision_model
    ? `Vision Summary via ${handoff.vision_model}`
    : "Vision Summary";
  elements.preparedVisionSummary.textContent = handoff.vision_summary
    || handoff.vision_error
    || "";

  const chips = [
    createPreparedChip(`For ${formatKind(handoff.kind)}`),
    createPreparedChip(handoff.resolution_label || "Current settings"),
    createPreparedChip(`Estimate ${formatDurationRange(handoff.estimated_time)}`),
    activeMediaCloudProvider ? createPreparedChip("Cloud Output Route") : createPreparedChip("Local Output Route"),
    handoff.interpreter_model && handoff.interpreter_model.includes("(cloud:")
      ? createPreparedChip("Cloud Prompt Assist")
      : handoff.prompt_assist && handoff.prompt_assist !== "off"
        ? createPreparedChip("Local Prompt Assist")
        : "",
    handoff.interpreter_model ? createPreparedChip(`Interpreter ${handoff.interpreter_model}`) : "",
    handoff.vision_model && handoff.vision_model.includes("(cloud:")
      ? createPreparedChip("Cloud Vision Assist")
      : handoff.vision_model
        ? createPreparedChip("Local Vision Assist")
        : "",
    handoff.vision_model ? createPreparedChip(`Vision Assist ${handoff.vision_model}`) : "",
    ...(Array.isArray(handoff.selected_lora_labels) && handoff.selected_lora_labels.length
      ? handoff.selected_lora_labels.map((label) => createPreparedChip(`LoRA ${label}`))
      : handoff.selected_lora_name
        ? [createPreparedChip(`LoRA ${handoff.selected_lora_name} @ ${Number(handoff.selected_lora_weight || 1).toFixed(2)}`)]
        : []),
    handoff.used_original_prompt ? createPreparedChip("Using original wording") : "",
    isSpeechAudio ? createPreparedChip("Speech handoff") : "",
    isSoundAudio ? createPreparedChip("Literal sound lane kept separate") : "",
  ].filter(Boolean);
  elements.preparedMetaChips.innerHTML = chips.join("");

  const noteParts = [
    handoff.note,
    activeMediaCloudProvider ? `Final output route: ${activeMediaCloudProvider.display_name}.` : "",
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

function formatBridgeDeliveredTime(unixMs) {
  const value = Number(unixMs);
  if (!Number.isFinite(value) || value <= 0) {
    return "";
  }
  return new Date(value).toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const prompt = elements.promptInput.value.trim();
  const negativePrompt = elements.negativePromptInput.value.trim();
  const audioLiteralPrompt = elements.audioLiteralPromptInput.value.trim();
  const batchCount = parseBatchCountInput();
  const includeAudioLiteral =
    kind === "audio"
    && (activeMediaCloudProvider || (model && model.backend === "audio_runtime"));
  const audioSegments = includeAudioLiteral && state.workflowMode === "advanced"
    ? getNormalizedAudioSegments()
    : [];
  const includeManualPromptControls = supportsManualPromptAssistInputs(model);
  const selectedLoras = supportsLoraControl(model) ? getNormalizedLoraSelections() : [];
  const includeLora = selectedLoras.length > 0;
  return {
    prompt,
    negative_prompt: negativePrompt ? negativePrompt : null,
    batch_count: batchCount,
    audio_literal_prompt:
      includeAudioLiteral && !audioSegments.length && audioLiteralPrompt ? audioLiteralPrompt : null,
    audio_segments: audioSegments,
    manual_focus_tags: includeManualPromptControls ? parsePromptListInput(elements.manualFocusCuesInput.value) : [],
    manual_assumptions: includeManualPromptControls ? parsePromptListInput(elements.manualAssumptionsInput.value) : [],
    manual_preserve_items: includeManualPromptControls ? parsePromptListInput(elements.manualPreserveInput.value) : [],
    manual_change_targets: includeManualPromptControls ? parsePromptListInput(elements.manualChangeInput.value) : [],
    manual_avoid_items: includeManualPromptControls ? parsePromptListInput(elements.manualAvoidInput.value) : [],
    prompt_assist: elements.promptAssistInput.value,
    model: activeMediaCloudProvider ? `cloud:${activeMediaCloudProvider.id}` : model.id,
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
    selected_prompt_model: selectedPromptAssistLocalModelId(),
    selected_vision_model: selectedVisionAssistLocalModelId(),
    selected_lora: includeLora ? selectedLoras[0].id : null,
    selected_lora_weight: includeLora ? selectedLoras[0].weight : null,
    selected_loras: includeLora ? selectedLoras : [],
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

function parseBatchCountInput() {
  const numeric = Number(elements.batchCountInput.value);
  if (!Number.isFinite(numeric)) {
    return 1;
  }
  return Math.min(64, Math.max(1, Math.round(numeric)));
}

function refreshBatchCountCopy() {
  const batchCount = parseBatchCountInput();
  const model = getSelectedModel();
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const kind = elements.prepareKindInput.value || "image";
  const kindLabel = formatKind(kind).toLowerCase();
  const selectedLoras = getNormalizedLoraSelections().length;
  const loraNote = selectedLoras > 0
    ? ` The current LoRA stack will be reused on every run.`
    : "";

  if (batchCount <= 1) {
    elements.batchCountCopy.textContent = `A count of 1 behaves like a normal single ${kindLabel} run.${loraNote}`;
    return;
  }

  const modelNote = activeMediaCloudProvider
    ? ` through ${activeMediaCloudProvider.display_name}`
    : model
      ? ` using ${model.name}`
      : "";
  elements.batchCountCopy.textContent = `This will run ${batchCount} ${kindLabel} generation${batchCount === 1 ? "" : "s"} one after another${modelNote}. Prompt, settings, references, and LoRA stack stay the same. Each run gets a fresh random seed, like clearing the seed box and pressing Generate again.${loraNote}`;
}

async function prepareGenerationRequest() {
  const model = getSelectedModel();
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  const kind = elements.prepareKindInput.value;

  if (!model && !activeMediaCloudProvider) {
    setProgress(0, "Model", `Choose a ${state.generationStyle} model first.`);
    return;
  }

  if (activeMediaCloudProvider && !mediaGenerationSupportedKinds(activeMediaCloudProvider).includes(kind)) {
    setProgress(0, "Output Route", `${activeMediaCloudProvider.display_name} is not configured for cloud ${formatKind(kind).toLowerCase()} generation.`);
    return;
  }

  if (!activeMediaCloudProvider && !kindSupported(model, kind)) {
    setProgress(0, "Mode", `${model.name} does not currently support ${kind} generation in ${state.generationStyle} mode.`);
    return;
  }

  if (activeMediaCloudProvider && !areCloudMediaReferencesCompatible()) {
    setProgress(0, "Reference", cloudMediaReferenceValidationMessage());
    return;
  }

  if (!activeMediaCloudProvider && !areAssignedReferencesCompatible(model)) {
    setProgress(0, "Reference", getAssignedReferenceValidationMessage(model));
    return;
  }

  const prompt = elements.promptInput.value.trim();
  const audioLiteralPrompt = elements.audioLiteralPromptInput.value.trim();
  const canUseAudioLiteral = kind === "audio" && (activeMediaCloudProvider || model?.backend === "audio_runtime");
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
  const activeCloudProvider = activePromptAssistCloudProvider();
  const activeVisionCloudProvider = activeVisionAssistCloudProvider();

  state.preparing = true;
  syncActionState();
  setProgress(
    0.06,
    "Previewing",
    [
      `Preparing a handoff preview for ${formatKind(kind).toLowerCase()} generation.`,
      activeMediaCloudProvider
        ? `Final ${formatKind(kind).toLowerCase()} output is routed through ${activeMediaCloudProvider.display_name}.`
        : "",
      activeCloudProvider
        ? `Prompt Assist will use cloud provider ${activeCloudProvider.display_name} if compilation is needed.`
        : "",
      activeVisionCloudProvider && state.primaryReference?.kind === "image"
        ? `Vision Assist will inspect the reference image through ${activeVisionCloudProvider.display_name} if image analysis is needed.`
        : "",
    ].filter(Boolean).join(" ")
  );

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
  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  if (!model && !activeMediaCloudProvider) {
    setProgress(0, "Model", `Choose a ${state.generationStyle} model first.`);
    return;
  }

  if (activeMediaCloudProvider && !mediaGenerationSupportedKinds(activeMediaCloudProvider).includes(kind)) {
    setProgress(0, "Output Route", `${activeMediaCloudProvider.display_name} is not configured for cloud ${formatKind(kind).toLowerCase()} generation.`);
    return;
  }

  if (!activeMediaCloudProvider && !kindSupported(model, kind)) {
    setProgress(0, "Mode", `${model.name} does not currently support ${kind} generation in ${state.generationStyle} mode.`);
    return;
  }

  if (activeMediaCloudProvider && !areCloudMediaReferencesCompatible()) {
    const message = cloudMediaReferenceValidationMessage();
    setProgress(0, "Reference", message);
    return;
  }

  if (!activeMediaCloudProvider && !areAssignedReferencesCompatible(model)) {
    const message = getAssignedReferenceValidationMessage(model);
    setProgress(0, "Reference", message);
    return;
  }

  const prompt = elements.promptInput.value.trim();
  const audioLiteralPrompt = elements.audioLiteralPromptInput.value.trim();
  const batchCount = parseBatchCountInput();
  const canUseAudioLiteral = kind === "audio" && (activeMediaCloudProvider || model?.backend === "audio_runtime");
  const audioSegments = canUseAudioLiteral && state.workflowMode === "advanced"
    ? getNormalizedAudioSegments()
    : [];
  if (!prompt && !(canUseAudioLiteral && (audioLiteralPrompt || audioSegments.length))) {
    setProgress(0, "Prompt", "Type a prompt or fill in the audio Words / Script / Sounds area first.");
    elements.promptInput.focus();
    return;
  }

  const currentRisk = !activeMediaCloudProvider && state.hardwareProfile
    ? assessCurrentKindPressure(model, kind, state.hardwareProfile)
    : null;

  state.generating = true;
  state.canceling = false;
  syncActionState();
  setProgress(
    0.04,
    "Queued",
    activeMediaCloudProvider
      ? `Submitting ${kind} job to cloud ${kind === "audio" ? "speech" : kind} generation via ${activeMediaCloudProvider.display_name}.`
      : state.generationStyle === "realism"
      ? `Submitting ${kind} job to the local stable-diffusion.cpp realism backend.${currentRisk?.tone === "risky" ? ` Warning: ${currentRisk.note}` : ""}`
      : `Submitting ${kind} job to the bundled expressive backend.`
  );

  let seed = null;
  if (batchCount === 1) {
    try {
      seed = parseSeedInput();
    } catch (error) {
      state.generating = false;
      syncActionState();
      setProgress(0, "Seed", error.message);
      elements.seedInput.focus();
      return;
    }
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
    state.currentBatchTotal = Number(accepted.batch_total || batchCount || 1);
    state.currentBatchCompleted = 0;
    if (accepted.used_seed !== undefined && accepted.used_seed !== null) {
      elements.seedInput.value = String(accepted.used_seed);
    }
    syncActionState();
    setProgress(0.08, "Starting", buildAcceptedMessage(model, kind, state.currentBatchTotal));
  } catch (error) {
    state.generating = false;
    state.canceling = false;
    state.currentJobId = null;
    state.currentBatchTotal = 1;
    state.currentBatchCompleted = 0;
    syncActionState();
    setProgress(0, "Error", error.message || "Generation request failed.");
  }
}

async function cancelCurrentGeneration() {
  if (!state.currentJobId || !state.generating || state.canceling) {
    return;
  }

  const activeMediaCloudProvider = activeMediaGenerationCloudProvider();
  state.canceling = true;
  syncActionState();
  setProgress(
    Math.max(0.05, Number(state.currentBatchCompleted || 0) / Math.max(1, Number(state.currentBatchTotal || 1))),
    "Canceling",
    activeMediaCloudProvider
      ? `Cancel requested. Chatty-art is stopping the current cloud run with ${activeMediaCloudProvider.display_name} and aborting any remaining queued batch items.`
      : "Cancel requested. Chatty-art is stopping the current generation and any remaining queued batch items."
  );

  try {
    const response = await fetch("/api/cancel", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ job_id: state.currentJobId }),
    });

    if (!response.ok) {
      const message = await response.text();
      throw new Error(message || "Cancel request failed.");
    }
  } catch (error) {
    state.canceling = false;
    syncActionState();
    setProgress(
      Math.max(0.05, Number(state.currentBatchCompleted || 0) / Math.max(1, Number(state.currentBatchTotal || 1))),
      "Cancel Failed",
      error.message || "Cancel request failed."
    );
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
      state.currentBatchCompleted += 1;
      if (state.currentBatchCompleted >= state.currentBatchTotal) {
        state.generating = false;
        state.canceling = false;
        state.currentJobId = null;
        state.currentBatchTotal = 1;
        state.currentBatchCompleted = 0;
        syncActionState();
        setProgress(1, "Complete", `${formatKind(event.output.kind)} batch finished and saved to outputs/.`);
      } else {
        setProgress(
          Math.min(0.99, state.currentBatchCompleted / state.currentBatchTotal),
          "Batch Progress",
          `${formatKind(event.output.kind)} ${state.currentBatchCompleted} of ${state.currentBatchTotal} saved. The next batch item is starting with a fresh random seed.`
        );
      }
    }
    return;
  }

  if (event.type === "canceled" && event.job_id === state.currentJobId) {
    state.generating = false;
    state.canceling = false;
    state.currentJobId = null;
    state.currentBatchTotal = 1;
    state.currentBatchCompleted = 0;
    syncActionState();
    setProgress(0, "Canceled", formatGenerationFailureMessage(event.message || "Generation canceled.", { canceled: true }));
    return;
  }

  if (event.type === "error" && event.job_id === state.currentJobId) {
    state.generating = false;
    state.canceling = false;
    state.currentJobId = null;
    state.currentBatchTotal = 1;
    state.currentBatchCompleted = 0;
    syncActionState();
    setProgress(0, "Error", formatGenerationFailureMessage(event.message || "Generation failed."));
  }
}

function formatGenerationFailureMessage(message, { canceled = false } = {}) {
  const text = String(message || "").trim();
  if (!text) {
    return canceled ? "Generation canceled." : "Generation failed.";
  }
  if (canceled) {
    if (/cloud/i.test(text)) {
      return `${text} The remote request was told to stop, and any remaining queued batch items were dropped.`;
    }
    return text;
  }
  if (/Cloud (image|speech) generation via /i.test(text)) {
    return `${text} Check the saved API key, model name, provider status, and any temporary provider-side limits before retrying.`;
  }
  if (/timed out|timeout/i.test(text) && /cloud|provider|openai|gemini|anthropic/i.test(text)) {
    return `${text} The provider took too long to answer. Retry in a moment, or verify the endpoint and model configuration.`;
  }
  return text;
}

function upsertOutput(output) {
  const index = state.outputs.findIndex((entry) => entry.id === output.id);
  if (index === -1) {
    state.outputs.unshift(output);
  } else {
    state.outputs.splice(index, 1, output);
  }
}

function isVisionAssistOnlyModel(model) {
  return Boolean(
    model
    && model.runtime_supported
    && model.backend === "llama_cpp"
    && model.generation_style === "expressive"
    && String(model.family || "").toLowerCase() === "vision"
    && model.supports_image_reference
    && typeof model.mmproj_path === "string"
    && model.mmproj_path.trim()
  );
}

function getVisibleModels() {
  return state.models.filter((model) =>
    model.generation_style === state.generationStyle
    && !isVisionAssistOnlyModel(model)
  );
}

function getSelectedModel() {
  return getVisibleModels().find((model) => model.id === elements.modelSelect.value) || null;
}

function getPromptAssistModels() {
  return state.models.filter((model) =>
    model.runtime_supported
    && model.backend === "llama_cpp"
    && model.generation_style === "expressive"
    && !model.supports_voice_output
    && (model.family || "").toLowerCase() !== "voice"
  );
}

function getPromptAssistCloudModels() {
  return getPromptAssistEligibleProviders().map((provider) => ({
    kind: "cloud",
    id: `cloud:${provider.id}`,
    providerId: provider.id,
    providerKind: provider.provider_kind,
    providerName: provider.display_name,
    name: provider.prompt_assist_model_name,
    modelName: provider.prompt_assist_model_name,
    baseUrl: provider.base_url,
    hasApiKey: provider.has_api_key,
    enabled: provider.enabled !== false,
    verification: provider.prompt_assist_verification || {},
    capabilities: provider.capabilities || {},
  }));
}

function getVisionAssistModels() {
  return state.models.filter((model) =>
    model.runtime_supported
    && model.backend === "llama_cpp"
    && model.generation_style === "expressive"
    && model.supports_image_reference
    && typeof model.mmproj_path === "string"
    && model.mmproj_path.trim()
    && !model.supports_voice_output
    && shouldSurfaceVisionAssistModel(model)
  );
}

function getVisionAssistCloudModels() {
  return getVisionAssistEligibleProviders().map((provider) => ({
    kind: "cloud",
    id: `cloud:${provider.id}`,
    providerId: provider.id,
    providerKind: provider.provider_kind,
    providerName: provider.display_name,
    name: provider.vision_model_name,
    modelName: provider.vision_model_name,
    baseUrl: provider.base_url,
    hasApiKey: provider.has_api_key,
    enabled: provider.enabled !== false,
    verification: provider.vision_assist_verification || {},
    capabilities: provider.capabilities || {},
  }));
}

function shouldSurfaceVisionAssistModel(model) {
  const lower = (model?.name || "").toLowerCase();
  return lower.includes("qwen2.5-vl") || lower.includes("llava");
}

function hasVisionAssistImageReference() {
  return Boolean(state.primaryReference && state.primaryReference.kind === "image");
}

function shouldShowVisionAssistControls() {
  return elements.promptAssistInput.value !== "off";
}

function shouldShowVisionModelSelector() {
  return shouldShowVisionAssistControls();
}

function shouldShowPromptModelSelector() {
  return Boolean(getSelectedModel());
}

function selectedPromptAssistLocalModelId() {
  const value = elements.promptModelSelect.value || "";
  return value && !value.startsWith("cloud:") ? value : null;
}

function selectedVisionAssistLocalModelId() {
  const value = elements.visionModelSelect.value || "";
  return value && !value.startsWith("cloud:") ? value : null;
}

function activePromptAssistCloudProvider() {
  const selection = elements.promptAssistLaneSelect?.value
    || state.cloudLaneAssignments?.prompt_assist
    || "local_auto";
  if (!selection.startsWith("cloud:")) {
    return null;
  }
  const providerId = selection.slice("cloud:".length);
  return getPromptAssistEligibleProviders().find((provider) => provider.id === providerId) || null;
}

function activeMediaGenerationCloudProvider() {
  const selection = elements.mediaGenerationLaneSelect?.value
    || state.cloudLaneAssignments?.media_generation
    || "local_only";
  if (!selection.startsWith("cloud:")) {
    return null;
  }
  const providerId = selection.slice("cloud:".length);
  return getMediaGenerationEligibleProviders().find((provider) => provider.id === providerId) || null;
}

function activeVisionAssistCloudProvider() {
  const selection = elements.visionAssistLaneSelect?.value
    || state.cloudLaneAssignments?.vision_assist
    || "local_auto";
  if (!selection.startsWith("cloud:")) {
    return null;
  }
  const providerId = selection.slice("cloud:".length);
  return getVisionAssistEligibleProviders().find((provider) => provider.id === providerId) || null;
}

function mediaGenerationSupportedKinds(provider) {
  const kinds = [];
  if (provider?.capabilities?.image_generation && String(provider.image_generation_model_name || "").trim()) {
    kinds.push("image");
  }
  if (provider?.capabilities?.video_generation && String(provider.video_generation_model_name || "").trim()) {
    kinds.push("video");
  }
  if (provider?.capabilities?.audio_generation && String(provider.audio_generation_model_name || "").trim()) {
    kinds.push("audio");
  }
  return kinds;
}

function getMediaGenerationCloudModels() {
  return getMediaGenerationEligibleProviders().map((provider) => ({
    kind: "cloud",
    id: `cloud:${provider.id}`,
    providerId: provider.id,
    providerKind: provider.provider_kind,
    providerName: provider.display_name,
    baseUrl: provider.base_url,
    hasApiKey: provider.has_api_key,
    enabled: provider.enabled !== false,
    verification: provider.media_generation_verification || {},
    capabilities: provider.capabilities || {},
    supportedKinds: mediaGenerationSupportedKinds(provider),
    imageModelName: provider.image_generation_model_name || "",
    videoModelName: provider.video_generation_model_name || "",
    audioModelName: provider.audio_generation_model_name || "",
    audioVoice: provider.audio_generation_voice || "",
  }));
}

function getMediaGenerationCloudRoutes(kind = null) {
  const activeKind = kind || elements.prepareKindInput.value || "image";
  return getMediaGenerationCloudModels()
    .filter((provider) => (provider.supportedKinds || []).includes(activeKind))
    .map((provider) => ({
      kind: "cloud-route",
      id: `cloud-route:${provider.providerId}:${activeKind}`,
      laneSelection: `cloud:${provider.providerId}`,
      outputKind: activeKind,
      providerId: provider.providerId,
      providerKind: provider.providerKind,
      providerName: provider.providerName,
      baseUrl: provider.baseUrl,
      hasApiKey: provider.hasApiKey,
      enabled: provider.enabled,
      verification: provider.verification,
      modelName:
        activeKind === "image"
          ? provider.imageModelName
          : activeKind === "video"
            ? provider.videoModelName
            : provider.audioModelName,
      audioVoice: activeKind === "audio" ? provider.audioVoice : "",
      supportedKinds: provider.supportedKinds,
    }));
}

function mediaGenerationLaneSelectionFromRouteValue(value) {
  if (!String(value).startsWith("cloud-route:")) {
    return null;
  }
  const parts = String(value).split(":");
  return parts.length >= 3 ? `cloud:${parts[1]}` : null;
}

function activeMediaGenerationCloudRoute() {
  const provider = activeMediaGenerationCloudProvider();
  const kind = elements.prepareKindInput.value || "image";
  if (!provider) {
    return null;
  }
  return getMediaGenerationCloudRoutes(kind).find((route) => route.providerId === provider.id) || null;
}

function isGeminiCloudVideoProvider(provider) {
  return provider?.provider_kind === "gemini" && mediaGenerationSupportedKinds(provider).includes("video");
}

function isOpenAiCloudVideoProvider(provider) {
  return provider?.provider_kind === "open_ai" && mediaGenerationSupportedKinds(provider).includes("video");
}

function isOpenAiProVideoModel(provider) {
  return String(provider?.video_generation_model_name || "").toLowerCase().includes("sora-2-pro");
}

function getPromptAssistEligibleProviders() {
  return (state.cloudProviders || []).filter((provider) =>
    provider.enabled
    && provider.capabilities?.text_assist
    && String(provider.prompt_assist_model_name || "").trim()
  );
}

function getVisionAssistEligibleProviders() {
  return (state.cloudProviders || []).filter((provider) =>
    provider.enabled
    && provider.capabilities?.vision_assist
    && String(provider.vision_model_name || "").trim()
  );
}

function getMediaGenerationEligibleProviders() {
  return (state.cloudProviders || []).filter((provider) =>
    provider.enabled
    && mediaGenerationSupportedKinds(provider).length > 0
  );
}

function mediaGenerationVideoRouteNote(provider) {
  if (!provider?.video_generation_model_name) {
    return "";
  }
  switch (provider.provider_kind) {
    case "open_ai":
      return " Cloud video on this lane currently uses a temporary deprecated video adapter and must be replaced before September 24, 2026.";
    case "gemini":
      return " Cloud video on this lane currently uses the provider family's compatible video bridge.";
    default:
      return "";
  }
}

function providerKindLabel(providerKind) {
  switch (providerKind) {
    case "open_ai":
      return "OpenAI";
    case "open_ai_compatible":
      return "OpenAI-compatible";
    case "anthropic":
      return "Anthropic";
    case "gemini":
      return "Gemini";
    case "x_ai_grok":
      return "xAI Grok";
    case "deep_seek":
      return "DeepSeek";
    default:
      return "This provider";
  }
}

function unsupportedMediaProviderMessage(provider) {
  switch (provider?.provider_kind) {
    case "anthropic":
      return "Anthropic is currently assist-only in Chatty-art. This saved account or route can still serve Prompt Assist or Vision Assist, but the cloud media lane is not wired for Anthropic yet.";
    case "open_ai_compatible":
      return "This generic OpenAI-compatible route currently covers Prompt Assist and Vision Assist only. Cloud media stays blocked here until a specific compatible-media adapter is wired.";
    case "x_ai_grok":
      return "xAI Grok is currently Prompt Assist-only in Chatty-art. Cloud media stays blocked here until a first-party media adapter exists.";
    case "deep_seek":
      return "DeepSeek is currently Prompt Assist-only in Chatty-art. Cloud media stays blocked here until a first-party media adapter exists.";
    default:
      return `${providerKindLabel(provider?.provider_kind)} is not wired for cloud media generation in Chatty-art yet.`;
  }
}

function mediaRouteInventorySummary(provider) {
  const routes = [];
  if (String(provider?.image_generation_model_name || "").trim()) {
    routes.push(`Image: ${provider.image_generation_model_name.trim()}`);
  }
  if (String(provider?.video_generation_model_name || "").trim()) {
    routes.push(`Video: ${provider.video_generation_model_name.trim()}`);
  }
  if (String(provider?.audio_generation_model_name || "").trim()) {
    const voiceNote = String(provider?.audio_generation_voice || "").trim();
    routes.push(`Speech: ${provider.audio_generation_model_name.trim()}${voiceNote ? ` (voice ${voiceNote})` : ""}`);
  }
  return routes.length
    ? `Configured routes: ${routes.join(" | ")}.`
    : "No image, video, or speech route model is configured yet.";
}

function mediaEditorSelectionStatus(selectedProvider, laneProvider) {
  if (!selectedProvider) {
    return "";
  }
  if (!laneProvider) {
    return "The active output lane is currently local.";
  }
  if (laneProvider.id === selectedProvider.id) {
    return "This saved route is currently active on the output lane.";
  }
  return `Editing ${selectedProvider.display_name}. Active output lane: ${laneProvider.display_name}.`;
}

function unsupportedVisionProviderMessage(provider) {
  if (provider?.provider_kind === "open_ai_compatible") {
    return "This saved OpenAI-compatible route needs an image-capable endpoint plus a Vision Assist model name before it can appear on the Vision Assist lane.";
  }
  if (provider?.provider_kind === "x_ai_grok" || provider?.provider_kind === "deep_seek") {
    return `${providerKindLabel(provider?.provider_kind)} is currently Prompt Assist-only in Chatty-art. Vision Assist is not wired for this family yet.`;
  }
  return `${providerKindLabel(provider?.provider_kind)} is not wired for Vision Assist in Chatty-art yet.`;
}

function cloudProviderDefaults(providerKind) {
  switch (providerKind) {
    case "open_ai":
      return { baseUrl: "https://api.openai.com/v1", model: "gpt-4.1-mini", imageModel: "gpt-image-2", videoModel: "sora-2", audioModel: "gpt-4o-mini-tts", audioVoice: "marin" };
    case "anthropic":
      return { baseUrl: "https://api.anthropic.com/v1", model: "claude-sonnet-5", imageModel: "", videoModel: "", audioModel: "", audioVoice: "" };
    case "gemini":
      return { baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai", model: "gemini-3.5-flash", imageModel: "gemini-2.5-flash-image", videoModel: "veo-3.1-generate-preview", audioModel: "gemini-3.1-flash-tts-preview", audioVoice: "Kore" };
    case "x_ai_grok":
      return { baseUrl: "https://api.x.ai/v1", model: "grok-4-fast-reasoning", imageModel: "", videoModel: "", audioModel: "", audioVoice: "" };
    case "deep_seek":
      return { baseUrl: "https://api.deepseek.com/v1", model: "deepseek-chat", imageModel: "", videoModel: "", audioModel: "", audioVoice: "" };
    case "open_ai_compatible":
      return { baseUrl: "", model: "", imageModel: "", videoModel: "", audioModel: "", audioVoice: "" };
    default:
      return { baseUrl: "", model: "", imageModel: "", videoModel: "", audioModel: "", audioVoice: "" };
  }
}

function buildProviderOptions(providerKinds) {
  return providerKinds
    .map((providerKind) => `<option value="${escapeHtml(providerKind)}">${escapeHtml(providerKindLabel(providerKind))}</option>`)
    .join("");
}

function promptAssistProviderKinds() {
  return ["open_ai", "anthropic", "gemini", "x_ai_grok", "deep_seek", "open_ai_compatible"];
}

function mediaGenerationProviderKinds() {
  return ["open_ai", "gemini"];
}

function visionAssistProviderKinds() {
  return ["open_ai", "anthropic", "gemini", "open_ai_compatible"];
}

function getSelectedPromptAssistCloudProvider() {
  const id = elements.promptAssistCloudEntrySelect.value;
  return state.cloudProviders.find((provider) => provider.id === id) || null;
}

function renderPromptAssistProviderInventory() {
  const current = elements.promptAssistCloudProviderInput.value;
  const providerKinds = promptAssistProviderKinds();
  elements.promptAssistCloudProviderInput.innerHTML = buildProviderOptions(providerKinds);
  if (providerKinds.includes(current)) {
    elements.promptAssistCloudProviderInput.value = current;
  } else if (providerKinds.length) {
    elements.promptAssistCloudProviderInput.value = providerKinds[0];
  }
}

function populatePromptAssistCloudForm() {
  const provider = getSelectedPromptAssistCloudProvider();
  if (!provider) {
    const defaults = cloudProviderDefaults(elements.promptAssistCloudProviderInput.value);
    elements.promptAssistCloudNameInput.value = "";
    elements.promptAssistCloudBaseUrlInput.value = defaults.baseUrl;
    elements.promptAssistCloudModelInput.value = defaults.model;
    elements.promptAssistCloudApiKeyInput.value = "";
    elements.promptAssistCloudEnabledInput.checked = true;
    return;
  }

  elements.promptAssistCloudNameInput.value = provider.display_name || "";
  elements.promptAssistCloudProviderInput.value = provider.provider_kind || "open_ai_compatible";
  elements.promptAssistCloudBaseUrlInput.value = provider.base_url || "";
  elements.promptAssistCloudModelInput.value = provider.prompt_assist_model_name || "";
  elements.promptAssistCloudApiKeyInput.value = "";
  elements.promptAssistCloudEnabledInput.checked = provider.enabled !== false;
}

function renderPromptAssistCloudControls() {
  renderPromptAssistProviderInventory();
  const providers = state.cloudProviders || [];
  const laneProviders = getPromptAssistEligibleProviders();
  const laneSelection = state.cloudLaneAssignments?.prompt_assist || "local_auto";
  const previousSelection = elements.promptAssistCloudEntrySelect.value;
  const laneOptions = [
    `<option value="local_auto">Local Auto (Default)</option>`,
    ...laneProviders
      .map((provider) => `<option value="cloud:${escapeHtml(provider.id)}">Cloud Route: ${escapeHtml(provider.display_name)}</option>`),
  ];
  elements.promptAssistLaneSelect.innerHTML = laneOptions.join("");
  elements.promptAssistLaneSelect.value = laneOptions.some((option) => option.includes(`value="${laneSelection}"`))
    ? laneSelection
    : "local_auto";

  elements.promptAssistCloudEntrySelect.innerHTML = [
    `<option value="">New Account / Route</option>`,
    ...providers.map((provider) => `<option value="${escapeHtml(provider.id)}">${escapeHtml(provider.display_name)}${provider.enabled === false ? " (disabled)" : ""}</option>`),
  ].join("");

  if (providers.some((provider) => provider.id === previousSelection)) {
    elements.promptAssistCloudEntrySelect.value = previousSelection;
  } else if (laneSelection.startsWith("cloud:")) {
    elements.promptAssistCloudEntrySelect.value = laneSelection.slice("cloud:".length);
  } else {
    elements.promptAssistCloudEntrySelect.value = "";
  }

  populatePromptAssistCloudForm();

  const selectedProvider = getSelectedPromptAssistCloudProvider();
  const laneProvider = activePromptAssistCloudProvider();
  const usingCloud = elements.promptAssistLaneSelect.value.startsWith("cloud:");
  if (usingCloud && laneProvider) {
    elements.promptAssistLaneSummary.textContent = `Prompt Assist will use ${laneProvider.prompt_assist_model_name} on ${providerKindLabel(laneProvider.provider_kind)} / ${laneProvider.display_name}. Prompt text leaves this machine for prompt expansion on that lane only; Vision Assist and final media generation stay on their own explicitly chosen lanes.`;
  } else {
    elements.promptAssistLaneSummary.textContent = "Prompt Assist stays local by default. Nothing is uploaded unless you explicitly switch this lane to a cloud provider.";
  }

  if (!selectedProvider) {
    elements.promptAssistCloudStatus.textContent = providers.length
      ? "Choose a saved cloud account to edit it, or leave this on New Account / Route to create another one."
      : "No saved Prompt Assist cloud account or route yet.";
  } else {
    const verification = selectedProvider.prompt_assist_verification || {};
    const status = verification.status ? verification.status : "Not verified yet.";
    const keyNote = selectedProvider.has_api_key ? "API key saved." : "No API key saved.";
    const enabledNote = selectedProvider.enabled === false
      ? "Disabled. This saved account or route stays editable but will not appear in the Prompt Assist lane selector until you re-enable it."
      : "Enabled for lane selection.";
    elements.promptAssistCloudStatus.textContent = `${keyNote} ${enabledNote} ${status}`;
  }

  elements.verifyPromptAssistCloudButton.disabled = !selectedProvider;
  elements.deletePromptAssistCloudButton.disabled = !selectedProvider;
}

function getSelectedMediaGenerationCloudProvider() {
  const id = elements.mediaGenerationCloudEntrySelect.value;
  return state.cloudProviders.find((provider) => provider.id === id) || null;
}

function renderMediaGenerationProviderInventory() {
  const current = elements.mediaGenerationCloudProviderInput.value;
  const providerKinds = mediaGenerationProviderKinds();
  elements.mediaGenerationCloudProviderInput.innerHTML = buildProviderOptions(providerKinds);
  if (providerKinds.includes(current)) {
    elements.mediaGenerationCloudProviderInput.value = current;
  } else if (providerKinds.length) {
    elements.mediaGenerationCloudProviderInput.value = providerKinds[0];
  }
}

function populateMediaGenerationCloudForm() {
  const provider = getSelectedMediaGenerationCloudProvider();
  if (!provider) {
    const defaults = cloudProviderDefaults(elements.mediaGenerationCloudProviderInput.value);
    elements.mediaGenerationCloudNameInput.value = "";
    elements.mediaGenerationCloudBaseUrlInput.value = defaults.baseUrl;
    elements.mediaGenerationCloudImageModelInput.value = defaults.imageModel || defaults.model;
    elements.mediaGenerationCloudVideoModelInput.value = defaults.videoModel || "";
    elements.mediaGenerationCloudAudioModelInput.value = defaults.audioModel || "";
    elements.mediaGenerationCloudVoiceInput.value = defaults.audioVoice || "";
    elements.mediaGenerationCloudApiKeyInput.value = "";
    elements.mediaGenerationCloudEnabledInput.checked = true;
    return;
  }

  elements.mediaGenerationCloudNameInput.value = provider.display_name || "";
  elements.mediaGenerationCloudProviderInput.value = provider.provider_kind || "open_ai_compatible";
  elements.mediaGenerationCloudBaseUrlInput.value = provider.base_url || "";
  elements.mediaGenerationCloudImageModelInput.value = provider.image_generation_model_name || "";
  elements.mediaGenerationCloudVideoModelInput.value = provider.video_generation_model_name || "";
  elements.mediaGenerationCloudAudioModelInput.value = provider.audio_generation_model_name || "";
  elements.mediaGenerationCloudVoiceInput.value = provider.audio_generation_voice || "";
  elements.mediaGenerationCloudApiKeyInput.value = "";
  elements.mediaGenerationCloudEnabledInput.checked = provider.enabled !== false;
}

function renderMediaGenerationCloudControls() {
  renderMediaGenerationProviderInventory();
  const providers = (state.cloudProviders || []).filter((provider) =>
    mediaGenerationProviderKinds().includes(provider.provider_kind)
  );
  const laneProviders = getMediaGenerationEligibleProviders();
  const laneSelection = state.cloudLaneAssignments?.media_generation || "local_only";
  const previousSelection = elements.mediaGenerationCloudEntrySelect.value;
  const laneOptions = [
    `<option value="local_only">Local Only (Default)</option>`,
    ...laneProviders
      .map((provider) => `<option value="cloud:${escapeHtml(provider.id)}">Cloud Route: ${escapeHtml(provider.display_name)}</option>`),
  ];
  elements.mediaGenerationLaneSelect.innerHTML = laneOptions.join("");
  elements.mediaGenerationLaneSelect.value = laneOptions.some((option) => option.includes(`value="${laneSelection}"`))
    ? laneSelection
    : "local_only";

  elements.mediaGenerationCloudEntrySelect.innerHTML = [
    `<option value="">New Account / Route</option>`,
    ...providers.map((provider) => `<option value="${escapeHtml(provider.id)}">${escapeHtml(provider.display_name)}${provider.enabled === false ? " (disabled)" : ""}</option>`),
  ].join("");

  if (providers.some((provider) => provider.id === previousSelection)) {
    elements.mediaGenerationCloudEntrySelect.value = previousSelection;
  } else if (laneSelection.startsWith("cloud:")) {
    elements.mediaGenerationCloudEntrySelect.value = laneSelection.slice("cloud:".length);
  } else {
    elements.mediaGenerationCloudEntrySelect.value = "";
  }

  populateMediaGenerationCloudForm();

  const selectedProvider = getSelectedMediaGenerationCloudProvider();
  const laneProvider = activeMediaGenerationCloudProvider();
  const usingCloud = elements.mediaGenerationLaneSelect.value.startsWith("cloud:");
  if (usingCloud && laneProvider) {
    const kinds = mediaGenerationSupportedKinds(laneProvider).map((kind) => formatKind(kind).toLowerCase());
    const videoNote = mediaGenerationVideoRouteNote(laneProvider);
    const voiceNote = laneProvider.audio_generation_model_name
      ? laneProvider.audio_generation_voice
        ? ` Cloud speech currently uses the provider-specific voice id '${laneProvider.audio_generation_voice}'.`
        : " Cloud speech will use the provider's default voice unless you save a specific voice or speaker id."
      : "";
    elements.mediaGenerationLaneSummary.textContent = `Final ${kinds.join(" + ")} output can be generated through ${laneProvider.display_name}. This is a remote output lane, not the same runtime as local generation. Prompts and any supported references only leave the machine when you explicitly keep this cloud lane selected.${videoNote}${voiceNote}`;
  } else {
    elements.mediaGenerationLaneSummary.textContent = "Local generation is still the default first-run path. Cloud output is opt-in only and remains separate from the local runtimes.";
  }

  if (!selectedProvider) {
    elements.mediaGenerationCloudStatus.textContent = providers.length
      ? "Choose a saved cloud account or route to edit credentials and route declarations, or leave this on New Account / Route to create another one."
      : "No saved cloud media account or route yet.";
  } else {
    const keyNote = selectedProvider.has_api_key ? "API key saved." : "No API key saved.";
    const enabledNote = selectedProvider.enabled === false
      ? "Disabled. This saved account or route stays editable but will not appear in the cloud media lane selector until you re-enable it."
      : "Enabled for lane selection.";
    const selectionNote = mediaEditorSelectionStatus(selectedProvider, laneProvider);
    const routeNote = mediaRouteInventorySummary(selectedProvider);
    const kinds = mediaGenerationSupportedKinds(selectedProvider);
    if (!kinds.length) {
      elements.mediaGenerationCloudStatus.textContent = `${keyNote} ${enabledNote} ${selectionNote} ${routeNote} ${unsupportedMediaProviderMessage(selectedProvider)}`;
    } else {
      const verification = selectedProvider.media_generation_verification || {};
      const status = verification.status ? verification.status : "Not verified yet.";
      elements.mediaGenerationCloudStatus.textContent = `${keyNote} ${enabledNote} ${selectionNote} ${routeNote} ${status}`;
    }
  }

  elements.verifyMediaGenerationCloudButton.disabled = !selectedProvider || !mediaGenerationSupportedKinds(selectedProvider).length;
  elements.deleteMediaGenerationCloudButton.disabled = !selectedProvider;
}

function getSelectedVisionAssistCloudProvider() {
  const id = elements.visionAssistCloudEntrySelect.value;
  return state.cloudProviders.find((provider) => provider.id === id) || null;
}

function renderVisionAssistProviderInventory() {
  const current = elements.visionAssistCloudProviderInput.value;
  const providerKinds = visionAssistProviderKinds();
  elements.visionAssistCloudProviderInput.innerHTML = buildProviderOptions(providerKinds);
  if (providerKinds.includes(current)) {
    elements.visionAssistCloudProviderInput.value = current;
  } else if (providerKinds.length) {
    elements.visionAssistCloudProviderInput.value = providerKinds[0];
  }
}

function populateVisionAssistCloudForm() {
  const provider = getSelectedVisionAssistCloudProvider();
  if (!provider) {
    const defaults = cloudProviderDefaults(elements.visionAssistCloudProviderInput.value);
    elements.visionAssistCloudNameInput.value = "";
    elements.visionAssistCloudBaseUrlInput.value = defaults.baseUrl;
    elements.visionAssistCloudModelInput.value = defaults.model;
    elements.visionAssistCloudApiKeyInput.value = "";
    elements.visionAssistCloudEnabledInput.checked = true;
    return;
  }

  elements.visionAssistCloudNameInput.value = provider.display_name || "";
  elements.visionAssistCloudProviderInput.value = provider.provider_kind || "open_ai_compatible";
  elements.visionAssistCloudBaseUrlInput.value = provider.base_url || "";
  elements.visionAssistCloudModelInput.value = provider.vision_model_name || "";
  elements.visionAssistCloudApiKeyInput.value = "";
  elements.visionAssistCloudEnabledInput.checked = provider.enabled !== false;
}

function renderVisionAssistCloudControls() {
  renderVisionAssistProviderInventory();
  const shouldShow = shouldShowVisionAssistControls();
  const hasImageReference = hasVisionAssistImageReference();
  elements.visionAssistCloudBlock.classList.toggle("hidden", !shouldShow);
  if (!shouldShow) {
    return;
  }

  const providers = (state.cloudProviders || []).filter((provider) =>
    visionAssistProviderKinds().includes(provider.provider_kind)
  );
  const laneProviders = getVisionAssistEligibleProviders();
  const laneSelection = state.cloudLaneAssignments?.vision_assist || "local_auto";
  const previousSelection = elements.visionAssistCloudEntrySelect.value;
  const laneOptions = [
    `<option value="local_auto">Local Auto (Default)</option>`,
    ...laneProviders
      .map((provider) => `<option value="cloud:${escapeHtml(provider.id)}">Cloud Route: ${escapeHtml(provider.display_name)}</option>`),
  ];
  elements.visionAssistLaneSelect.innerHTML = laneOptions.join("");
  elements.visionAssistLaneSelect.value = laneOptions.some((option) => option.includes(`value="${laneSelection}"`))
    ? laneSelection
    : "local_auto";
  elements.visionAssistLaneSelect.disabled = !hasImageReference;

  elements.visionAssistCloudEntrySelect.innerHTML = [
    `<option value="">New Account / Route</option>`,
    ...providers.map((provider) => `<option value="${escapeHtml(provider.id)}">${escapeHtml(provider.display_name)}${provider.enabled === false ? " (disabled)" : ""}</option>`),
  ].join("");

  if (providers.some((provider) => provider.id === previousSelection)) {
    elements.visionAssistCloudEntrySelect.value = previousSelection;
  } else if (laneSelection.startsWith("cloud:")) {
    elements.visionAssistCloudEntrySelect.value = laneSelection.slice("cloud:".length);
  } else {
    elements.visionAssistCloudEntrySelect.value = "";
  }

  populateVisionAssistCloudForm();

  const selectedProvider = getSelectedVisionAssistCloudProvider();
  const laneProvider = activeVisionAssistCloudProvider();
  const usingCloud = elements.visionAssistLaneSelect.value.startsWith("cloud:");
  if (!hasImageReference) {
    elements.visionAssistLaneSummary.textContent = "Vision Assist becomes active only after you assign a still image as the primary reference. You can save or preselect local/cloud Vision Assist now, but no image analysis runs until that reference is in place.";
  } else if (usingCloud && laneProvider) {
    elements.visionAssistLaneSummary.textContent = `Vision Assist will inspect reference images with ${laneProvider.vision_model_name} on ${providerKindLabel(laneProvider.provider_kind)} / ${laneProvider.display_name}. That means the selected image may leave this machine on this lane; Prompt Assist and final generation can still stay local or use their own separate cloud lanes.`;
  } else {
    elements.visionAssistLaneSummary.textContent = "Vision Assist stays local by default. No reference image is uploaded unless you explicitly switch this lane to a cloud provider.";
  }

  if (!selectedProvider) {
    elements.visionAssistCloudStatus.textContent = providers.length
      ? "Choose a saved cloud account to edit it, or leave this on New Account / Route to create another one."
      : "No saved Vision Assist cloud account or route yet.";
  } else {
    const keyNote = selectedProvider.has_api_key ? "API key saved." : "No API key saved.";
    const enabledNote = selectedProvider.enabled === false
      ? "Disabled. This saved account or route stays editable but will not appear in the Vision Assist lane selector until you re-enable it."
      : "Enabled for lane selection.";
    if (!selectedProvider.capabilities?.vision_assist || !String(selectedProvider.vision_model_name || "").trim()) {
      elements.visionAssistCloudStatus.textContent = `${keyNote} ${enabledNote} ${unsupportedVisionProviderMessage(selectedProvider)}`;
    } else {
      const verification = selectedProvider.vision_assist_verification || {};
      const status = verification.status ? verification.status : "Not verified yet.";
      elements.visionAssistCloudStatus.textContent = `${keyNote} ${enabledNote} ${status}`;
    }
  }

  elements.verifyVisionAssistCloudButton.disabled = !selectedProvider || !selectedProvider.capabilities?.vision_assist || !String(selectedProvider.vision_model_name || "").trim();
  elements.deleteVisionAssistCloudButton.disabled = !selectedProvider;
}

async function savePromptAssistLaneSelection() {
  try {
    const response = await postJson("/api/cloud/lanes", {
      lane_assignments: {
        ...state.cloudLaneAssignments,
        prompt_assist: elements.promptAssistLaneSelect.value,
      },
    });
    state.cloudLaneAssignments = response.lane_assignments || state.cloudLaneAssignments;
    clearPreparedHandoff();
    renderPromptAssistCloudControls();
    renderPromptModelSelector();
    setProgress(0.02, "Prompt Assist", "Saved Prompt Assist lane selection.");
  } catch (error) {
    setProgress(0, "Cloud Lane", error.message || "Could not save Prompt Assist lane selection.");
    renderPromptAssistCloudControls();
  }
}

async function saveMediaGenerationLaneSelection() {
  try {
    const response = await postJson("/api/cloud/lanes", {
      lane_assignments: {
        ...state.cloudLaneAssignments,
        media_generation: elements.mediaGenerationLaneSelect.value,
      },
    });
    state.cloudLaneAssignments = response.lane_assignments || state.cloudLaneAssignments;
    clearPreparedHandoff();
    renderMediaGenerationCloudControls();
    renderModels();
    renderPrepareKindOptions();
    setProgress(0.02, "Output Route", "Saved media generation route.");
  } catch (error) {
    setProgress(0, "Cloud Lane", error.message || "Could not save media generation route.");
    renderMediaGenerationCloudControls();
  }
}

async function saveMediaGenerationCloudProvider() {
  try {
    const existing = getSelectedMediaGenerationCloudProvider();
    const nextProviderKind = elements.mediaGenerationCloudProviderInput.value;
    const providerKindChanged = existing && existing.provider_kind !== nextProviderKind;
    if (providerKindChanged && !elements.mediaGenerationCloudApiKeyInput.value.trim()) {
      throw new Error("Paste a new API key before switching this saved account or route to a different provider family.");
    }
    const payload = {
      id: existing ? existing.id : null,
      display_name: elements.mediaGenerationCloudNameInput.value.trim(),
      provider_kind: nextProviderKind,
      base_url: elements.mediaGenerationCloudBaseUrlInput.value.trim(),
      prompt_assist_model_name: providerKindChanged ? "" : existing?.prompt_assist_model_name || "",
      vision_model_name: providerKindChanged ? "" : existing?.vision_model_name || "",
      image_generation_model_name: elements.mediaGenerationCloudImageModelInput.value.trim(),
      video_generation_model_name: elements.mediaGenerationCloudVideoModelInput.value.trim(),
      audio_generation_model_name: elements.mediaGenerationCloudAudioModelInput.value.trim(),
      audio_generation_voice: elements.mediaGenerationCloudVoiceInput.value.trim(),
      enabled: elements.mediaGenerationCloudEnabledInput.checked,
      api_key: elements.mediaGenerationCloudApiKeyInput.value.trim() || null,
    };
    await postJson("/api/cloud/providers", payload);
    elements.mediaGenerationCloudApiKeyInput.value = "";
    await Promise.all([loadCloudProviders(), loadCloudLaneAssignments()]);
    clearPreparedHandoff();
    renderMediaGenerationCloudControls();
    renderModels();
    renderPrepareKindOptions();
    setProgress(0.02, "Cloud Route", "Saved cloud media account / route.");
  } catch (error) {
    setProgress(0, "Cloud Route", error.message || "Could not save the cloud account or route.");
  }
}

async function savePromptAssistCloudProvider() {
  try {
    const existing = getSelectedPromptAssistCloudProvider();
    const nextProviderKind = elements.promptAssistCloudProviderInput.value;
    const providerKindChanged = existing && existing.provider_kind !== nextProviderKind;
    if (providerKindChanged && !elements.promptAssistCloudApiKeyInput.value.trim()) {
      throw new Error("Paste a new API key before switching this saved account or route to a different provider family.");
    }
    const payload = {
      id: existing ? existing.id : null,
      display_name: elements.promptAssistCloudNameInput.value.trim(),
      provider_kind: nextProviderKind,
      base_url: elements.promptAssistCloudBaseUrlInput.value.trim(),
      prompt_assist_model_name: elements.promptAssistCloudModelInput.value.trim(),
      vision_model_name: providerKindChanged ? "" : existing?.vision_model_name || "",
      image_generation_model_name: providerKindChanged ? "" : existing?.image_generation_model_name || "",
      video_generation_model_name: providerKindChanged ? "" : existing?.video_generation_model_name || "",
      audio_generation_model_name: providerKindChanged ? "" : existing?.audio_generation_model_name || "",
      audio_generation_voice: providerKindChanged ? "" : existing?.audio_generation_voice || "",
      enabled: elements.promptAssistCloudEnabledInput.checked,
      api_key: elements.promptAssistCloudApiKeyInput.value.trim() || null,
    };
    await postJson("/api/cloud/providers", payload);
    elements.promptAssistCloudApiKeyInput.value = "";
    await Promise.all([loadCloudProviders(), loadCloudLaneAssignments()]);
    clearPreparedHandoff();
    renderPromptModelSelector();
    setProgress(0.02, "Cloud Provider", "Saved Prompt Assist cloud provider.");
  } catch (error) {
    setProgress(0, "Cloud Route", error.message || "Could not save the cloud account or route.");
  }
}

async function saveVisionAssistLaneSelection() {
  try {
    const response = await postJson("/api/cloud/lanes", {
      lane_assignments: {
        ...state.cloudLaneAssignments,
        vision_assist: elements.visionAssistLaneSelect.value,
      },
    });
    state.cloudLaneAssignments = response.lane_assignments || state.cloudLaneAssignments;
    clearPreparedHandoff();
    renderVisionAssistCloudControls();
    renderVisionModelSelector();
    setProgress(0.02, "Vision Assist", "Saved Vision Assist lane selection.");
  } catch (error) {
    setProgress(0, "Cloud Lane", error.message || "Could not save Vision Assist lane selection.");
    renderVisionAssistCloudControls();
  }
}

async function saveVisionAssistCloudProvider() {
  try {
    const existing = getSelectedVisionAssistCloudProvider();
    const nextProviderKind = elements.visionAssistCloudProviderInput.value;
    const providerKindChanged = existing && existing.provider_kind !== nextProviderKind;
    if (providerKindChanged && !elements.visionAssistCloudApiKeyInput.value.trim()) {
      throw new Error("Paste a new API key before switching this saved account or route to a different provider family.");
    }
    const payload = {
      id: existing ? existing.id : null,
      display_name: elements.visionAssistCloudNameInput.value.trim(),
      provider_kind: nextProviderKind,
      base_url: elements.visionAssistCloudBaseUrlInput.value.trim(),
      prompt_assist_model_name: providerKindChanged ? "" : existing?.prompt_assist_model_name || "",
      vision_model_name: elements.visionAssistCloudModelInput.value.trim(),
      image_generation_model_name: providerKindChanged ? "" : existing?.image_generation_model_name || "",
      video_generation_model_name: providerKindChanged ? "" : existing?.video_generation_model_name || "",
      audio_generation_model_name: providerKindChanged ? "" : existing?.audio_generation_model_name || "",
      audio_generation_voice: providerKindChanged ? "" : existing?.audio_generation_voice || "",
      enabled: elements.visionAssistCloudEnabledInput.checked,
      api_key: elements.visionAssistCloudApiKeyInput.value.trim() || null,
    };
    await postJson("/api/cloud/providers", payload);
    elements.visionAssistCloudApiKeyInput.value = "";
    await Promise.all([loadCloudProviders(), loadCloudLaneAssignments()]);
    clearPreparedHandoff();
    renderVisionAssistCloudControls();
    renderVisionModelSelector();
    setProgress(0.02, "Cloud Provider", "Saved Vision Assist cloud provider.");
  } catch (error) {
    setProgress(0, "Cloud Route", error.message || "Could not save the cloud account or route.");
  }
}

async function verifyPromptAssistCloudProvider() {
  const provider = getSelectedPromptAssistCloudProvider();
  if (!provider) {
    setProgress(0, "Cloud Verify", "Choose a saved cloud account or route first.");
    return;
  }
  try {
    const response = await postJson("/api/cloud/providers/verify", {
      id: provider.id,
      lane: "prompt_assist",
    });
    await loadCloudProviders();
    renderPromptAssistCloudControls();
    renderPromptModelSelector();
    setProgress(0.02, "Cloud Verify", response.note || `Verified ${provider.display_name}.`);
  } catch (error) {
    setProgress(0, "Cloud Verify", error.message || "Cloud route verification failed.");
  }
}

async function deletePromptAssistCloudProvider() {
  const provider = getSelectedPromptAssistCloudProvider();
  if (!provider) {
    setProgress(0, "Cloud Delete", "Choose a saved cloud account or route first.");
    return;
  }
  try {
    await postJson("/api/cloud/providers/delete", { id: provider.id });
    await Promise.all([loadCloudProviders(), loadCloudLaneAssignments()]);
    clearPreparedHandoff();
    renderPromptModelSelector();
    setProgress(0.02, "Cloud Delete", `Deleted ${provider.display_name}.`);
  } catch (error) {
    setProgress(0, "Cloud Delete", error.message || "Could not delete the saved cloud account or route.");
  }
}

async function verifyMediaGenerationCloudProvider() {
  const provider = getSelectedMediaGenerationCloudProvider();
  if (!provider) {
    setProgress(0, "Cloud Verify", "Choose a saved cloud account or route first.");
    return;
  }
  try {
    const response = await postJson("/api/cloud/providers/verify", {
      id: provider.id,
      lane: "media_generation",
    });
    await loadCloudProviders();
    renderMediaGenerationCloudControls();
    renderModels();
    setProgress(0.02, "Cloud Verify", response.note || `Verified ${provider.display_name}.`);
  } catch (error) {
    setProgress(0, "Cloud Verify", error.message || "Cloud route verification failed.");
  }
}

async function deleteMediaGenerationCloudProvider() {
  const provider = getSelectedMediaGenerationCloudProvider();
  if (!provider) {
    setProgress(0, "Cloud Delete", "Choose a saved cloud account or route first.");
    return;
  }
  try {
    await postJson("/api/cloud/providers/delete", { id: provider.id });
    await Promise.all([loadCloudProviders(), loadCloudLaneAssignments()]);
    clearPreparedHandoff();
    renderMediaGenerationCloudControls();
    renderModels();
    setProgress(0.02, "Cloud Delete", `Deleted ${provider.display_name}.`);
  } catch (error) {
    setProgress(0, "Cloud Delete", error.message || "Could not delete the saved cloud account or route.");
  }
}

async function verifyVisionAssistCloudProvider() {
  const provider = getSelectedVisionAssistCloudProvider();
  if (!provider) {
    setProgress(0, "Cloud Verify", "Choose a saved cloud account or route first.");
    return;
  }
  try {
    const response = await postJson("/api/cloud/providers/verify", {
      id: provider.id,
      lane: "vision_assist",
    });
    await loadCloudProviders();
    renderVisionAssistCloudControls();
    renderVisionModelSelector();
    setProgress(0.02, "Cloud Verify", response.note || `Verified ${provider.display_name}.`);
  } catch (error) {
    setProgress(0, "Cloud Verify", error.message || "Cloud route verification failed.");
  }
}

async function deleteVisionAssistCloudProvider() {
  const provider = getSelectedVisionAssistCloudProvider();
  if (!provider) {
    setProgress(0, "Cloud Delete", "Choose a saved cloud account or route first.");
    return;
  }
  try {
    await postJson("/api/cloud/providers/delete", { id: provider.id });
    await Promise.all([loadCloudProviders(), loadCloudLaneAssignments()]);
    clearPreparedHandoff();
    renderVisionAssistCloudControls();
    renderVisionModelSelector();
    setProgress(0.02, "Cloud Delete", `Deleted ${provider.display_name}.`);
  } catch (error) {
    setProgress(0, "Cloud Delete", error.message || "Could not delete the saved cloud account or route.");
  }
}

function renderPromptModelSelector() {
  const shouldShow = shouldShowPromptModelSelector();
  elements.promptModelBlock.classList.remove("hidden");
  const activeCloudProvider = activePromptAssistCloudProvider();

  if (!shouldShow) {
    elements.promptModelSelect.innerHTML = `<option value="">Auto (Recommended)</option>`;
    elements.promptModelSelect.disabled = true;
    elements.promptModelSummary.innerHTML = buildPromptAssistMessageCard("Pick a generation model first to unlock Prompt Assist model selection. Auto keeps Prompt Assist on the safer text-only route by default, while advanced users can pin a multimodal expressive model here if they want one model handling prompt expansion and image analysis.");
    return;
  }

  const promptAssistEnabled = elements.promptAssistInput.value !== "off";
  const selected = elements.promptModelSelect.value;
  const promptModels = getPromptAssistModels();
  const cloudModels = getPromptAssistCloudModels();

  if (!promptModels.length && !cloudModels.length) {
    elements.promptModelSelect.innerHTML = `<option value="">No Prompt Assist models found</option>`;
    elements.promptModelSelect.disabled = true;
    elements.promptModelSummary.innerHTML = buildPromptAssistMessageCard(
      promptAssistEnabled
        ? activeCloudProvider
          ? `Prompt Assist is routed through ${activeCloudProvider.prompt_assist_model_name} on ${providerKindLabel(activeCloudProvider.provider_kind)} / ${activeCloudProvider.display_name}. Local helper discovery is currently empty, but cloud prompt expansion is available as a separate remote lane.`
          : "Prompt Assist needs at least one local expressive llama.cpp model in models/. Auto prefers plain text helpers, but you can pin multimodal expressive models here when you deliberately want them doing both jobs."
        : "Turn Prompt Assist on to use this selector. When enabled, Chatty-art prefers plain text helpers by default, but you can pin multimodal expressive models here when you deliberately want them doing both jobs."
    );
    return;
  }

  elements.promptModelSelect.disabled = false;
  elements.promptModelSelect.innerHTML = buildPromptAssistSelectorOptions(promptModels, cloudModels);

  const activeCloudValue = activeCloudProvider ? `cloud:${activeCloudProvider.id}` : "";
  if (selected && (promptModels.some((model) => model.id === selected) || cloudModels.some((model) => model.id === selected))) {
    elements.promptModelSelect.value = selected;
  } else if (activeCloudValue && cloudModels.some((model) => model.id === activeCloudValue)) {
    elements.promptModelSelect.value = activeCloudValue;
  } else {
    elements.promptModelSelect.value = "";
  }

  const chosen = promptModels.find((model) => model.id === elements.promptModelSelect.value) || null;
  const chosenCloud = cloudModels.find((model) => model.id === elements.promptModelSelect.value) || null;
  const multimodal = chosen && chosen.supports_image_reference && typeof chosen.mmproj_path === "string" && chosen.mmproj_path.trim();
  if (!promptAssistEnabled) {
    elements.promptModelSummary.innerHTML = chosenCloud
      ? buildCloudPromptAssistSummary(chosenCloud, "Prompt Assist is currently off. This cloud selection stays saved as an explicit route choice, but nothing will be sent until Prompt Assist is turned on.")
      : chosen
        ? buildLocalPromptAssistSummary(chosen, `${chosen.name} is selected as the Prompt Assist interpreter, but Prompt Assist is currently off. Turn it on to use this model.`)
        : buildPromptAssistMessageCard("Prompt Assist is currently off. You can still pick a model here in advance; Chatty-art will use it once Prompt Assist is turned on. Auto keeps Prompt Assist on the safer text-only route by default when enabled.");
    return;
  }

  if (chosenCloud) {
    elements.promptModelSummary.innerHTML = buildCloudPromptAssistSummary(
      chosenCloud,
      `Prompt Assist is currently routed through ${chosenCloud.modelName} on ${providerKindLabel(chosenCloud.providerKind)} / ${chosenCloud.providerName}. This selection reuses the saved cloud lane instead of the local llama.cpp helper path.`
    );
    return;
  }

  elements.promptModelSummary.innerHTML = chosen
    ? buildLocalPromptAssistSummary(
        chosen,
        multimodal
          ? `${chosen.name} is pinned as the Prompt Assist interpreter. This intentionally overrides the safe text-only default and allows one multimodal expressive model to handle both prompt expansion and image analysis.`
          : `${chosen.name} is pinned as the Prompt Assist interpreter. Leave this on Auto if you want Chatty-art to stay on the safer text-only route.`
      )
    : buildPromptAssistMessageCard("Auto keeps Prompt Assist on the safer text-only route by default. Advanced users can pin a multimodal expressive model here if they want one model handling prompt expansion and image analysis.");
}

function buildPromptAssistSelectorOptions(localModels, cloudModels) {
  return [
    `<option value="">Auto (Recommended)</option>`,
    localModels.length
      ? `<optgroup label="Local GGUFs">${localModels.map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildDropdownLabel(model))}</option>`).join("")}</optgroup>`
      : "",
    cloudModels.length
      ? `<optgroup label="Cloud routes">${cloudModels.map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildPromptAssistCloudDropdownLabel(model))}</option>`).join("")}</optgroup>`
      : "",
  ].join("");
}

function buildPromptAssistCloudDropdownLabel(model) {
  const provider = providerKindLabel(model.providerKind);
  return `${model.modelName} | ${provider} | ${model.providerName} [Cloud Route]`;
}

function buildPromptAssistMessageCard(message) {
  return `
    <div class="model-summary-card">
      <div class="model-summary-copy">${escapeHtml(message)}</div>
    </div>
  `;
}

function buildLocalPromptAssistSummary(model, message) {
  const stateInfo = describeModelState(model);
  const badges = [
    createModelBadge("Local", "backend"),
    createModelBadge(stateInfo.label, `state-${stateInfo.tone}`),
    createModelBadge(formatBackendBadge(model.backend), "backend"),
    createModelBadge(model.family, "family"),
  ];
  if ((model.supported_kinds || []).length) {
    badges.push(createModelBadge(`Outputs: ${formatKinds(model.supported_kinds)}`, "outputs"));
  }
  const runtimeLine = buildExplicitRuntimeLine(model);
  const recommendations = buildRecommendedLimitsMarkup(model);
  return `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(model.name)}</strong>
      </div>
      <div class="model-badges">${badges.join("")}</div>
      ${runtimeLine ? `<div class="model-summary-runtime">${escapeHtml(runtimeLine)}</div>` : ""}
      <div class="model-summary-copy">${escapeHtml(message)}</div>
      ${recommendations}
    </div>
  `;
}

function buildCloudPromptAssistSummary(model, message) {
  const verificationStatus = model.verification?.status ? model.verification.status : "Not verified yet.";
  const providerBits = [
    providerKindLabel(model.providerKind),
    model.providerName,
    model.hasApiKey ? "API key saved" : "No API key saved",
  ];
  return `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(model.modelName)}</strong>
      </div>
      <div class="model-badges">
        ${createModelBadge("Cloud", "backend")}
        ${createModelBadge(providerKindLabel(model.providerKind), "family")}
        ${createModelBadge("Prompt Assist", "outputs")}
      </div>
      <div class="model-summary-runtime">${escapeHtml(providerBits.join(" | "))}</div>
      <div class="model-summary-copy">${escapeHtml(message)}</div>
      <div class="recommended-limits">
        <div class="recommended-limits-head">
          <strong>Cloud Route Details</strong>
          <span>${escapeHtml(model.baseUrl || "Saved route endpoint")}</span>
        </div>
        <div class="recommended-limits-list">
          <div class="recommended-limit-row current-safe">
            <strong>Privacy</strong>
            <span>Prompt text leaves this machine only when this cloud route stays selected.</span>
            <span>Reference images remain on their own Vision Assist lane unless you explicitly switch that lane too.</span>
            <span class="recommended-current current-safe"><em>Verification:</em> ${escapeHtml(verificationStatus)}</span>
            <span class="recommended-current-note">No automatic fallback occurs. If this route fails, Prompt Assist fails visibly instead of hopping lanes.</span>
          </div>
          <div class="recommended-limit-row current-stretch">
            <strong>Usage shape</strong>
            <span><em>Best fit:</em> prompt expansion, cleanup, structure, and optional richer brief drafting.</span>
            <span><em>Watch for:</em> route latency, rate limits, and family-specific prompt behavior.</span>
            <span><em>Configured model:</em> ${escapeHtml(model.modelName)}</span>
            <span class="recommended-current current-stretch"><em>Route:</em> ${escapeHtml(cloudRouteIdentity(model, "Prompt Assist"))}</span>
            <span class="recommended-current-note">This mirrors the local recommendation panel with cloud-facing route details instead of hardware pressure.</span>
          </div>
        </div>
        <div class="recommended-limits-note">Cloud Prompt Assist is route-aware rather than hardware-aware. Expect latency and policy behavior to vary by endpoint and queue depth.</div>
      </div>
    </div>
  `;
}

function renderVisionModelSelector() {
  const shouldShow = shouldShowVisionModelSelector();
  const hasImageReference = hasVisionAssistImageReference();
  elements.visionModelBlock.classList.toggle("hidden", !shouldShow);
  renderVisionAssistCloudControls();
  const activeCloudProvider = activeVisionAssistCloudProvider();

  if (!shouldShow) {
    elements.visionModelSelect.innerHTML = `<option value="">Auto (Recommended)</option>`;
    elements.visionModelSelect.disabled = true;
    elements.visionModelSummary.innerHTML = buildPromptAssistMessageCard("Turn Prompt Assist on to review or preselect Vision Assist. Once Prompt Assist is active, you can keep Vision Assist on Auto or preselect a local/cloud image-analysis route before assigning a still image.");
    return;
  }

  const selected = elements.visionModelSelect.value;
  const visionModels = getVisionAssistModels();
  const cloudModels = getVisionAssistCloudModels();

  if (!visionModels.length && !cloudModels.length) {
    elements.visionModelSelect.innerHTML = `<option value="">No Vision Assist models found</option>`;
    elements.visionModelSelect.disabled = true;
    elements.visionModelSummary.innerHTML = buildPromptAssistMessageCard("Prompt Assist can still run, but Vision Assist has no surfaced local or cloud helper right now. Recommended local pair: Qwen2.5-VL-7B-Instruct-Q4_K_M.gguf plus mmproj-Qwen2.5-VL-7B-Instruct-f16.gguf. Fallback local pair: llava-v1.5-7b-Q4_K_M.gguf plus llava-v1.5-7b-mmproj-model-f16.gguf.");
    return;
  }

  elements.visionModelSelect.disabled = false;
  elements.visionModelSelect.innerHTML = buildVisionAssistSelectorOptions(visionModels, cloudModels);

  const activeCloudValue = activeCloudProvider ? `cloud:${activeCloudProvider.id}` : "";
  if (selected && (visionModels.some((model) => model.id === selected) || cloudModels.some((model) => model.id === selected))) {
    elements.visionModelSelect.value = selected;
  } else if (activeCloudValue && cloudModels.some((model) => model.id === activeCloudValue)) {
    elements.visionModelSelect.value = activeCloudValue;
  } else {
    elements.visionModelSelect.value = "";
  }

  const chosen = visionModels.find((model) => model.id === elements.visionModelSelect.value) || null;
  const chosenCloud = cloudModels.find((model) => model.id === elements.visionModelSelect.value) || null;
  if (!hasImageReference) {
    if (chosenCloud) {
      elements.visionModelSummary.innerHTML = buildCloudVisionAssistSummary(
        chosenCloud,
        `Vision Assist is preselected to use ${chosenCloud.modelName} on ${providerKindLabel(chosenCloud.providerKind)} / ${chosenCloud.providerName}, but it will stay inactive until you assign a still image as the primary reference.`
      );
      return;
    }
    elements.visionModelSummary.innerHTML = chosen
      ? buildLocalVisionAssistSummary(
          chosen,
          `${visionAssistLabel(chosen)} is preselected as the Vision Assist helper, but it will stay inactive until you assign a still image as the primary reference.`
        )
      : buildPromptAssistMessageCard("Vision Assist is available now because Prompt Assist is on, but it only runs after you assign a still image as the primary reference. Auto will prefer Qwen2.5-VL-7B first and fall back to LLaVA once an image is present.");
    return;
  }
  if (chosenCloud) {
    elements.visionModelSummary.innerHTML = buildCloudVisionAssistSummary(
      chosenCloud,
      `Vision Assist is currently routed through ${chosenCloud.modelName} on ${providerKindLabel(chosenCloud.providerKind)} / ${chosenCloud.providerName}. This selection reuses the saved Vision Assist cloud lane instead of the local multimodal helper path.`
    );
    return;
  }
  elements.visionModelSummary.innerHTML = chosen
    ? buildLocalVisionAssistSummary(
        chosen,
        `${visionAssistLabel(chosen)} is pinned as the image-analysis helper for Prompt Assist. Leave this on Auto if you want Chatty-art to choose for you.`
      )
    : buildPromptAssistMessageCard("Auto lets Chatty-art choose a local multimodal helper for image analysis before Prompt Assist expands the handoff. Right now Auto prefers Qwen2.5-VL-7B first and falls back to LLaVA.");
}

function buildVisionAssistSelectorOptions(localModels, cloudModels) {
  return [
    `<option value="">Auto (Recommended)</option>`,
    localModels.length
      ? `<optgroup label="Local GGUFs">${localModels.map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(visionAssistLabel(model))}</option>`).join("")}</optgroup>`
      : "",
    cloudModels.length
      ? `<optgroup label="Cloud routes">${cloudModels.map((model) => `<option value="${escapeHtml(model.id)}">${escapeHtml(buildVisionAssistCloudDropdownLabel(model))}</option>`).join("")}</optgroup>`
      : "",
  ].join("");
}

function buildVisionAssistCloudDropdownLabel(model) {
  const provider = providerKindLabel(model.providerKind);
  return `${model.modelName} | ${provider} | ${model.providerName} [Cloud Route]`;
}

function buildLocalVisionAssistSummary(model, message) {
  const stateInfo = describeModelState(model);
  const badges = [
    createModelBadge("Local", "backend"),
    createModelBadge(stateInfo.label, `state-${stateInfo.tone}`),
    createModelBadge(formatBackendBadge(model.backend), "backend"),
    createModelBadge("Vision Assist", "outputs"),
  ];
  if (model.supports_image_reference) {
    badges.push(createModelBadge("Image analysis", "reference"));
  }
  const runtimeLine = buildExplicitRuntimeLine(model);
  const recommendations = buildRecommendedLimitsMarkup(model);
  return `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(visionAssistLabel(model))}</strong>
      </div>
      <div class="model-badges">${badges.join("")}</div>
      ${runtimeLine ? `<div class="model-summary-runtime">${escapeHtml(runtimeLine)}</div>` : ""}
      <div class="model-summary-copy">${escapeHtml(message)}</div>
      ${recommendations}
    </div>
  `;
}

function buildCloudVisionAssistSummary(model, message) {
  const verificationStatus = model.verification?.status ? model.verification.status : "Not verified yet.";
  const providerBits = [
    providerKindLabel(model.providerKind),
    model.providerName,
    model.hasApiKey ? "API key saved" : "No API key saved",
  ];
  return `
    <div class="model-summary-card">
      <div class="model-summary-head">
        <strong class="model-summary-name">${escapeHtml(model.modelName)}</strong>
      </div>
      <div class="model-badges">
        ${createModelBadge("Cloud", "backend")}
        ${createModelBadge(providerKindLabel(model.providerKind), "family")}
        ${createModelBadge("Vision Assist", "outputs")}
      </div>
      <div class="model-summary-runtime">${escapeHtml(providerBits.join(" | "))}</div>
      <div class="model-summary-copy">${escapeHtml(message)}</div>
      <div class="recommended-limits">
        <div class="recommended-limits-head">
          <strong>Cloud Route Details</strong>
          <span>${escapeHtml(model.baseUrl || "Saved route endpoint")}</span>
        </div>
        <div class="recommended-limits-list">
          <div class="recommended-limit-row current-safe">
            <strong>Privacy</strong>
            <span>The selected reference image may leave this machine only when this Vision Assist cloud route stays selected.</span>
            <span>Prompt Assist and final generation can still stay on their own local or cloud lanes.</span>
            <span class="recommended-current current-safe"><em>Verification:</em> ${escapeHtml(verificationStatus)}</span>
            <span class="recommended-current-note">No automatic fallback occurs. If this route fails, Vision Assist fails visibly instead of switching to another lane.</span>
          </div>
          <div class="recommended-limit-row current-stretch">
            <strong>Usage shape</strong>
            <span><em>Best fit:</em> image analysis, visual cue extraction, scene breakdown, and edit-preservation hints.</span>
            <span><em>Watch for:</em> route latency, image-input policy checks, and endpoint-specific multimodal behavior.</span>
            <span><em>Configured model:</em> ${escapeHtml(model.modelName)}</span>
            <span class="recommended-current current-stretch"><em>Route:</em> ${escapeHtml(cloudRouteIdentity(model, "Vision Assist"))}</span>
            <span class="recommended-current-note">This mirrors the local recommendation panel with cloud-facing route details instead of hardware pressure.</span>
          </div>
        </div>
        <div class="recommended-limits-note">Cloud Vision Assist is route-aware rather than hardware-aware. Expect latency and image-input behavior to vary by endpoint and queue depth.</div>
      </div>
    </div>
  `;
}

function visionAssistLabel(model) {
  const name = model?.name || "";
  const lower = name.toLowerCase();
  if (lower.includes("qwen2.5-vl")) {
    return `${name} (Preferred)`;
  }
  if (lower.includes("llava")) {
    return `${name} (Fallback)`;
  }
  return name;
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
    ? `<div class="model-summary-foot">${escapeHtml(`${hiddenModeCount} file(s) are hidden from this generator picker right now.`)}</div>`
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
  syncChattyCogBridgeStatus();
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
          <span>This older video file may not preview inline. MP4 is now the preferred export format.</span>
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
  if (backend === "cloud") return "cloud provider";
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

async function postJson(url, payload) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Request failed for ${url}`);
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

function syncChattyCogBridgeStatus() {
  if (!window.chattyCogBridge?.available || typeof window.chattyCogBridge.updateStatus !== "function") {
    return;
  }

  const payload = buildChattyCogBridgeStatus();
  const fingerprint = JSON.stringify(payload);
  if (!fingerprint || fingerprint === lastBridgeStatusFingerprint) {
    return;
  }

  lastBridgeStatusFingerprint = fingerprint;
  window.chattyCogBridge.updateStatus(payload);
}

function buildChattyCogBridgeStatus() {
  const selectedModel = getSelectedModel();
  const currentOutput = state.currentPreview;
  const prompt = elements.promptInput.value.trim();
  const audioPrompt = elements.audioLiteralPromptInput.value.trim();
  const activePrompt = prompt || audioPrompt;
  const outputCount = state.outputs.length;
  const assignedReferences = [
    state.primaryReference ? `${state.primaryReference.name} (${referenceIntentLabel(state.referenceIntent)})` : null,
    state.endReference ? `${state.endReference.name} (end frame)` : null,
    state.controlReference ? `${state.controlReference.name} (control)` : null,
  ].filter(Boolean);
  const preparedKind = state.preparedHandoff?.kind ? formatKind(state.preparedHandoff.kind) : null;
  const latestOutputLabel = currentOutput
    ? `${formatKind(currentOutput.kind)} - ${currentOutput.file_name || currentOutput.name || "saved output"}`
    : null;

  let statusLine = "Chatty-art is idle.";
  if (state.generating && state.currentJobId) {
    statusLine = `Chatty-art is generating media in ${state.generationStyle} mode.`;
  } else if (state.preparing) {
    statusLine = "Chatty-art is preparing a generation handoff preview.";
  } else if (preparedKind) {
    statusLine = `Chatty-art has a prepared ${preparedKind.toLowerCase()} handoff ready to review.`;
  } else if (latestOutputLabel) {
    statusLine = `Chatty-art is idle after saving ${latestOutputLabel}.`;
  }

  const summaryParts = [
    statusLine,
    selectedModel ? `Model: ${selectedModel.name}.` : "Model not selected yet.",
    activePrompt ? `Prompt focus: ${truncateBridgeText(activePrompt, 140)}.` : "Prompt is still empty.",
    assignedReferences.length ? `References: ${assignedReferences.join("; ")}.` : "No references assigned.",
    latestOutputLabel ? `Latest output: ${latestOutputLabel}.` : outputCount ? `Saved outputs available: ${outputCount}.` : "No saved outputs yet.",
  ];

  const snapshotLines = [
    "# Chatty-art Snapshot",
    "",
    `- Generation style: ${state.generationStyle}`,
    `- Workflow mode: ${state.workflowMode}`,
    `- Current phase: ${elements.progressPhase.textContent.trim() || "(unset)"}`,
    `- Current status: ${elements.progressMessage.textContent.trim() || "(unset)"}`,
    `- Selected model: ${selectedModel?.name || "(none)"}`,
    `- Prompt assist: ${elements.promptAssistInput.value || "(unset)"}`,
    `- Seed: ${elements.seedInput.value.trim() || "(random)"}`,
    `- Prepared handoff: ${preparedKind || "none"}`,
    `- Prompt: ${prompt || "(empty)"}`,
    `- Audio words/sounds: ${audioPrompt || "(empty)"}`,
    `- Negative prompt: ${elements.negativePromptInput.value.trim() || "(empty)"}`,
    `- Manual focus cues: ${elements.manualFocusCuesInput.value.trim() || "(empty)"}`,
    `- Selected references: ${assignedReferences.length ? assignedReferences.join(" | ") : "none"}`,
    `- Current output count: ${outputCount}`,
    `- Latest output: ${latestOutputLabel || "none"}`,
  ];

  return {
    module_id: "chatty_art",
    event_type: "suspend_rundown",
    summary: summaryParts.join(" "),
    snapshot: snapshotLines.join("\n"),
    tags: ["generation", "media", "prompt", "reference", "output", "webview"],
    payload: {
      activity_hint: state.generating
        ? "Host is generating media"
        : state.preparing
          ? "Host is preparing a handoff preview"
          : preparedKind
            ? "Host is reviewing a prepared handoff"
            : "Host is adjusting media settings",
      generation_style: state.generationStyle,
      workflow_mode: state.workflowMode,
      generating: state.generating,
      preparing: state.preparing,
      canceling: state.canceling,
      current_job_id: state.currentJobId,
      current_batch_total: state.currentBatchTotal,
      current_batch_completed: state.currentBatchCompleted,
      selected_model_id: selectedModel?.id || null,
      selected_model_name: selectedModel?.name || null,
      selected_model_backend: selectedModel?.backend || null,
      prepared_handoff_kind: state.preparedHandoff?.kind || null,
      prompt: prompt || null,
      audio_prompt: audioPrompt || null,
      negative_prompt: elements.negativePromptInput.value.trim() || null,
      output_count: outputCount,
      latest_output: currentOutput
        ? {
            id: currentOutput.id || null,
            kind: currentOutput.kind || null,
            file_name: currentOutput.file_name || currentOutput.name || null,
            model: currentOutput.model || null,
            url: currentOutput.url || null,
          }
        : null,
      references: {
        primary: state.primaryReference?.name || null,
        primary_intent: state.primaryReference ? state.referenceIntent : null,
        end: state.endReference?.name || null,
        control: state.controlReference?.name || null,
      },
    },
    updated_at_unix_ms: Date.now(),
  };
}

function truncateBridgeText(value, limit) {
  const text = String(value || "").replace(/\s+/g, " ").trim();
  if (!text) {
    return "";
  }
  if (text.length <= limit) {
    return text;
  }
  return `${text.slice(0, Math.max(0, limit - 3)).trim()}...`;
}
