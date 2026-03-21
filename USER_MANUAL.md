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

Chatty-art uses local GGUF model files. A GGUF file is just the AI model file you place in the `models/` folder.

Chatty-art now has two generation modes:

- `Expressive`
  Uses the built-in local `llama.cpp` + renderer workflow.

- `Realism`
  Uses local `stable-diffusion.cpp` for diffusion-style image, GIF, or video workflows.

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

- `outputs/gif/`
  Saved GIF files go here.

- `outputs/video/`
  Saved video files go here.

- `outputs/audio/`
  Saved audio files go here.

- `runtime/`
  This already contains the bundled local runtime. You do not need to change it.

- `diffuse_runtime/`
  This contains the local `stable-diffusion.cpp` source used by `Realism` mode. Chatty-art builds `sd-cli` from here the first time you use realism mode.

## First-time setup

Follow these steps exactly:

1. Open the Chatty-art folder.
2. Copy at least one `.gguf` model into `models/`.
3. If you want to use a reference file, copy it into one of the `input/` subfolders.
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

- `Realism` mode does not do audio
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
  `Realism` does not currently make `Audio`

Simple advice:

- If you want the easiest all-in-one local experience, start with `Expressive`
- If you want more literal or photoreal visuals, use `Realism`
- If you want sound output, use `Expressive`

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
   - `Realism` for local `stable-diffusion.cpp` diffusion/GIF/video generation
3. Type what you want to create.
4. Pick a model from the dropdown.
5. Leave the default settings alone for your first test.
6. Click one button:
   - `Generate Image`
   - `Generate GIF`
   - `Generate Video`
   - `Generate Audio`
7. Watch the progress bar.
8. When it finishes, the result appears in the preview area.
9. The file is also saved in `outputs/`.

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

### Seed

This is the repeatability number.

- Same prompt + same model + same settings + same seed = same result
- Blank seed = new random result

Use a seed when you want to keep a version you like and reproduce it later.

## Using reference files

If you want to use an existing file during generation:

1. Put the file into the correct `input/` folder.
2. Click `Refresh Files` in the app.
3. Open the `Input Tray`.
4. Click the file you want.
5. Choose how Chatty-art should use it:
   - `Use as Guide`
     Good when you want the selected file to act like inspiration, composition guidance, or a soft steering cue.
   - `Edit Selected`
     Good when you want Chatty-art to treat the selected file as the source it should transform.
6. Generate again.

The selected file and its current use appear in the `Selected reference` area.

You can clear it at any time with the `Clear` button.

Important:

- `Expressive` mode can use the selected file as a guide or edit/source cue during planning.
- `Realism` mode currently uses still images from `input/images/` for these guide/edit workflows.
- `Realism` does not currently use audio files as references.

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
