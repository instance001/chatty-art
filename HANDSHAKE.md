# Chatty-art - Handshake

## Module identity

- **module_id**: `chatty_art`
- **display_name**: `Chatty-art`

## What this module is for

Chatty-art is the local media-generation department. It is where image, GIF, video, and audio jobs are prepared, run, monitored, and reviewed, including reference-guided edits, prompt-assist expansion, LoRA stacking, and output inspection.

## Inputs this module expects

- A clear generation goal: image, GIF, video, or audio
- Prompt text, spoken text, or sound description
- Optional negative prompt or "avoid" instructions
- Optional guide/edit/reference assets placed into the module's `input/` folders or selected from prior `outputs/`
- Model family choice, quality/speed constraints, and any advanced settings worth preserving
- Any important assumptions about style, realism, duration, resolution, or hardware limits

## Outputs this module produces

- Generated media saved into `outputs/image/`, `outputs/gif/`, `outputs/video/`, or `outputs/audio/`
- A prompt/configured run state that can be resumed or iterated on
- A short record of what model/settings/reference strategy produced the current result
- Follow-up clues about what to tweak next if a run needs another pass
- Optional mediated artifact handoffs to sibling modules or `Chatty_Sandbox` when the user explicitly selects outputs and confirms a destination

## Operating rules / preferences

- Tone: concise, builder-friendly
- Risk level: medium
- Default tags to use in logs: generation, media, prompt, reference, output
- Preferred file naming: mention the saved output folder and the model/style used when relevant

## Suspend rundown template

> **Status:** Current generation lane, model choice, and latest output state are updated.
> **What changed:** Prompt/settings/reference inputs were adjusted and one or more generation attempts were completed, canceled, or prepared.
> **Open questions:** Confirm whether the latest result is approved or which variable should be changed next.
> **Next action:** Re-run with the most likely next tweak, or export/reuse the approved output.
> **Artifacts:** `outputs/image/`, `outputs/gif/`, `outputs/video/`, `outputs/audio/`, plus any important guide assets in `input/`

## Cold log envelope hints

- `module_id`: `chatty_art`
- `event_type`: `suspend_rundown`
- `summary`: one short handoff paragraph focused on active lane, current result, and next tweak
- `tags`: `generation`, `media`, `prompt`, `reference`, `output`
- `payload_json`: optional model, style, lane, and output-path details

## Portable bridge note

This module is being hosted inside ChattyCog as a docked web dashboard. The hosted UI remains Chatty-art's own app; ChattyCog should treat the bridge as optional handoff telemetry rather than as the module's primary state store.

Current hosted bridge lanes / surfaces:
- outgoing artifact handoff requests:
  - `dataset_candidates` toward `chatty_lora`
  - sandbox export toward `chattycog_sandbox`
- incoming artifact inbox:
  - `lora_imports`

Important boundary:
- Chatty-art still owns its real files and generation state
- ChattyCog only mediates approved copy-only handoffs and lightweight bridge summaries
