# Chatty-art User Manual

This guide is written for someone who has never used a local AI tool before.

You do not need to know Rust, coding, or machine learning terms to use Chatty-art. You only need to:

1. Put a model file in the right folder.
2. Start the app.
3. Type what you want.
4. Click a button.

## What Chatty-art does

Chatty-art is a local media generator.

- It can make an image.
- It can make a looping GIF.
- Some realism models can also make a short local video file.
- It can make an audio file.
- It runs on your computer.
- It saves everything into the `outputs/` folder automatically.
- It can show a small live `ECG Window` during generation on Windows.
- It can show recommended hardware-safe limits for the model you selected.
- It lets you collapse and reopen the `Controls`, `Outputs`, and `Input Tray` columns.

Chatty-art uses local GGUF model files. A GGUF file is just the AI model file you place in the `models/` folder.

Chatty-art now has two generation modes:

- `Expressive`
  Uses the built-in local `llama.cpp` + renderer workflow.

- `Realism`
  Uses local specialist backends:
  - `stable-diffusion.cpp` for diffusion-style image, GIF, or video workflows
  - `OuteTTS` for realism speech / voice output
  - `Stable Audio Open` for realism soundscape / SFX style audio

## Project license

Chatty-art's own code and documentation are licensed under the GNU Affero General Public License version 3 or later (`AGPLv3-or-later`).

Simple version:

- If you share or redistribute Chatty-art, keep the AGPL license with it.
- If you modify Chatty-art and let other people use it over a network, the AGPL expects you to provide the corresponding source for your modified version.
- Third-party runtimes that Chatty-art uses, such as `llama.cpp` and `stable-diffusion.cpp`, keep their own upstream licenses. Chatty-art being AGPL does not erase those separate notices.

If you want the full legal text, read the `LICENSE` file in the project folder.

## What you need before you start

You need:

- This project folder on your computer
- At least one `.gguf` model file
- Rust installed, because this app starts with `cargo run`
- The `diffuse_runtime/` source folder, if you want to use `Realism` mode

If you already ran the app once, the folder structure is already ready for you.

## Important folders

These folders matter most:

- `models/`
  Put your `.gguf` model files here.
  Some realism models also need extra local support files here, such as `.safetensors` or helper `.gguf` text encoders.

- `input/images/`
  Put pictures here if you want to use a picture as a reference.

- `input/video/`
  Put video files here if you want to use a video as a reference.

- `input/audio/`
  Put sound files here if you want to use an audio file as a reference.

- `outputs/image/`
  Saved images go here.
  These also appear in the Input Tray under `Output Folder`.

- `outputs/gif/`
  Saved GIF files go here.
  These also appear in the Input Tray under `Output Folder`.

- `outputs/video/`
  Saved video files go here.
  These also appear in the Input Tray under `Output Folder`.

- `outputs/audio/`
  Saved audio files go here.
  These also appear in the Input Tray under `Output Folder`.

- `runtime/`
  This already contains the bundled local runtime. You do not need to change it.

- `diffuse_runtime/`
  This contains the local `stable-diffusion.cpp` source used by `Realism` mode. Chatty-art builds `sd-cli` from here the first time you use realism mode.

## First-time setup

Follow these steps exactly:

1. Open the Chatty-art folder.
2. Copy at least one `.gguf` model into `models/`.
3. If you want to use an outside reference file, copy it into one of the `input/` subfolders.
   Files you generate later will also appear automatically in the tray under `Output Folder`, so you do not need to move them back into `input/`.
4. If you want `Realism` mode, make sure `diffuse_runtime/` is present.
5. If your realism model needs extra support files, copy those into `models/` too.
   - `Qwen Image` needs `qwen_image_vae.safetensors` and a Qwen2.5-VL text encoder.
   - `Wan` models need a Wan VAE plus a `umt5` or `t5xxl` text encoder.
   - Some reference-driven Wan variants also need `clip_vision_h.safetensors`.
6. Start the app with one of these:

```powershell
cargo run
```

or

```powershell
.\launch-chatty-art.ps1
```

7. Wait for your browser to open to `http://127.0.0.1:7878`.

If the browser does not open by itself, open that address manually.

## Understanding model types

Not every `.gguf` is the same kind of model.

In Chatty-art, there are two big families:

- `Expressive`
  Uses regular `llama.cpp` text GGUFs.
  These are language models. Chatty-art uses them to plan and render local image, GIF, or audio output.

- `Realism`
  Uses `stable-diffusion.cpp`.
  These are diffusion-style image or video models. They are the better choice when you want more literal or photoreal-looking results.

Simple rule:

- If it is a normal chat or reasoning GGUF, use `Expressive`.
- If it is a Stable Diffusion, FLUX, Qwen Image, Wan, or similar visual GGUF, use `Realism`.

## Recommended starter stack

If you want the easiest up-to-date `Realism` setup, start with this exact download set and ignore everything more advanced until this works:

1. a full `stable-diffusion.cpp` checkout with submodules
2. `stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf`
3. `v2-1_768-nonema-pruned-Q4_0.gguf`
4. `wan2.1-t2v-14b-Q4_K_M.gguf`
5. `wan_2.1_vae.safetensors`
6. `umt5-xxl-encoder-Q4_K_M.gguf`

Exact links:

1. Runtime source:
   - Project page:
     https://github.com/leejet/stable-diffusion.cpp
   - Releases page:
     https://github.com/leejet/stable-diffusion.cpp/releases
   - Recommended setup command:

```powershell
git clone --recurse-submodules https://github.com/leejet/stable-diffusion.cpp diffuse_runtime
```

2. `stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf`
   - Model page:
     https://huggingface.co/second-state/stable-diffusion-v1-5-GGUF
   - Direct file:
     https://huggingface.co/second-state/stable-diffusion-v1-5-GGUF/resolve/main/stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf

3. `v2-1_768-nonema-pruned-Q4_0.gguf`
   - Model page:
     https://huggingface.co/second-state/stable-diffusion-2-1-GGUF
   - Direct file:
     https://huggingface.co/second-state/stable-diffusion-2-1-GGUF/resolve/main/v2-1_768-nonema-pruned-Q4_0.gguf

