# Glossary (Repo Excerpt)

For the full glossary, see: https://github.com/instance001/Whatisthisgithub/blob/main/GLOSSARY.md

This file contains only the glossary entries for this repository. Mapping tag legends and global notes live in the full glossary.

## chatty-art
| Term | Alternate term(s) | Alt map | External map | Relation to existing terminology | What it is | What it is not | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Chatty-art | chatty-art | = | ~ | Local-first generative media workbench | Rust backend plus single-page local dashboard for generating images, GIFs, video, and audio from GGUF-backed local workflows; supports bundled `llama.cpp`, optional `stable-diffusion.cpp` realism backend, autosave, and live progress updates | Not a cloud generator; not a hosted model service; not limited to still images | chatty-art/README.md; chatty-art/USER_MANUAL.md |
| Expressive mode | Expressive | = | ~ | Local planner/renderer mode | Built-in local `llama.cpp` plus Chatty-art renderer workflow used for fast all-in-one image, GIF, and audio generation with regular expressive GGUFs | Not the `stable-diffusion.cpp` path; not for diffusion-style realism families | chatty-art/README.md; chatty-art/USER_MANUAL.md |
| Realism mode | Realism | = | ~ | Local diffusion/video backend mode | Local `stable-diffusion.cpp` workflow for diffusion-style image, GIF, or video jobs; can require companion weights and builds `sd-cli` from `diffuse_runtime/` on first use | Not audio generation; not the regular chat/reasoning GGUF path | chatty-art/README.md; chatty-art/USER_MANUAL.md |
| Prompt Assist | prompt compiler, Off/Gentle/Strong | ~ | ~ | Prompt expansion stage | Local helper stage that expands a short prompt into a richer brief before generation, using an expressive `llama.cpp` model as an interpreter; supports `Off`, `Gentle`, and `Strong` strengths | Not a cloud rewrite service; not the final generator model itself | chatty-art/README.md; chatty-art/USER_MANUAL.md |
| Input Tray | guide/edit tray, Use as Guide, Edit Selected | ~ | ~ | Reference media chooser | UI column for selecting reference media from `input/` and marking files as guide or edit sources for reference-driven runs | Not the output gallery; not a permanent asset library | chatty-art/README.md; chatty-art/USER_MANUAL.md |
| Low VRAM Mode | low vram | = | ~ | Memory-saving inference mode | Conservative Realism runtime profile that spills more work to CPU and tiles VAE decode so larger jobs can survive on tighter GPUs | Not faster than the normal profile; not a separate model family | chatty-art/README.md; chatty-art/USER_MANUAL.md |
| Recommended Limits On This Hardware | hardware guidance | ~ | ~ | Hardware-fit heuristic | Model-aware UI guidance showing what is usually safe, stretch, or risky on the detected hardware for the selected model and output type | Not a hard scheduler or guarantee that a run will succeed | chatty-art/README.md; chatty-art/USER_MANUAL.md |
