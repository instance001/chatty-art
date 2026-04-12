# Chatty-art

![Chatty-art screenshot](<./Screenshot 2026-03-20 104930.png>)

Chatty-art is a simple local image/GIF/video/audio generator with:

- GGUF auto-detection from `models/`
- Bundled `llama.cpp` runtime from `runtime/`
- Local `stable-diffusion.cpp` realism backend sourced from `diffuse_runtime/`
- Plain HTML/CSS/JS single-page dashboard
- Rust backend with WebSocket progress updates
- Auto-save into `outputs/`
- Optional reference media selection from `input/`
- Input Tray `Use as Guide` / `Edit Selected` controls for reference-driven runs
- Separate `Generate GIF` and `Generate Video` paths with video resolution, duration, and FPS controls
- `Low VRAM Mode` for safer realism jobs on tighter GPUs
- Live `ECG Window` under the progress area on Windows, similar to Task Manager
- Model-aware `Recommended Limits On This Hardware` guidance in the UI
- Collapsible `Controls`, `Outputs`, and `Input Tray` columns for easier layout management
- Optional `Prompt Assist` compiler that expands short prompts into richer local briefs before generation
- Dedicated audio `Words / Script` or `Words / Sounds` box for realism audio models
- `Basic / Advanced` prompt mode split, so the beginner path stays simple while advanced users get deeper controls
- Advanced audio sequencing with reusable voice/layer names, plus `after last box` or `same time as last box` timing

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
- This avoids more fragile starter choices like random SD3.5 merges, FLUX companion bundles, or Wan2.2 paired-model setups.

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
- The current known-good video recommendation remains `Wan2.1 T2V 14B` from the starter stack while smaller Wan video conversions are still being validated.
- This gives Chatty-art a practical middle tier without jumping straight to the biggest models.

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
- Smaller Wan video conversions such as `VACE 1.3B`
  Experimental for now. Promising, but not yet pinned as the default beginner recommendation.
- `Wan2.2` paired models and heavier multi-part families
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
  - `Words / Script` = exactly what should be said
- soundscape model:
  - `Prompt` = overall scene and texture
  - `Words / Sounds` = literal cue list

Example speech setup:

- `Prompt`
  `warm Australian female voice, calm pacing, clear diction, close microphone, gentle smile`
- `Words / Script`
  `Welcome to Chatty-art. Local generation is ready.`

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

## Run

1. Drop one or more `.gguf` models into `models/`
2. Put optional reference files into:
   - `input/images`
   - `input/video`
   - `input/audio`
3. If you want `Realism` mode, also place any required companion weights into `models/`.
   - `Qwen Image` needs its VAE and Qwen2.5-VL text encoder.
   - `Wan` models need a Wan VAE and a `umt5`/`t5xxl` text encoder.
4. Start the app:

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
- The Input Tray lets you choose whether the selected file should be used as a `Guide` or treated as the image to `Edit`.
- The dashboard columns can be collapsed with `Hide` and restored from the bottom-right dock as `Controls`, `Outputs`, and `Input Tray`.
- In realism mode, image references currently come from `input/images/`.
- Realism audio uses specialist backends alongside the realism visual lane:
  - `OuteTTS` for speech
  - `Stable Audio Open` for soundscapes / SFX
- Advanced audio currently stays within one backend at a time.
  - `OuteTTS` handles multi-segment speech
  - `Stable Audio Open` handles multi-segment sound layers
  - Chatty-art does not yet merge speech and sound backends into one combined audio job
- `Prompt Assist` can be set to `Off`, `Gentle`, or `Strong`.
- Prompt Assist uses a local expressive `llama.cpp` model as an interpreter role before generation.
- For realism speech models, Prompt Assist now separates spoken words from delivery direction.
- For realism sound models, Prompt Assist only expands the descriptive prompt and negative prompt.
- For realism audio models, the `Words / Script` or `Words / Sounds` field is the best place for verbatim content.
- `Generate GIF` and `Generate Video` are separate on purpose. GIF is usually the easier preview/share format, while true local video depends on the selected realism family.
- GIF/video settings include clip resolution, duration, and FPS.
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
- Realism true video output is saved as `.avi` for the families that support real local video
- Realism audio output is saved as `.wav`
- Some browsers do not preview `.avi` inline cleanly, so GIF is still the easiest animated preview format
- If a model returns invalid JSON during planning, Chatty-art falls back to a deterministic local renderer so the job can still finish cleanly.
- Expressive runs now also save raw planner sidecars as `*.planner.json`, and Prompt Assist runs save compiler sidecars as `*.compiler.json`.