4. `wan2.1-t2v-14b-Q4_K_M.gguf`
   - Model page:
     https://huggingface.co/city96/Wan2.1-T2V-14B-gguf
   - Direct file:
     https://huggingface.co/city96/Wan2.1-T2V-14B-gguf/resolve/main/wan2.1-t2v-14b-Q4_K_M.gguf

5. `wan_2.1_vae.safetensors`
   - Model page:
     https://huggingface.co/Comfy-Org/Wan_2.1_ComfyUI_repackaged
   - Direct file:
     https://huggingface.co/Comfy-Org/Wan_2.1_ComfyUI_repackaged/resolve/main/split_files/vae/wan_2.1_vae.safetensors

6. `umt5-xxl-encoder-Q4_K_M.gguf`
   - Model page:
     https://huggingface.co/city96/umt5-xxl-encoder-gguf
   - Direct file:
     https://huggingface.co/city96/umt5-xxl-encoder-gguf/resolve/main/umt5-xxl-encoder-Q4_K_M.gguf

Why this is the recommended beginner stack:

- `SD1.5` and `SD2.1` are simple image models that are easy to test.
- `Wan2.1 T2V` is a good current local text-to-video family for consumer PCs.
- The Wan VAE plus `umt5` encoder are the minimum extra files that make that video path usable.
- This avoids common beginner traps like mismatched Wan parts, random SD3.5 merge GGUFs, and larger multi-part image families that need more helper files.

Where they go:

- Put a full `stable-diffusion.cpp` checkout with submodules into `diffuse_runtime/`.
  The easiest safe way is:

```powershell
git clone --recurse-submodules https://github.com/leejet/stable-diffusion.cpp diffuse_runtime
```

- If you use a downloaded source zip instead, check that this file exists afterward:
  `diffuse_runtime/ggml/CMakeLists.txt`
  If that file is missing, the runtime tree is incomplete and realism models will all show as not ready.
- Put the other 5 files into `models/`.

What not to add yet:

- `FLUX` unless you also want to manage its extra `ae`, `clip_l`, and `t5xxl` files
- `Wan2.2` paired video models
- random `SD3` or `SD3.5` merge GGUFs from pages that do not clearly list the required helper files
- duplicate downloads of the same file with `(1)` in the name

## Recommended upgrade stack

Once the starter stack works, the cleanest next tested image upgrade set is:

1. `flux1-schnell-q4_0.gguf`
2. `ae.safetensors`
3. `clip_l.safetensors`
4. `t5xxl_fp16.safetensors`

Exact links:

1. `flux1-schnell-q4_0.gguf`
   - Model page:
     https://huggingface.co/leejet/FLUX.1-schnell-gguf
   - Direct file:
     https://huggingface.co/leejet/FLUX.1-schnell-gguf/resolve/main/flux1-schnell-q4_0.gguf

2. `ae.safetensors`
   - Model page:
     https://huggingface.co/black-forest-labs/FLUX.1-schnell
   - Direct file:
     https://huggingface.co/black-forest-labs/FLUX.1-schnell/resolve/main/ae.safetensors

3. `clip_l.safetensors`
   - Model page:
     https://huggingface.co/black-forest-labs/FLUX.1-schnell
   - Direct file:
     https://huggingface.co/black-forest-labs/FLUX.1-schnell/resolve/main/clip_l.safetensors

4. `t5xxl_fp16.safetensors`
   - Model page:
     https://huggingface.co/black-forest-labs/FLUX.1-schnell
   - Direct file:
     https://huggingface.co/black-forest-labs/FLUX.1-schnell/resolve/main/t5xxl_fp16.safetensors

Keep using these files from the starter stack:

- `wan_2.1_vae.safetensors`
- `umt5-xxl-encoder-Q4_K_M.gguf`
- `stable-diffusion-v1-5-pruned-emaonly-Q4_0.gguf`
- `v2-1_768-nonema-pruned-Q4_0.gguf`

Why this is a good next step:

- `FLUX.1-schnell` is a noticeably stronger image model than `SD1.5` or `SD2.1`.
- This gives you a practical image-quality upgrade without changing the current known-good video stack.

Important note:

- We are still testing which smaller Wan video conversion is the best beginner recommendation for Chatty-art.
- Until that is pinned down, the current known-good video recommendation stays `Wan2.1 T2V 14B` from the starter stack.
- If you see older notes elsewhere in this manual mentioning `Wan2.1 VACE 1.3B` as the default next step, treat that as experimental rather than the current recommended download.

## Audio downloads and where they go

The easiest audio path in Chatty-art is still `Expressive`, but the realism audio lanes now work too.

Simple version:

- `Expressive` audio = easiest quick local WAV output
- `Realism` speech = `OuteTTS`
- `Realism` soundscape / SFX = `Stable Audio Open`

### Recommended audio downloads to keep on hand

#### OuteTTS speech GGUFs

These are the cleanest first audio models to stage for local speech-style realism audio.

1. `OuteTTS-1.0-0.6B-FP16.gguf`
   - Model page:
     https://huggingface.co/OuteAI/OuteTTS-1.0-0.6B-GGUF
   - Direct file:
     https://huggingface.co/OuteAI/OuteTTS-1.0-0.6B-GGUF/resolve/main/OuteTTS-1.0-0.6B-FP16.gguf

2. `Llama-OuteTTS-1.0-1B-FP16.gguf`
   - Model page:
     https://huggingface.co/OuteAI/Llama-OuteTTS-1.0-1B-GGUF
   - Direct file:
     https://huggingface.co/OuteAI/Llama-OuteTTS-1.0-1B-GGUF/resolve/main/Llama-OuteTTS-1.0-1B-FP16.gguf

3. `OuteTTS` runtime source
   - Project page:
     https://github.com/edwko/OuteTTS

Where they go:

- Put the `.gguf` files into `models/`
- Keep any downloaded runtime/source tree out of `models/`
- A good place for the source tree is:
  `audio_runtime/outetts/`

#### Stable Audio Open package

This is not a one-file GGUF. It is a model package that needs to stay together as a folder.

1. `stable-audio-open-1.0`
   - Model page:
     https://huggingface.co/stabilityai/stable-audio-open-1.0

2. `stable-audio-tools` runtime source
   - Project page:
     https://github.com/Stability-AI/stable-audio-tools

Where it goes:

