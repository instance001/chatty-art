# Chatty-art
Sister repo to local, AWD Windows focused LoRA trainer found at:
https://github.com/instance001/chatty-lora

![Chatty-art screenshot](<./Screenshot 2026-03-20 104930.png>)

Standalone local media generator, and ChattyCog-compatible module drop-in.

(Drop me in ChattyCog's `modules/` folder!)

Chatty-art is a simple local image/GIF/video/audio generator with:

- GGUF auto-detection from `models/`
- Bundled `llama.cpp` runtime from `runtime/`
- Local `stable-diffusion.cpp` realism backend sourced from `diffuse_runtime/`
- Plain HTML/CSS/JS single-page dashboard
- Rust backend with WebSocket progress updates
- Auto-save into `outputs/`
- Optional reference media selection from both `input/` and previously generated files in `outputs/`
- Input Tray `Use as Guide` / `Edit Selected` controls for reference-driven runs
- Separate `Generate GIF` and `Generate Video` paths with video resolution, duration, and FPS controls
- `Sequential Batch Count` for repeating the same job one generation at a time with a fresh random seed on each run
- `Cancel Run` beside the generate buttons so the current job or remaining sequential batch can be stopped cleanly
- `Low VRAM Mode` for safer realism jobs on tighter GPUs
- Live `ECG Window` under the progress area on Windows, similar to Task Manager
- Model-aware `Recommended Limits On This Hardware` guidance in the UI
- Collapsible `Controls`, `Outputs`, and `Input Tray` columns for easier layout management
- Optional `Prompt Assist` compiler that expands short prompts into richer local briefs before generation
- Optional `Vision Assist` stage for still-image `Guide` / `Edit Selected` references before Prompt Assist expands the final handoff
- Advanced realism controls for `Sampler`, `Scheduler`, `Reference Strength`, `Flow Shift`, and family-aware `LoRA`
- Manual `Focus Cues` and `Defaults / Assumptions` fields in `Realism + Advanced` for steering the handoff when Prompt Assist needs help
- Dedicated audio `Words / Script` or `Words / Sounds` box for realism audio models
- Dedicated `Voice Reference` tray assignment for realism speech models like `OuteTTS`
- `Basic / Advanced` prompt mode split, so the beginner path stays simple while advanced users get deeper controls
- Advanced audio sequencing with reusable voice/layer names, plus `after last box` or `same time as last box` timing
- Hosted ChattyCog handoff actions for sending selected outputs to `Chatty-lora` as dataset candidates or to `Chatty_Sandbox` for review
- Hosted LoRA inbox import from sibling modules through the approved `lora_imports` bridge lane
- Explicit `Delete Selected` cleanup for saved outputs, using the same checkbox surface as handoff

If you want a true beginner walkthrough, start with [USER_MANUAL.md](./USER_MANUAL.md).

## License

Chatty-art's project code and documentation are licensed under the GNU Affero General Public License v3.0 or later (`AGPLv3-or-later`). See [LICENSE](./LICENSE).

Important note:

- Chatty-art itself is `AGPLv3-or-later`.
- Bundled or checked-out third-party runtimes such as `llama.cpp`, `stable-diffusion.cpp`, and their dependencies keep their own upstream licenses.
- When you redistribute Chatty-art, make sure you preserve both Chatty-art's AGPL terms and any separate notices required by bundled third-party components.

## Recommended Starter Stack

If you want the easiest current `Realism` setup, use this exact starter pack:

- a full `stable-diffusion.cpp` checkout with submodules
- `stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf`
- `v2-1_768-nonema-pruned-Q4_0.gguf`
- `wan2.1-t2v-14b-Q4_K_M.gguf`
- `wan_2.1_vae.safetensors`
- `umt5-xxl-encoder-Q4_K_M.gguf`

Exact links:

- Runtime project:
  https://github.com/leejet/stable-diffusion.cpp
- Runtime releases:
  https://github.com/leejet/stable-diffusion.cpp/releases
- Recommended setup command:

```powershell
git clone --recurse-submodules https://github.com/leejet/stable-diffusion.cpp diffuse_runtime
```

- `stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf`
  https://huggingface.co/second-state/stable-diffusion-v1-5-GGUF/resolve/main/stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf
- `v2-1_768-nonema-pruned-Q4_0.gguf`
  https://huggingface.co/second-state/stable-diffusion-2-1-GGUF/resolve/main/v2-1_768-nonema-pruned-Q4_0.gguf
- `wan2.1-t2v-14b-Q4_K_M.gguf`
  https://huggingface.co/city96/Wan2.1-T2V-14B-gguf/resolve/main/wan2.1-t2v-14b-Q4_K_M.gguf
- `wan_2.1_vae.safetensors`
  https://huggingface.co/Comfy-Org/Wan_2.1_ComfyUI_repackaged/resolve/main/split_files/vae/wan_2.1_vae.safetensors
- `umt5-xxl-encoder-Q4_K_M.gguf`
  https://huggingface.co/city96/umt5-xxl-encoder-gguf/resolve/main/umt5-xxl-encoder-Q4_K_M.gguf

Why this set:

- `SD1.5` and `SD2.1` are simple self-contained image models.
- `Wan2.1 T2V` is the easiest current local video family to start with.
- The Wan helper files above are the minimum companion files needed for that video path.
- This avoids more fragile starter choices like random SD3.5 merges, FLUX companion bundles, or the heavier Wan2.2 A14B paired-model setups.

Put them here:

- use a full `stable-diffusion.cpp` checkout with submodules in `diffuse_runtime/`
  - preferred:
    `git clone --recurse-submodules https://github.com/leejet/stable-diffusion.cpp diffuse_runtime`
- if you use a downloaded source archive instead, make sure `diffuse_runtime/ggml/CMakeLists.txt` exists afterward
- put the other 5 files into `models/`

## Recommended Upgrade Stack

Once the starter stack is working, this is the cleanest next tested image step:

- `flux1-schnell-q4_0.gguf`
- `ae.safetensors`
- `clip_l.safetensors`
- `t5xxl_fp16.safetensors`

Exact links:

- `flux1-schnell-q4_0.gguf`
  https://huggingface.co/leejet/FLUX.1-schnell-gguf/resolve/main/flux1-schnell-q4_0.gguf
- `ae.safetensors`
  https://huggingface.co/black-forest-labs/FLUX.1-schnell/resolve/main/ae.safetensors
- `clip_l.safetensors`
  https://huggingface.co/black-forest-labs/FLUX.1-schnell/resolve/main/clip_l.safetensors
- `t5xxl_fp16.safetensors`
  https://huggingface.co/black-forest-labs/FLUX.1-schnell/resolve/main/t5xxl_fp16.safetensors

Keep using these files from the starter stack:

- `wan_2.1_vae.safetensors`
- `umt5-xxl-encoder-Q4_K_M.gguf`
- `stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf`
- `v2-1_768-nonema-pruned-Q4_0.gguf`

Why this upgrade set:

- `FLUX.1-schnell` is a stronger image model than `SD1.5` or `SD2.1`, but still much easier to manage than heavier multi-part setups.
- The starter `Wan2.1 T2V 14B` lane remains the easiest plain local video recommendation.
- This gives Chatty-art a practical middle tier without jumping straight to the biggest models.

## Later Video Upgrade

If you want to try the newer Wan2.2 hybrid lane after the starter stack is already working, use:

- `Wan2.2-TI2V-5B-Q4_K_M.gguf`
- `Wan2.2_VAE.safetensors`
- `umt5-xxl-encoder-Q4_K_M.gguf`

Exact links:

- `Wan2.2-TI2V-5B-Q4_K_M.gguf`
  https://huggingface.co/QuantStack/Wan2.2-TI2V-5B-GGUF
- `Wan2.2_VAE.safetensors`
  https://huggingface.co/QuantStack/Wan2.2-TI2V-5B-GGUF/blob/main/VAE/Wan2.2_VAE.safetensors
- `umt5-xxl-encoder-Q4_K_M.gguf`
  https://huggingface.co/city96/umt5-xxl-encoder-gguf/resolve/main/umt5-xxl-encoder-Q4_K_M.gguf

Important notes:

- `Wan2.2 TI2V 5B` is the simpler Wan2.2 hybrid lane here.
- One main GGUF can handle both text-to-video and image-to-video.
- It prefers `Wan2.2_VAE.safetensors`.
- It still needs a `umt5` or `t5xxl` text encoder companion file.
- It is not the same thing as the paired Wan2.2 A14B high-noise / low-noise setups.

## Heavier Wan2.2 A14B Lane

If you want to round out the full Wan2.2 paired path after `TI2V 5B` is already making sense, the `A14B` line in Chatty-art expects:

- one `Wan2.2-T2V-A14B` or `Wan2.2-I2V-A14B` low-noise GGUF
- the matching high-noise GGUF partner
- `Wan2.2_VAE.safetensors`
- `umt5-xxl-encoder-*.gguf` or another compatible `umt5` / `t5xxl`
- `clip_vision_h.safetensors` for image-conditioned `I2V` runs

Important notes:

- `T2V A14B` is the paired text-to-video line.
- `I2V A14B` is the paired image-to-video line.
- Chatty-art uses the low-noise file as the primary model and passes the matching high-noise file as the companion diffusion model.
- Keep the low-noise and high-noise pair from the same conversion family, and match their quant suffixes such as both `Q4_K_M` or both `Q5_K_M`.
- `Wan2.2 A14B` prefers `Wan2.2_VAE.safetensors`.
- `I2V A14B` also needs a start image from the tray plus `clip_vision_h.safetensors`.

## Model Ladder

Use this as a plain-language guide:

- `Expressive` text GGUFs
  Best for the simplest all-in-one local image, GIF, and audio workflow.
- `SD1.5` / `SD2.1`
  Easiest realism image models. Good for first tests and weaker hardware.
- `FLUX.1-schnell`
  Stronger realism image model than `SD1.5` / `SD2.1`, while still consumer-PC friendly.
- `Wan2.1 T2V 14B`
  Current known-good local video model, but still a heavier realism path.
- `Wan2.2 TI2V 5B`
  The simpler Wan2.2 hybrid lane. One main model can cover both text-to-video and image-to-video, but it still wants `Wan2.2_VAE.safetensors` plus a UMT5/T5 encoder.
- Smaller Wan video conversions such as `VACE 1.3B`
  Experimental for now. Promising, but not yet pinned as the default beginner recommendation.
- `Wan2.2` paired A14B models and heavier multi-part families
  Stronger but easier to mismatch. Better as a later upgrade, not a first install.

## Audio Downloads

`Expressive` is still the easiest one-click audio path, but Chatty-art now also supports specialist realism audio lanes:

- `OuteTTS` for speech / voice output
- `Stable Audio Open` for ambience, effects, and soundscape-style clips

These are the current audio downloads to keep on hand:

- `OuteTTS-1.0-0.6B-FP16.gguf`
  https://huggingface.co/OuteAI/OuteTTS-1.0-0.6B-GGUF/resolve/main/OuteTTS-1.0-0.6B-FP16.gguf
- `Llama-OuteTTS-1.0-1B-FP16.gguf`
  https://huggingface.co/OuteAI/Llama-OuteTTS-1.0-1B-GGUF/resolve/main/Llama-OuteTTS-1.0-1B-FP16.gguf
- `OuteTTS` runtime project:
  https://github.com/edwko/OuteTTS

For the heavier realism soundscape lane, keep this package together:

- `stable-audio-open-1.0` model package:
  https://huggingface.co/stabilityai/stable-audio-open-1.0
- `stable-audio-tools` runtime project:
  https://github.com/Stability-AI/stable-audio-tools

Where they go:

- Put the `OuteTTS` `.gguf` files directly into `models/`
- Keep the `stable-audio-open-1.0` package together as a folder under `models/stable-audio-open-1.0/`
- Keep source/runtime repos out of `models/`
  - `audio_runtime/outetts/`
  - `audio_runtime/stable_audio_tools/`

Important:

- `OuteTTS` is the cleaner first target for realism-style speech audio
- `stable-audio-open-1.0` is a full model package, not a one-file GGUF
- `stable-audio-open-1.0` is not part of the current image/video starter stack and is not required for the existing realism visual workflow
- If you download Stable Audio via `hf download`, keep the folder structure intact

## Audio Prompt Workflow

When you pick a realism audio model, Chatty-art can work in two prompt modes:

- `Basic`
  The clean beginner path. You get the normal audio prompt boxes and can generate quickly.
- `Advanced`
  The deeper control path. You can add multiple timed audio boxes and reuse names to keep a stable identity across the sequence.

In `Basic`, Chatty-art can show three different text boxes:

- `Prompt`
  Use this for descriptors and intent.
  Think: tone, accent, pacing, environment, delivery, texture, mood, recording style.
- `Negative Prompt`
  Use this for what you do not want.
  Think: muddy audio, robotic voice, clipping, harsh hiss, distorted bass, overprocessed sound.
- `Words / Script` or `Words / Sounds`
  This is the literal lane.
  For speech models like `OuteTTS`, use it for the exact words you want spoken.
  For sound models like `Stable Audio Open`, use it for literal cue words you want preserved, like `dripping water, distant thunder, crackling fire`.

For realism speech models, the Input Tray can also show:

- `Use as Voice Reference`
  Choose an audio clip from either `Input Folder` or `Output Folder`, then assign it as the voice reference.
  Chatty-art will hand that audio file to `OuteTTS` as the cloning reference for the generated speech.
  Short `.wav` clips work best.
  Keep voice-reference clips at `20 seconds or less`, and ideally around `15 seconds or under`.

In `Advanced`, audio models can expand that literal lane into a sequence builder:

- Add multiple boxes with `Add new prompt box`
- Remove a box with the `X` in the box corner
- Choose whether each box starts `after last box` or `same time as last box`
- Reuse the same name to keep the same identity across segments

Identity rule:

- `OuteTTS`
  Reusing the same `Voice Name / Character Note` tells Chatty-art to keep the same character-like voice identity across those segments.
- `Stable Audio Open`
  Reusing the same `Layer Name / Sound Note` tells Chatty-art to keep the same seeded sound identity across those segments.

Beginner rule:

- speech model:
  - `Prompt` = how it should sound
  - `Negative Prompt` = what to avoid in the delivery
  - `Words / Script` = exactly what should be said
  - `Voice Reference` = whose voice to imitate
- soundscape model:
  - `Prompt` = overall scene and texture
  - `Negative Prompt` = what to avoid in the sound design
  - `Words / Sounds` = literal cue list

Example speech setup:

- `Prompt`
  `warm Australian female voice, calm pacing, clear diction, close microphone, gentle smile`
- `Words / Script`
  `Welcome to Chatty-art. Local generation is ready.`

Example speech setup with cloning:

- `Prompt`
  `calm male narration, clear pacing, warm tone, slight radio texture`
- `Words / Script`
  `The local generation run is complete.`
- `Voice Reference`
  `short prerecorded WAV voice clip from the tray`

Example soundscape setup:

- `Prompt`
  `quiet nighttime forest ambience, cinematic depth, soft wind, natural field recording`
- `Words / Sounds`
  `distant owl, dry leaves, soft wind, creek water`

Example advanced speech setup:

- Box 1
  - `Voice Name / Character Note`
    `Narrator`
  - timing
    `after last box`
  - `Words / Script`
    `Welcome to Chatty-art.`
- Box 2
  - `Voice Name / Character Note`
    `Narrator`
  - timing
    `after last box`
  - `Words / Script`
    `Everything is running locally on this machine.`
- Box 3
  - `Voice Name / Character Note`
    `Caller`
  - timing
    `same time as last box`
  - `Words / Script`
    `Can you hear me?`

Example advanced sound setup:

- Box 1
  - `Layer Name / Sound Note`
    `Rain Bed`
  - timing
    `after last box`
  - `Words / Sounds`
    `steady rain, soft roof patter`
- Box 2
  - `Layer Name / Sound Note`
    `Thunder Hit`
  - timing
    `same time as last box`
  - `Words / Sounds`
    `distant thunder crack`
- Box 3
  - `Layer Name / Sound Note`
    `Rain Bed`
  - timing
    `after last box`
  - `Words / Sounds`
    `steady rain, soft roof patter`

## Advanced Realism Controls

When you switch to `Realism + Advanced`, Chatty-art can show extra realism controls for models that support them.

The goal is to keep `Basic` mode clean while still giving power users room to experiment.

The current advanced realism controls are:

- `Sampler`
  Chooses the main sampling method for the realism backend.
  If you do not know what to pick, leave it on `Euler`.

- `Scheduler`
  Chooses how noise is distributed across the generation run.
  If you are not intentionally testing combinations, leave it on `Auto / Runtime Default`.

- `Reference Strength`
  Only appears for realism models and workflows that actually use still-image reference strength.
  Higher values hold closer to the reference image.
  Lower values give the model more freedom to drift.

- `Flow Shift`
  Only appears for model families that use it, such as `Wan` and `Qwen`.
  This is an advanced flow-model tuning control.
  If you are not deliberately experimenting, leave it at the default value.

- `Manual Focus Cues`
  Lets you type your own short visual cues directly into the handoff.
  Good examples are things like `golden hour`, `shallow depth of field`, `wet pavement`, or `cinematic framing`.
  These are useful when Prompt Assist misses an important visual priority or when you want to add your own extra steering without rewriting the whole prompt.

- `Manual Defaults / Assumptions`
  Lets you type the sensible defaults you want the handoff to assume.
  Good examples are things like `adult woman`, `stormy coast`, `modern city street`, or `overcast afternoon`.
  These are useful when the base prompt is short and you want to lock in a few concrete assumptions yourself instead of relying on the automatic expansion stage.

- `LoRA`
  Appears in `Realism + Advanced` for local `stable-diffusion.cpp` realism models.
  Chatty-art supports a small LoRA stack here, with a separate strength slider for each added LoRA row.
  Put LoRA files in `models/loras/<family>/` or `models/lora/<family>/`, for example:
  - `models/loras/flux/`
  - `models/loras/sd/`
  - `models/loras/sd3/`
  - `models/loras/wan/`
  - `models/lora/flux/`
  - `models/lora/sd/`
  - `models/lora/sd3/`
  - `models/lora/wan/`
  Supported LoRA file types are `.safetensors` and `.ckpt`.
  Chatty-art will only show LoRAs that match the selected model family.

- `LoRA Strength`
  Each added LoRA row has its own strength slider.
  Lower values are gentler.
  Higher values push harder toward that LoRA's style or subject behavior.
  If you are just starting, around `0.6` to `1.0` per LoRA is a sensible range.

Simple beginner advice:

- `Basic` mode is still the best place to start.
- Use `Realism + Advanced` only when you want to tune behavior on purpose.
- Start with:
  - `Sampler = Euler`
  - `Scheduler = Auto / Runtime Default`
  - `Reference Strength = default`
  - `Flow Shift = default`
- `LoRA = off` until the base model is behaving the way you want
- Change one thing at a time so you can tell what actually helped.

## LoRA Basics

`LoRA` stands for `Low-Rank Adaptation`.

Plain-language meaning:

- a LoRA is a small add-on file that changes how a base model behaves
- it usually pushes the model toward a style, character, look, camera feel, or subject behavior
- it is not a full model replacement
- you still need the main base model first

Good beginner mental model:

- base model = the main engine
- LoRA = a bolt-on tuning pack

In Chatty-art today:

- LoRA is available in `Realism + Advanced`
- Chatty-art supports a small stacked LoRA workflow
- each added LoRA gets its own strength slider
- Chatty-art only shows LoRAs that match the selected model family

### Where LoRAs Go

Put LoRA files inside `models/loras/<family>/` or `models/lora/<family>/`

Examples:

- `models/loras/flux/`
- `models/loras/sd/`
- `models/loras/sd3/`
- `models/loras/wan/`
- `models/lora/flux/`
- `models/lora/sd/`
- `models/lora/sd3/`
- `models/lora/wan/`

Supported file types:

- `.safetensors`
- `.ckpt`

### Matching The Right LoRA

This part matters a lot:

- `FLUX` LoRAs should be used with `FLUX` models
- `SD1.5` / `SD2.1` style LoRAs belong in the general `sd` bucket
- `SD3` / `SD3.5` LoRAs belong in `sd3`
- `Wan` LoRAs belong in `wan`

If the family does not match, the LoRA usually will not show up or will not behave properly.

### Tips For Finding LoRAs

The easiest search pattern is:

- base model family name
- plus the word `LoRA`
- plus the file type or ecosystem you want

Examples:

- `FLUX LoRA safetensors`
- `Stable Diffusion 1.5 LoRA safetensors`
- `SD3 LoRA safetensors`
- `Wan LoRA safetensors`

Good beginner rule:

- if a LoRA page does not clearly say what base family it was trained for, skip it
- if the examples are all for a different model family than yours, skip it
- if you are unsure, test with the LoRA off first, then on at `1.0`

### First LoRA Advice

If you are new:

- get the base model working first
- turn LoRA on only after the base model is already producing sensible results
- start with one LoRA before building a stack
- start around `LoRA Strength = 0.6` to `1.0`
- if the result becomes too distorted or overpowering, lower the strength
- if the LoRA effect is too weak, raise it slowly
- if you stack multiple LoRAs, add them one at a time and keep the first stacked pass conservative
- only change one advanced control at a time

## Run

1. Drop one or more `.gguf` models into `models/`
2. Put optional outside reference files into:
   - `input/images`
   - `input/video`
   - `input/audio`
3. Previously generated files in `outputs/` will also appear in the tray automatically under `Output Folder`.
4. If you want `Realism` mode, also place any required companion weights into `models/`.
   - `Qwen Image` needs its VAE and Qwen2.5-VL text encoder.
- `Wan` models need a Wan VAE and a `umt5`/`t5xxl` text encoder.
  - `Wan2.1` usually pairs with `wan_2.1_vae.safetensors`
  - `Wan2.2 TI2V 5B` prefers `Wan2.2_VAE.safetensors`
  - `Wan2.2` A14B T2V / I2V also prefers `Wan2.2_VAE.safetensors`
5. Start the app:

```powershell
cargo run
```

Or use:

```powershell
.\launch-chatty-art.ps1
```

The app opens at `http://127.0.0.1:7878`.

## Notes

- The bundled Vulkan-capable `llama.cpp` runtime is used for planning with your GGUF model.
- Expressive mode uses the bundled `llama.cpp` runtime and Chatty-art's local renderer.
- Realism mode uses `stable-diffusion.cpp` locally. On the first realism run, Chatty-art builds `sd-cli` from `diffuse_runtime/` automatically.
- For the cleanest first realism setup, prefer the exact 6-file starter stack listed above.
- The Input Tray now shows both `Input Folder` files and `Output Folder` files so you can reuse generated material without moving it by hand.
- The Input Tray lets you choose whether the selected file should be used as a `Guide` or treated as the image to `Edit`.
- For realism speech models, the Input Tray also lets you assign an audio clip as `Voice Reference`.
- Short `.wav` clips work best for `Voice Reference`, and OuteTTS expects the cloning clip to be `20 seconds or less`.
- The dashboard columns can be collapsed with `Hide` and restored from the bottom-right dock as `Controls`, `Outputs`, and `Input Tray`.
- In realism mode, still-image references can be selected from the tray, including files from `input/` or previously generated output images.
- Realism audio uses specialist backends alongside the realism visual lane:
  - `OuteTTS` for speech
  - `Stable Audio Open` for soundscapes / SFX
- Advanced audio currently stays within one backend at a time.
  - `OuteTTS` handles multi-segment speech
  - `Stable Audio Open` handles multi-segment sound layers
  - Chatty-art does not yet merge speech and sound backends into one combined audio job
- `Preview Handoff` can now be used in both `Basic` and `Advanced` modes.
- `Realism + Advanced` can surface extra controls like `Sampler`, `Scheduler`, `Reference Strength`, and `Flow Shift` depending on the selected model family.
- `Realism + Advanced` can also surface `Manual Focus Cues` and `Manual Defaults / Assumptions` so you can add your own handoff steering directly.
- `Prompt Assist` can be set to `Off`, `Gentle`, or `Strong`.
- Prompt Assist uses a local expressive `llama.cpp` model as an interpreter role before generation.
- When Prompt Assist is working from a still-image reference, Chatty-art can also run `Vision Assist` first.
- This now applies to both:
  - `Expressive` image handoffs
  - `Realism` image handoffs
- Vision Assist uses a local multimodal `llama.cpp` helper plus its matching `mmproj` file to read the image before Prompt Assist expands the final handoff.
- The current proven base is still-image `Use as Guide` and `Edit Selected`.
- Right now the recommended local helpers are:
  - `Qwen2.5-VL-7B-Instruct` as the primary option
  - `LLaVA v1.5 7B` as the fallback option
- `Auto` in the UI now prefers `Qwen2.5-VL-7B` first and falls back to `LLaVA`.
- `Vision Assist Model` only appears when:
  - Prompt Assist is on
  - a still image is assigned as `Use as Guide` or `Edit Selected`
- Experimental helpers such as `Moondream` and smaller `Qwen2-VL` variants are parked for later for now, so the surfaced selector stays focused on the two most reliable lanes.
- Best results usually come from reference images with:
  - one clear subject
  - readable lighting
  - one main focal point instead of a very busy collage
- For `Edit Selected`, the most reliable prompt pattern is:
  - say what must stay
  - say the one exact thing to add or change
  - say what must not happen
- In `Realism + Advanced`, use these boxes to split that instruction cleanly:
  - `Manual Keep / Preserve`
  - `Manual Change Targets`
  - `Manual Avoid`
- Example: adding a new subject
  - Prompt:
    `Keep the kookaburra intact on the branch. Add a small separate green tree frog sitting on the kookaburra's back like a rider.`
  - `Manual Keep / Preserve`:
    `kookaburra, branch, background, overall pose`
  - `Manual Change Targets`:
    `add small green tree frog rider on kookaburra's back`
  - `Manual Avoid`:
    `merged bodies, single hybrid animal, frog replacing bird, extra limbs`
- Example: replacing the subject
  - Prompt:
    `Replace the kookaburra with a frog on the same branch in the same scene.`
  - `Manual Keep / Preserve`:
    `branch, background, lighting, framing`
  - `Manual Change Targets`:
    `replace kookaburra with frog`
  - `Manual Avoid`:
    `bird remaining in frame, merged bird and frog, extra heads`
- Example: changing the background only
  - Prompt:
    `Keep the bird exactly as it is. Change the background to a rainy forest.`
  - `Manual Keep / Preserve`:
    `bird, pose, branch, framing`
  - `Manual Change Targets`:
    `rainy forest background`
  - `Manual Avoid`:
    `bird changing species, extra animals, merged background and subject`
- Practical edit tips:
  - Prefer one main edit per run instead of stacking several big requests together.
  - If the model replaces instead of adding, include the word `separate`.
  - If the model keeps changing too much, strengthen `Manual Keep / Preserve` before making the prompt longer.
  - Put exact failure modes like `merged bodies` into `Manual Avoid` instead of hoping the model infers them.
- Current download targets:
  - `Qwen2.5-VL-7B-Instruct-Q4_K_M.gguf`
  - `mmproj-Qwen2.5-VL-7B-Instruct-f16.gguf`
  - `llava-v1.5-7b-Q4_K_M.gguf`
  - `llava-v1.5-7b-mmproj-model-f16.gguf`
- Current model links:
  - `Qwen2.5-VL-7B-Instruct-GGUF`: https://huggingface.co/ggml-org/Qwen2.5-VL-7B-Instruct-GGUF/tree/main
  - `LLaVA v1.5 7B GGUF`: https://huggingface.co/QuantFactory/llava-v1.5-7b-GGUF/tree/main
- Handy search terms if those repos move:
  - `ggml-org Qwen2.5-VL-7B-Instruct-GGUF Hugging Face`
  - `llava-v1.5-7b GGUF mmproj Hugging Face`
- For realism speech models, Prompt Assist now separates spoken words from delivery direction.
- For realism sound models, Prompt Assist only expands the descriptive prompt and negative prompt.
- For realism audio models, the `Words / Script` or `Words / Sounds` field is the best place for verbatim content.
- For realism speech models, `Voice Reference` stays separate from Prompt Assist and is passed through as the cloning clip.
- `Generate GIF` and `Generate Video` are separate on purpose. GIF is usually the easier preview/share format, while true local video depends on the selected realism family.
- `Sequential Batch Count` reruns the same prompt, settings, references, and LoRA stack one job at a time. Each extra run behaves like clearing the seed box and pressing Generate again.
- GIF/video settings include clip resolution, duration, and FPS.
- When `Sequential Batch Count` is greater than `1`, Chatty-art rolls a fresh random seed for each run instead of reusing a manual seed.
- `Cancel Run` stops the current generation and also prevents the rest of a sequential batch from continuing.
- `Low VRAM Mode` uses a safer realism launch profile that spills more work to CPU and tiles VAE decode when needed.
- The UI now shows `Recommended Limits On This Hardware` based on the selected model, output kind, detected GPU, and whether `Low VRAM Mode` is on.
- On Windows, the progress area includes a small `ECG Window` that shows the busiest local GPU engine as an ECG-style activity line, similar to the Task Manager graph.
- If `diffuse_runtime/ggml` is missing, restore the `ggml` submodule or re-copy a full source tree before using realism mode.
- Realism models may need extra local support files in `models/`, not just one GGUF.
- Expressive image output is saved as `.png`
- Expressive GIF output is saved as looping `.gif`
- Expressive audio output is saved as `.wav`
- Realism image output is saved as `.png`
- Realism GIF output is saved as `.gif`
- Realism true video output is saved as `.mp4` for the families that support real local video
- Realism audio output is saved as `.wav`
- GIF is still the easiest lightweight animated preview format, but realism video now exports in the more trainer-friendly `.mp4` format
- MP4 export and control-video unpacking depend on local `ffmpeg` and `ffprobe` being available in `PATH`
- If a model returns invalid JSON during planning, Chatty-art falls back to a deterministic local renderer so the job can still finish cleanly.
- Expressive runs now also save raw planner sidecars as `*.planner.json`, and Prompt Assist runs save compiler sidecars as `*.compiler.json`.