- Keep the package together as:
  `models/stable-audio-open-1.0/`
- Keep the runtime/source code out of `models/`
- A good place for the source tree is:
  `audio_runtime/stable_audio_tools/`

Plain-language rule:

- one `.gguf` file -> usually place it directly in `models/`
- a model package with folders like `tokenizer/`, `transformer/`, `vae/`, `scheduler/` -> keep that whole folder together inside `models/`
- source code zip or GitHub repo -> do not place that in `models/`

If you use the Hugging Face CLI for Stable Audio, the easiest pattern is:

```powershell
hf auth login
hf download stabilityai/stable-audio-open-1.0 --local-dir stable-audio-open-1.0
```

Then move the finished `stable-audio-open-1.0` folder into `models/`.

Important:

- `OuteTTS` is the cleaner first target for speech-style audio
- `stable-audio-open-1.0` is a heavier realism soundscape package, so first runs may be slower than `OuteTTS`
- neither of these audio downloads is required for the current image/video realism starter stack
- if you are brand new and just want a working output right now, `Expressive` audio is still the easy starting point

## Quality and hardware gradient

Think of the realism models like a ladder.

### Lowest setup friction

- `stable-diffusion-v1-5-pruned-emaonly`
- `stable-diffusion-v2-1`

These are the easiest realism image models.

- Best for: first tests, lower-risk setup, weaker hardware
- Output style: realism image and GIF
- Tradeoff: easier to run, but less capable than newer image families

### Middle ground image

- `FLUX.1-schnell`

This is the best current "stronger image model" upgrade for Chatty-art.

- Best for: better realism image quality on a consumer PC
- Output style: strong realism image and GIF generation
- Tradeoff: much better than `SD1.5` / `SD2.1`, but needs extra helper files

### Heavier starter video

- `Wan2.1-T2V-14B`

This is the current known-good video recommendation for Chatty-art, but it is still a heavier realism path.

- Best for: users who specifically want stronger text-to-video output
- Output style: image, GIF, and video workflows
- Tradeoff: more VRAM pressure and slower runs

### Experimental lighter video path

- `Wan2.1 VACE 1.3B`

This may become the better "middle ground" video option later, but we are not pinning one exact file as the default beginner recommendation yet.

- Best for: advanced users who are comfortable testing alternative conversions
- Output style: image, GIF, and video workflows
- Tradeoff: lighter on paper, but easier to mismatch with helper files or runtime expectations

### Advanced later-stage families

- `Wan2.2` paired models
- larger `SD3` / `SD3.5` setups
- heavier multi-part image families

These are not bad models, but they are not the easiest place to start.

- Best for: later experimentation after the simpler stacks already work
- Tradeoff: more files, more chance of mismatches, and more hardware pressure

## The main model types

### 1. Regular expressive GGUFs

These are the easiest kind to understand.

- Usually one file: one `.gguf`
- Typical use: `Expressive` mode
- Best for: simple local image, looping GIF, or WAV audio without extra setup
- Output types in Chatty-art: `Image`, `GIF`, `Audio`

Common examples:

- `Qwen3-8B GGUF`
- `gpt-oss-20b GGUF`
- `QwQ-32B GGUF`

Important:

- These are not diffusion image models.
- They work through Chatty-art's planner + renderer pipeline.
- They are more stylized and abstract than realism mode.

### 2. All-in-one realism GGUFs

These are the easiest realism models.

- Usually one file: one `.gguf`
- Typical use: `Realism` mode
- Best for: your first local diffusion test
- Output types in Chatty-art: usually `Image` and `GIF`, and sometimes true `Video` if the family supports it

Common signs:

- The model name often includes `stable-diffusion`
- The page usually says it can run directly as GGUF
- You do not need to download a separate VAE or text encoder just to get started

Beginner examples:

- `stable-diffusion-v1-5-pruned-emaonly GGUF`
- `stable-diffusion-v2-1 GGUF`

Important:

- `Realism` audio uses specialist audio backends rather than one-file diffusion GGUFs
- one-file realism models are the easiest way to test whether your local diffusion runtime is working

### 3. Realism GGUFs that need companion weights

These use a main GGUF plus extra files.

- Main file: one or more `.gguf`
- Extra files: often `.safetensors`, sometimes extra `.gguf` encoder files too
- Typical use: `Realism` mode
- Output types in Chatty-art: depends on the model family, usually `Image + GIF`, sometimes `Image + GIF + Video`

Common extra files you may see mentioned:

- `vae`
- `llm`
- `t5xxl`
- `umt5`
- `clip_l`
- `clip_g`
- `mmproj`
- `clip_vision_h`

Plain-language meaning:

- The main GGUF is not enough by itself
- the helper files must also be copied into `models/`
- if Chatty-art says support files are missing, this is the kind of model you are dealing with

Common families:

- `Qwen Image`
- `Qwen Image Edit`
- `Wan`
- `FLUX`
- `FLUX Kontext`
- `Z-Image`
- `Ovis-Image`
- `Chroma`
- `Anima`

### 4. Split or paired video models

Some realism video models are more advanced and may use:

- a low-noise model plus a high-noise model
- a VAE
- a text encoder
- sometimes an extra image or vision encoder for reference-driven generation

These are still local models, but they are not "drop in one file and forget it" models.

The most common family here is `Wan`.

That is why a Wan download page may list more than one GGUF and more than one helper weight.

## Which mode can make what

- `Expressive`
  Can make `Image`, `GIF`, and `Audio`

- `Realism`
  Can make `Image`
  Most realism families can also make `GIF`
  Some realism families can also make true `Video`
  Can also make `Audio` through specialist backends:
  - `OuteTTS` for realism speech / voice
  - `Stable Audio Open` for realism soundscape / SFX style output

Simple advice:

- If you want the easiest all-in-one local experience, start with `Expressive`
- If you want more literal or photoreal visuals, use `Realism`
- If you want the quickest simple sound output, use `Expressive`
- If you want realism speech or realism sound design, use `Realism` with the dedicated audio models

## How to tell what you downloaded

If the download page says things like:

- `chat`
- `instruct`
- `reasoning`
- `llama.cpp`
- `8B`, `14B`, `20B`, `32B` language model

that usually means it is a regular expressive GGUF.

If the download page says things like:

- `stable diffusion`
- `diffusion`
- `image model`
- `video model`
- `vae`
- `text encoder`
- `clip`
- `t5xxl`
- `wan`
- `flux`
- `qwen image`

that usually means it belongs in `Realism`.

If the page mentions extra helper files, it is not an all-in-one model.

## Model ideas you can copy into Google

These are search phrases, not magic passwords.

Copy one into Google, then look for a GGUF download page that clearly matches the family you want.

### Easy expressive starters

- `Qwen3-8B GGUF`
- `gpt-oss-20b GGUF`
- `QwQ-32B GGUF`

### Easy realism starters

- `stable-diffusion-v1-5-pruned-emaonly GGUF`
- `stable-diffusion-v2-1 GGUF`
- `Wan2.1-T2V-14B GGUF`
- `wan_2.1_vae.safetensors`
- `umt5-xxl-encoder GGUF`

These are the best current beginner-friendly realism downloads for Chatty-art.

Important:

- `SD1.5` and `SD2.1` are self-contained image models.
- `Wan2.1-T2V-14B` is not self-contained. It also needs `wan_2.1_vae.safetensors` and a `umt5` encoder.
- `umt5-xxl-encoder` is a helper file, not a model you normally pick by itself.

### Middle-ground upgrades

- `FLUX.1-schnell GGUF`
- `ae.safetensors`
- `clip_l.safetensors`
- `t5xxl_fp16.safetensors`

Important:

- `FLUX.1-schnell` is a stronger image option than `SD1.5` and `SD2.1`.
- `FLUX.1-schnell` needs `ae`, `clip_l`, and `t5xxl_fp16`.
- Smaller Wan video conversions are still experimental here. The current known-good video starter is still `Wan2.1-T2V-14B`.

### Realism image models that usually need helper files

- `FLUX.1-schnell GGUF`
- `Qwen-Image GGUF`
- `Qwen-Image-Edit GGUF`
- `FLUX.1-Kontext-dev GGUF`
- `Z-Image-Turbo GGUF`
- `Z-Image GGUF`
- `Ovis-Image-7B GGUF`
- `Chroma GGUF`
- `Chroma1-Radiance GGUF`
- `Anima GGUF`
- `Anima2 GGUF`

### Realism video families that usually need helper files

- `Wan2.1-T2V-14B GGUF`
- `Wan2.1-I2V-14B GGUF`
- `Wan2.1_14B_VACE GGUF`
- `Wan2.2-TI2V-5B GGUF`
- `Wan2.2-T2V-A14B GGUF`
- `Wan2.2-I2V-A14B GGUF`

Beginner advice:

- start with `Wan2.1-T2V-14B`
- add `Wan2.1-I2V-14B` later if you want image-to-video
- leave `Wan2.2` for later, because paired-model setups are easier to mismatch

## Search tips that save time

When you search for a model, add the words:

- `GGUF`
- `stable-diffusion.cpp` for realism models

Good example searches:

- `stable-diffusion-v1-5-pruned-emaonly GGUF stable-diffusion.cpp`
- `stable-diffusion-v2-1 GGUF stable-diffusion.cpp`
- `Wan2.1-T2V-14B GGUF stable-diffusion.cpp`
- `umt5-xxl-encoder GGUF stable-diffusion.cpp`
- `wan_2.1_vae.safetensors stable-diffusion.cpp`
- `Qwen3-8B GGUF llama.cpp`

Avoid pages that only offer:

- `.safetensors` with no GGUF option, unless you already know you want to do your own conversion
- incomplete downloads that do not explain the required helper files

If you are brand new, start with:

1. one regular expressive GGUF
2. `stable-diffusion-v1-5-pruned-emaonly GGUF`
3. `stable-diffusion-v2-1 GGUF`
4. `Wan2.1-T2V-14B GGUF` plus its `wan_2.1_vae` and `umt5` helper files

That gives you the easiest current first success in both modes and a simple path into local video.

## Your first generation

Once the page opens:

1. Find the `Prompt` box.
2. Choose a mode:
   - `Expressive` for the built-in fast local renderer
   - `Realism` for local diffusion-style visuals plus the specialist realism audio backends
3. Type what you want to create.
   - If you picked a realism audio model, you can also fill in the extra `Words / Script` or `Words / Sounds` box.
4. Pick a model from the dropdown.
5. Leave the default settings alone for your first test.
6. Click one button:
   - `Generate Image`
   - `Generate GIF`
   - `Generate Video`
   - `Generate Audio`
7. Watch the progress bar.
   - On Windows, you can also watch the small `ECG Window` under the progress area. It is meant to feel similar to the Task Manager GPU graph.
8. When it finishes, the result appears in the preview area.
9. The file is also saved in `outputs/`.

## Helpful dashboard features

Chatty-art now has a few helper features that make it easier to stay inside the safe range for your current machine.

- `Recommended Limits On This Hardware`
  The model card shows a quick guide for what is usually safe, stretch, or risky on your detected hardware for the currently selected model and output type.

- `Low VRAM Mode`
  In `Realism` mode, this uses a more conservative runtime profile. It is slower, but it can save jobs that would otherwise fail on GPUs with limited dedicated VRAM.

- `ECG Window`
  On Windows, the progress area can show a little live GPU graph during generation. It is an ECG-style "heartbeat" view of local GPU activity, similar to the Task Manager performance graph.

- Advanced realism controls
  In `Realism + Advanced`, Chatty-art can show extra tuning controls like `Sampler`, `Scheduler`, `Reference Strength`, `Flow Shift`, `Manual Focus Cues`, `Manual Defaults / Assumptions`, and family-aware `LoRA` controls when the selected model family supports them.

- Collapsible columns
  The `Controls`, `Outputs`, and `Input Tray` columns can be hidden with `Hide` and brought back from the small button dock near the bottom-right of the app.

## How to write a prompt

A prompt is simply a text description of what you want.

Good prompt example:

`A glowing retro robot standing in orange fog with strong poster-style shapes`

Good prompt example:

`A calm nighttime ocean loop with teal waves and soft golden highlights`

Good prompt example:

`A warm synth soundscape for a sunrise over a quiet city`

If the output feels vague, add:

- mood
- color
- style
- subject
- lighting
- motion words for GIF or video
- sound words for audio

## Advanced realism controls

`Basic` mode is still the best place to start.

But if you switch to `Realism + Advanced`, Chatty-art can show extra realism controls for models that support them.

These controls are optional. They are meant for experimenting, not for the first run.

### Sampler

This changes the main sampling method used by the realism backend.

Simple advice:

- if you do not know what this means, leave it on `Euler`
- change it only when you are intentionally comparing realism behavior

### Scheduler

This changes how noise is distributed across the run.

Simple advice:

- leave it on `Auto / Runtime Default` unless you are intentionally testing combinations
- if you changed the sampler and want to experiment further, change one setting at a time

### Reference Strength

This only appears when:

- the selected realism model supports still-image reference strength
- and the current workflow is actually using a reference image

Plain-language meaning:

- higher = stay closer to the reference image
- lower = let the model drift further away and reinterpret more freely

This is especially useful when you are using a guide/edit image and want more or less freedom.

### Flow Shift

This only appears for model families that use it, especially flow-style families like `Wan` and `Qwen`.

Plain-language meaning:

- this is an advanced tuning control for those families
- if you are not intentionally experimenting, leave it at the default

### Manual Focus Cues

This gives you a place to type your own short visual steering cues directly into the realism handoff.

Plain-language meaning:

- use this box for important visual ideas you do not want the handoff to miss
- think in short cue phrases, not long sentences
- Chatty-art pushes these cues into the prepared realism prompt even if Prompt Assist is off

Good examples:

- `golden hour`
- `shallow depth of field`
- `wet pavement`
- `cinematic framing`
- `backlit portrait`

When to use it:

- Prompt Assist missed something important
- you want more control without rewriting the whole prompt
- you know the exact visual emphasis you want

### Manual Defaults / Assumptions

This gives you a place to type the sensible defaults you want the realism handoff to assume.

Plain-language meaning:

- use this box for concrete background assumptions
- these are the details you want Chatty-art to treat as true unless your prompt already says otherwise

Good examples:

- `adult woman`
- `modern city street`
- `stormy coast`
- `overcast afternoon`
- `wide-angle photo`

When to use it:

- your prompt is short but you already know a few key defaults
- you do not want Prompt Assist making those decisions for you
- you want the handoff to stay more grounded and specific

### LoRA

This only appears when:

- the selected realism model family supports LoRAs
- and you have compatible LoRA files available locally

Where LoRA files go:

- `models/loras/flux/`
- `models/loras/sd/`
- `models/loras/sd3/`
- `models/loras/wan/`

Supported LoRA file types:

- `.safetensors`
- `.ckpt`

Plain-language meaning:

- a LoRA is a small add-on that nudges the base model toward a style, subject, look, or behavior
- Chatty-art currently supports one LoRA at a time in `Realism + Advanced`
- Chatty-art only shows LoRAs that match the selected model family

### LoRA Weight

Plain-language meaning:

- lower = gentler effect
- higher = stronger LoRA influence

Beginner advice:

- start around `1.0`
- if the LoRA is overpowering the image, lower it
- if the LoRA is barely doing anything, raise it slowly

### Beginner rule

If you are new:

- use `Basic` first
- only move to `Realism + Advanced` when you actually want to test a change
- start with:
  - `Sampler = Euler`
  - `Scheduler = Auto / Runtime Default`
  - `Reference Strength = default`
  - `Flow Shift = default`
- leave `Manual Focus Cues` and `Manual Defaults / Assumptions` empty unless you know exactly what you want to add
- `LoRA = off` until the base model is already behaving the way you want
- change one thing at a time

That makes it much easier to tell what helped and what made the output worse.

## LoRA guide

This section is for people who have seen the word `LoRA` but do not really know what it means yet.

### What a LoRA is

`LoRA` stands for `Low-Rank Adaptation`.

You do not need to remember the technical words. The useful meaning is:

- a LoRA is a small add-on file
- it changes how a base model behaves
- it can push the model toward a style, subject, character look, lighting feel, outfit style, camera vibe, or other visual bias
- it is not a full replacement for the main model

Simple mental model:

- base model = the main engine
- LoRA = a bolt-on tuning pack

So if you are using:

- a `FLUX` model, the `FLUX` model is still doing the generation
- the LoRA is just nudging it in a more specific direction

### What LoRAs are good for

People usually use LoRAs for things like:

- a specific illustration style
- a particular photo aesthetic
- clothing or fashion styles
- character appearance tendencies
- stronger camera or lighting vibes
- object or environment themes

They are often a fast way to get a look that would otherwise take a lot of prompt trial and error.

### What LoRAs are not

LoRAs are not magic.

They do not:

- fix a bad base model
- turn the wrong model family into the right one
- always work well at high strength
- automatically match every realism model

If the base model is fighting you already, fix that first before adding a LoRA.

### How Chatty-art handles LoRAs

In Chatty-art today:

- LoRAs are available in `Realism + Advanced`
- Chatty-art currently supports one LoRA at a time
- Chatty-art only shows LoRAs that match the selected model family

That means Chatty-art is trying to protect you from obvious mismatches.

### Where LoRA files go

Put LoRAs into one of these folders:

- `models/loras/flux/`
- `models/loras/sd/`
- `models/loras/sd3/`
- `models/loras/wan/`

Supported file types:

- `.safetensors`
- `.ckpt`

You do not put LoRAs loose in the project root.
You also do not mix all families in one anonymous folder if you can help it.

### Matching the right LoRA to the right base model

This is the part that trips people up most often.

Use:

- `FLUX` LoRAs with `FLUX` models
- `Stable Diffusion 1.x / 2.x` style LoRAs with the general `sd` family
- `SD3 / SD3.5` LoRAs with `sd3`
- `Wan` LoRAs with `wan`

If the family does not match, one of these usually happens:

- the LoRA does not appear in the dropdown
- the output looks broken or weak
- the LoRA simply does not meaningfully affect the result

### Tips for finding LoRAs

The easiest beginner search formula is:

- model family name
- plus `LoRA`
- plus `safetensors`

Examples:

- `FLUX LoRA safetensors`
- `Stable Diffusion 1.5 LoRA safetensors`
- `SD3 LoRA safetensors`
- `Wan LoRA safetensors`

What to look for on the model page:

- does it clearly say what base family it was trained for?
- are the example images in the same ecosystem as the model you are using?
- does it look like a real LoRA file, not a full checkpoint?

Good beginner rule:

- if the page does not clearly say what family it belongs to, skip it
- if it seems made for a different family than your model, skip it
- if it looks confusing, keep browsing until you find a clearer one

### Beginner way to test a LoRA

The safest test flow is:

1. Get the base model working first with no LoRA.
2. Generate a result you already roughly like.
3. Turn on one LoRA.
4. Leave the other advanced controls alone.
5. Start around `LoRA Weight = 1.0`.
6. Compare before and after.

If the LoRA is too strong:

- lower the weight

If the LoRA is too weak:

- raise the weight slowly

### Signs a LoRA is a bad fit

Common warning signs:

- the output becomes muddy or overcooked
- everything starts looking like the same strange style
- faces, anatomy, or objects get worse instead of better
- the LoRA effect is barely noticeable even at higher weights

If that happens:

- turn the LoRA off
- confirm the base model is still healthy
- try a different LoRA
- or try a LoRA from the correct family

## Audio prompt workflow

When you select a realism audio model, Chatty-art can work in two prompt modes:

- `Basic`
  The simple beginner path. You get the normal audio prompt boxes and can generate quickly.

- `Advanced`
  The power-user path. You can add multiple timed audio boxes so a job can become a sequence instead of one single literal line.

In `Basic`, Chatty-art can show three separate prompt boxes:

- `Prompt`
  This is the descriptive direction box. Use it for tone, mood, pacing, texture, style, environment, and how the result should feel.

- `Negative Prompt`
  This is where you say what you do not want.

- `Words / Script` or `Words / Sounds`
  This is the literal box. Use it for exact spoken words or direct sound cues that should be preserved more directly.

For realism speech models, the tray can also show:

- `Use as Voice Reference`
  Choose an audio clip from either `Input Folder` or `Output Folder`, then assign it as the voice reference.
  Chatty-art will hand that audio file to `OuteTTS` as the cloning reference for the generated speech.

In `Advanced`, the literal box can expand into a sequence builder:

- click `Add new prompt box` to add another speech or sound segment
- use the `X` in the top-right of a box to remove it
- choose whether a box starts `after last box` or `same time as last box`
- give each box a reusable name so Chatty-art can keep a stable identity

Plain-language meaning:

- `Prompt` = how it should sound
- `Negative Prompt` = what to avoid
- `Words / Script` = exactly what should be spoken
- `Words / Sounds` = literal ingredient list of sound cues
- `Voice Reference` = whose voice to imitate

### Basic vs Advanced

Use `Basic` if:

- you want the easiest path
- you only need one spoken line or one sound prompt
- you are still learning the workflow

Use `Advanced` if:

- you want a conversation with multiple turns
- you want two voices or sound layers to overlap
- you want the same voice or layer to return later in the sequence
- you want stronger control over timing

Beginner advice:

- Start in `Basic`
- Move to `Advanced` only when you actually need multiple segments
- Reuse the same name if you want the same identity to come back
- `Preview Handoff` is available in both `Basic` and `Advanced`, so you do not need to enter advanced mode just to review the request before generation

### For OuteTTS speech models

Use:

- `Prompt` for delivery direction
- `Words / Script` for the exact line to say
- `Voice Reference` for an audio clip whose voice you want OuteTTS to imitate

Good beginner example:

- `Prompt`
  `warm Australian female voice, calm pacing, clear diction, friendly smile, close microphone`
- `Words / Script`
  `Welcome to Chatty-art. Everything is running locally on this machine.`

Good beginner example with cloning:

- `Prompt`
  `calm male narration, clear pacing, warm tone, slight radio texture`
- `Words / Script`
  `The local generation run is complete.`
- `Voice Reference`
  `short prerecorded voice clip from the tray`

Simple advice:

- Put the exact spoken sentence in `Words / Script`
- Put voice, tone, speed, mood, and delivery notes in `Prompt`
- Put the speaker you want copied in `Voice Reference`
- Use `Negative Prompt` for things like robotic delivery, harsh sibilance, mumbling, noisy background, or clipping

In `Advanced` mode:

- each box becomes one speech segment
- `Voice Name / Character Note` is the identity field
- reusing the same voice name tells Chatty-art to keep the same character-like voice identity across those segments
- using a different voice name tells Chatty-art to make a different stable voice identity
- `same time as last box` overlaps the speech with the previous box
- `after last box` plays it after the previous segment ends

Simple example:

- Box 1
  - `Voice Name / Character Note`: `John`
  - timing: `after last box`
  - `Words / Script`: `Hello there.`
- Box 2
  - `Voice Name / Character Note`: `Jane`
  - timing: `after last box`
  - `Words / Script`: `Hi John.`
- Box 3
  - `Voice Name / Character Note`: `John`
  - timing: `same time as last box`
  - `Words / Script`: `Wait, listen.`

### For Stable Audio Open sound models

Use:

- `Prompt` for the overall scene and texture
- `Words / Sounds` for literal cues you want preserved more directly

Good beginner example:

- `Prompt`
  `quiet nighttime forest ambience, cinematic depth, soft wind, natural field recording`
- `Words / Sounds`
  `distant owl, dry leaves, soft wind, creek water`

Simple advice:

- Think of `Words / Sounds` as the literal ingredient list
- Think of `Prompt` as the mixing, atmosphere, and style direction
- Use `Negative Prompt` for things like distortion, harsh static, crowd noise, clipping, or artificial digital buzz

In `Advanced` mode:

- each box becomes one sound layer or timed event
- `Layer Name / Sound Note` is the identity field
- reusing the same layer name tells Chatty-art to keep the same seeded sound identity across those segments
- using a different layer name tells Chatty-art to make a different stable layer identity
- `same time as last box` overlaps the sound with the previous box
- `after last box` starts it after the previous segment ends

Simple example:

- Box 1
  - `Layer Name / Sound Note`: `Rain Bed`
  - timing: `after last box`
  - `Words / Sounds`: `steady rain, soft roof patter`
- Box 2
  - `Layer Name / Sound Note`: `Thunder Hit`
  - timing: `same time as last box`
  - `Words / Sounds`: `distant thunder crack`
- Box 3
  - `Layer Name / Sound Note`: `Rain Bed`
  - timing: `after last box`
  - `Words / Sounds`: `steady rain, soft roof patter`

### If you only fill in one box

- For OuteTTS, `Words / Script` alone is enough to make it speak
- For Stable Audio Open, `Words / Sounds` alone is enough to give it direct sound cues
- Using both usually gives the best control
- If you are unsure, keep the literal box short and clear, and keep the main `Prompt` focused on mood and quality

### Preview Handoff for audio

Before you generate, the `Preview Handoff` panel can show how Chatty-art is preparing the request.

For speech models, look for:

- `Prepared Spoken Text`
- `Speech Direction`
- `Voice reference` if you assigned a cloning clip from the tray

For sound models, look for:

- the compiled prompt
- the effective negative prompt
- the literal `Words / Sounds` cues shown as their own separate lane

In `Advanced` mode, the handoff is still per backend:

- `OuteTTS` prepares a speech sequence
- `Stable Audio Open` prepares a sound sequence

Right now Chatty-art does **not** merge those two backends into one combined audio render. Speech and sound stay in their own specialist lanes for now.

## Preview Handoff for realism images and video

In visual `Realism` mode, the `Preview Handoff` panel is where you can check what Chatty-art is actually about to send to the model.

This is especially useful in `Realism + Advanced`.

Look for:

- the prepared prompt
- the effective negative prompt
- any `Manual Focus Cues` you added
- any `Manual Defaults / Assumptions` you added
- the note explaining what was added to the handoff

Simple advice:

- if the handoff looks wrong, do not generate yet
- fix the prompt or your manual cue/default boxes first
- use `Manual Focus Cues` for visual priorities
- use `Manual Defaults / Assumptions` for concrete background facts

## Prompt Assist

Chatty-art now includes `Prompt Assist`.

This is a local helper stage that can expand a short human prompt into a richer brief before generation starts.

Examples:

- `Off`
  Uses only exactly what you typed.

- `Gentle`
  Keeps your idea mostly unchanged and fills in a few missing details.

- `Strong`
  Makes more decisions for you about things like lighting, framing, materials, mood, motion, or ambience.

Simple advice:

- Use `Off` if you already wrote a detailed prompt.
- Use `Gentle` if your prompt is short but you want it to stay close to your wording.
- Use `Strong` if your prompt is very short and you want Chatty-art to do more creative gap-filling for you.

Prompt Assist still runs locally.

It uses a local expressive `llama.cpp` model as an interpreter role before the main generation step.

For realism speech models, Prompt Assist now separates spoken words from delivery direction.

For realism sound models, Prompt Assist only expands the descriptive prompt and negative prompt.

The literal `Words / Script` or `Words / Sounds` boxes stay verbatim and are not rewritten by Prompt Assist.

For realism audio models, the `Words / Script` or `Words / Sounds` field is the best place for verbatim content.

For realism speech models, `Voice Reference` also stays separate from Prompt Assist and is passed through as the cloning clip.

For visual realism in `Advanced`, Prompt Assist also stays separate from your manual handoff controls:

- `Manual Focus Cues` are your own extra visual steering phrases
- `Manual Defaults / Assumptions` are your own explicit defaults

Those boxes are useful when Prompt Assist does not quite give you the handoff you wanted.

## What each setting means

### Temperature

This controls how surprising the result is.

- Low temperature = safer and more predictable
- High temperature = more unusual and experimental

Simple advice:

- Try `0.6` to `0.9` for normal use
- Try `1.2` or higher for stranger results

Important:

- `Temperature` matters in `Expressive` mode
- `Realism` mode mostly ignores it

### Steps

This controls how much planning work the app does.

- Low steps = faster
- High steps = slower, usually richer

Simple advice:

- Try `20` to `35` for normal use
- Use higher values if you want more detail

### CFG Scale

This controls how strictly the output follows your prompt.

- Low CFG = more freedom
- High CFG = sticks closer to your wording

Simple advice:

- Try `7` to `10` for normal use
- Raise it if the result is ignoring your prompt

### Resolution

This controls output size.

- Bigger sizes look sharper
- Bigger sizes also take longer and create larger files

Simple advice:

- Start with `Square 512` or `Square 768`
- Use larger presets once everything is working

### Video Resolution

This controls the size used for `Generate GIF` and `Generate Video`.

- Smaller video sizes are much easier on VRAM
- Video memory use rises very quickly as resolution increases

Simple advice:

- Start with `256x256` for Wan-style video tests
- Move to `512x512` only after smaller tests work
- Treat `768x768` as a heavier setting that can fail on weaker or mid-range hardware

### Video Duration

This controls how many seconds Chatty-art tries to generate for GIF or video output.

- Longer duration = more frames
- More frames = more time and more memory pressure

Simple advice:

- Start with `2s`
- Try `5s` after that
- Treat `10s` and `20s` as advanced/heavier settings

### Video FPS

This controls playback smoothness for GIF or video output.

- Low FPS = fewer frames and less hardware load
- High FPS = smoother motion but more frames to generate

Simple advice:

- Start with `8 FPS`
- Try `16 FPS` once your shorter clips work reliably
- Use `24 FPS` only if your model and hardware are already behaving well

### Low VRAM Mode

This is mainly for `Realism` mode.

- It spills more work to CPU
- It uses a safer low-memory runtime profile
- It is slower, but it can prevent out-of-memory errors on bigger image, GIF, or video jobs

Simple advice:

- Leave it on if realism jobs are failing with VRAM errors
- Try turning it off only after you already know your current model/settings run comfortably on your GPU

### Seed

This is the repeatability number.

- Same prompt + same model + same settings + same seed = same result
- Blank seed = new random result

Use a seed when you want to keep a version you like and reproduce it later.

## Using reference files

If you want to use an existing file during generation:

1. Either:
   - put an outside file into the correct `input/` folder
   - or use a file that already appears under `Output Folder` because Chatty-art generated it earlier
2. Click `Refresh Files` in the app.
3. Open the `Input Tray`.
4. Choose the file from either:
   - `Input Folder`
   - `Output Folder`
5. Choose how Chatty-art should use it:
   - `Use as Guide`
     Good when you want the selected file to act like inspiration, composition guidance, or a soft steering cue.
   - `Edit Selected`
     Good when you want Chatty-art to treat the selected file as the source it should transform.
   - `Use as Voice Reference`
     Good for realism speech models such as `OuteTTS` when you want Chatty-art to clone the voice from a prerecorded audio clip.
   - `Set as End Frame`
     Good for supported video workflows when you want the clip to finish on a particular still image.
   - `Use as Control Video`
     Good for supported motion-guided video workflows. GIFs can also be used here because they count as motion assets in the tray.
6. Generate again.

The assigned file and its current use appear in the tray slots:

- `Primary input` or `Voice reference`
- `End frame`
- `Control video`

You can clear it at any time with the `Clear` button.

Important:

- `Expressive` mode can use the selected file as a guide or edit/source cue during planning.
- `Realism` mode uses tray-selected still images for guide/edit workflows, including files from `input/` and previously generated output images.
- `Realism` speech models like `OuteTTS` can use tray-selected audio files as a `Voice Reference`.
- Some realism video families can also use:
  - `Set as End Frame`
  - `Use as Control Video`

## What gets saved

Chatty-art saves files automatically.

- Expressive images are saved as `.png`
- Expressive GIFs are saved as `.gif`
- Expressive audio is saved as `.wav`
- Realism images are saved as `.png`
- Realism GIFs are saved as `.gif`
- Realism true videos are saved as `.avi`

You do not need to press Save.

Every finished result should appear:

- in the preview panel
- in the Recent Outputs area
- in the correct `outputs/` subfolder on disk

Some runs also save extra sidecar files:

- `*.json`
  Basic output metadata

- `*.planner.json`
  Raw expressive planner output for debugging

- `*.compiler.json`
  Raw Prompt Assist compiler output for debugging

## What kind of output to expect

Chatty-art is designed to be simple and local.

That means:

- image output is rendered locally as artwork
- GIF output is saved as a looping animation
- some realism families can also save a true local video file
- audio output is saved as a WAV file

This tool uses the bundled local `llama.cpp` runtime to plan the output, then renders the final media locally.

In plain language: it is built to be easy and self-contained, not to behave like a full cloud diffusion studio.

Realism mode is different:

- it uses local `stable-diffusion.cpp`
- it is better suited to photoreal or model-specific diffusion/GIF/video workflows
- some advanced GGUFs also need extra local companion weights in `models/`
- browser preview is usually smooth for GIF output, while `.avi` video support depends on the browser

## If the model does not appear in the dropdown

Check these things:

1. Make sure the file ends with `.gguf`
2. Make sure the file is inside `models/`
3. Click `Refresh Files`
4. Restart the app if needed

## If generation fails

Try these fixes in order:

1. Use a shorter prompt
2. Lower `Steps`
3. Try a different model
4. Remove the reference file and try again
5. Restart the app

If you are in `Expressive` mode with a large model such as `gpt-oss-20b`, `qwq`, or other big GGUFs:

- the first planning phase can take a few minutes
- this is local model time, not a cloud token limit
- Chatty-art now sends heartbeat updates during long planning jobs, but the first update can still take a short moment to appear

If you are in `Realism` mode, also check:

- `diffuse_runtime/` exists and includes its `ggml` subfolder
- the first realism run was allowed to build `sd-cli`
- your model's extra support files are also present in `models/`
- `Qwen Image` has its VAE and Qwen2.5-VL text encoder
- `Wan` has its VAE and `umt5`/`t5xxl` text encoder
- some reference-guided Wan variants also need `clip_vision_h.safetensors`

Also check that:

- your model file is not corrupted
- the model is really a GGUF file
- the runtime files in `runtime/` still exist

## If `Realism` mode says a model is missing support files

This means the main GGUF is present, but the extra local files for that model are not.

Common examples:

- `Qwen Image` needs `qwen_image_vae.safetensors`
- `Qwen Image` also needs a Qwen2.5-VL text encoder
- `Wan` needs a Wan VAE
- `Wan` also needs a `umt5` or `t5xxl` text encoder

Try these fixes:

1. Read the model note shown in the dropdown.
2. Put the missing support files into `models/`.
3. Click `Refresh Files`.
4. Try again.

If the first realism run fails before generation starts, Chatty-art may still need to build `sd-cli` from `diffuse_runtime/`.

If that happens, make sure:

- `diffuse_runtime/` exists
- `diffuse_runtime/ggml/` exists
- `cmake` is installed

If you do not want to deal with realism support files right now, switch back to `Expressive` mode instead.

## If the page opens but looks empty

Check that:

- the server is running
- your browser is opening `http://127.0.0.1:7878`
- JavaScript is not blocked in the browser

You can also restart the app and refresh the page.

## If `cargo run` says the app is already running or shows `Access is denied`

This usually means an older copy of Chatty-art is still running in the background.

On Windows, that old copy can keep:

- port `7878` busy
- the `chatty-art.exe` file locked

That can cause errors like:

- `Only one usage of each socket address ... is normally permitted`
- `LNK1104`
- `Access is denied`

Try these steps:

1. Go back to the older terminal window that started Chatty-art.
2. Press `Ctrl+C` to stop it.
3. Run `cargo run` again.

If you cannot find the older terminal window, run this in Command Prompt or PowerShell:

```powershell
taskkill /IM chatty-art.exe /F
```

Then start Chatty-art again with:

```powershell
cargo run
```

Important:

- You usually do **not** need `cargo clean` for this problem.
- Closing the browser tab does **not** stop the app. The terminal window is what keeps it running.
- If port `7878` is busy with another program, Chatty-art may choose a different local port and print the new address in the terminal.

## If you want the same result again

Write down or keep:

- the prompt
- the model
- the Temperature
- the Steps
- the CFG Scale
- the Resolution
- the Seed

Then run it again with the same values.

## Quick start version

If you want the shortest possible version:

1. Put a `.gguf` file in `models/`
2. Run `.\launch-chatty-art.ps1`
3. Open the browser page
4. Type a prompt
5. Pick the model
6. Click a generate button
7. Find the file in `outputs/`

## Final tip

If something is confusing, start with the defaults and change only one setting at a time.

That is the fastest way to learn what each control does.
