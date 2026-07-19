# Chatty-art API Key Lanes Design

Design note for adding optional cloud lanes to Chatty-art without weakening its local-first identity.

This document is for planning only.
It does not imply that cloud paths should replace the current local defaults.

## Core stance

Local first.
Cloud optional.
User sovereignty over model choice.

For Chatty-art specifically, that means:

- local generation stays the baseline
- cloud lanes are opt-in and clearly labeled
- no silent fallback from local generation to cloud generation
- the UI should always make it obvious whether cloud is helping prepare a job or generating the final media itself

## Why Chatty-art needs separate lanes

Chatty-art is not a plain chat tool.
It has at least three different kinds of AI work:

1. prompt preparation
2. image/reference interpretation
3. final media generation

Those jobs have different capability needs, different risk profiles, and different user expectations.

Because of that, Chatty-art should not use one flat generic "cloud model" bucket.

It should expose separate lanes:

- Prompt Assist lane
- Vision Assist lane
- Media Generation lane

## Proposed lane split

### 1. Prompt Assist lane

Purpose:

- expand short prompts into richer handoffs
- help draft negative prompts
- split spoken words from delivery direction for audio
- help compile realism handoff text

This lane is assistant-only.
It does not generate the final image, video, GIF, or audio output.

Expected capability shape:

- text input
- text output
- optional structured JSON-friendly output
- no image generation required

Good fit for:

- OpenAI-style text models
- Anthropic text models
- Gemini text models
- xAI Grok text models
- DeepSeek text models
- OpenAI-compatible text endpoints

### 2. Vision Assist lane

Purpose:

- analyze guide/edit reference images
- summarize visible subjects, framing, lighting, palette, and objects
- infer edit intent or preserve/change cues
- feed structured visual context into Prompt Assist or direct handoff prep

This lane is also assistant-only.
It interprets media but does not produce the final output file.

Expected capability shape:

- image input support
- text or JSON output
- multimodal understanding

Good fit for:

- Gemini multimodal models
- OpenAI multimodal models if the chosen endpoint supports image input
- other multimodal OpenAI-compatible endpoints

Notes:

- this lane must be capability-gated harder than Prompt Assist
- not every provider entry that works for chat should appear here

### 3. Media Generation lane

Purpose:

- generate final remote image, video, or audio outputs
- act as an explicit cloud alternative to some local output paths

This is the most sensitive lane because it changes where final media is made.

Expected capability shape:

- provider-specific image, video, or audio generation APIs
- likely async job handling
- likely provider-specific parameters instead of one universal schema

Notes:

- this lane should be introduced last
- it should be visibly separate from the current bundled local backends
- it should never masquerade as the same thing as `llama.cpp`, `stable-diffusion.cpp`, or the local audio runtime

## Product truth we should preserve

Chatty-art currently presents itself as a local media generator with optional local Prompt Assist and local Vision Assist.

That truth should remain intact after cloud support lands.

Recommended product wording:

- local generation is still the default
- cloud assistant lanes can help prepare a job
- cloud generation is a separate optional route
- FMI is not an intermediary inference host
- prompts and reference assets only leave the machine when the user explicitly chooses a cloud lane that needs them

## Recommended rollout order

### Phase 1

Add Prompt Assist cloud lane only.

Why first:

- easiest fit with current architecture
- lowest conceptual risk
- strongest immediate value when the local prompt helper model is weak or missing
- does not disturb the final generation pipeline

### Phase 2

Add Vision Assist cloud lane.

Why second:

- still preparation-only
- useful when local multimodal helpers are missing or weak
- more sensitive than Prompt Assist because reference images may be uploaded

### Phase 3

Add Media Generation cloud lane.

Why last:

- largest product shift
- most provider fragmentation
- biggest UI and persistence consequences
- needs the clearest privacy and output-route disclosure

## Shared architecture recommendation

Use one shared provider/account registry, then bind entries to specific lanes through capability filtering and lane assignment.

That means:

- one place to save provider metadata
- one place to verify provider connectivity
- one place to store secrets
- multiple separate lane selectors

Do not create three unrelated secret systems unless the stack forces it.

## Proposed data model shape

Chatty-art currently does not have a strong app-preferences system comparable to Chatty-cog.
We should add one.

Suggested top-level config file:

- `runtime/config/preferences.json`

Suggested structure:

```json
{
  "api_key_lanes_enabled": false,
  "cloud_providers": [],
  "lane_assignments": {
    "prompt_assist": null,
    "vision_assist": null,
    "media_generation": null
  }
}
```

Suggested provider entry shape:

```json
{
  "id": "provider_123",
  "display_name": "My Gemini",
  "provider_kind": "gemini",
  "base_url": "https://generativelanguage.googleapis.com/v1beta/openai",
  "api_key_ref": "provider_123",
  "chat_model_name": "gemini-3.5-flash",
  "vision_model_name": "gemini-3.5-flash",
  "image_generation_model_name": "",
  "audio_generation_model_name": "",
  "video_generation_model_name": "",
  "enabled": true,
  "capabilities": {
    "text_assist": true,
    "vision_assist": true,
    "image_generation": false,
    "video_generation": false,
    "audio_generation": false
  },
  "verification": {
    "prompt_assist_status": "",
    "vision_assist_status": "",
    "media_generation_status": ""
  }
}
```

Important note:

- the config should store metadata
- secrets should be stored separately if we can do that cleanly

## Secret storage recommendation

Best reference pattern:

- Chatty-mini metadata in app settings
- API keys stored separately in encrypted local storage

For Chatty-art on desktop, the practical options are:

1. plain local file at first, with clear warning
2. Windows Credential Manager or DPAPI-backed storage
3. app-local encrypted blob tied to the current user profile

Recommended direction:

- if we want the cleanest long-term result, use user-profile-backed secret storage
- if we need a faster first pass, store a separate local secrets file and mark it clearly as an upgrade target

What we should avoid if possible:

- storing raw API keys inside the same visible JSON file as normal settings

## Lane assignment model

Each lane should have its own explicit assignment.

Suggested rules:

- `prompt_assist` can be `local_auto`, `local_selected`, or `cloud:<provider_id>`
- `vision_assist` can be `local_auto`, `local_selected`, or `cloud:<provider_id>`
- `media_generation` can be `local_only` or `cloud:<provider_id>`

Why this matters:

- Prompt Assist and Vision Assist already have local selection concepts
- Media Generation is structurally different and should default to local-only

## Capability filtering rules

Provider entries should only appear in selectors for lanes they can actually satisfy.

Examples:

- an Anthropic entry with prompt and vision models can appear in Prompt Assist and Vision Assist
- a generic OpenAI-compatible text entry can appear in Prompt Assist
- an OpenAI-compatible entry backed by an endpoint that accepts image input can appear in Vision Assist
- a multimodal Gemini entry can appear in Prompt Assist and Vision Assist
- an xAI Grok entry can appear in Prompt Assist
- a DeepSeek entry can appear in Prompt Assist
- an image-generation provider can appear in Media Generation image workflows
- a provider with no video support should not appear for remote video generation
- a provider family that is not wired for first-party media generation should not appear in the Media Generation provider inventory just because it exists in the shared registry

## Verification model

Verification should be lane-specific.

Do not treat "provider answered a text ping" as proof that every lane works.

Suggested checks:

### Prompt Assist verification

- simple text round-trip
- optional JSON-shaped response check

### Vision Assist verification

- tiny image analysis prompt using a fixture or a user-approved sample
- confirm the provider's image input path works

### Media Generation verification

- provider-specific dry-run or tiny cheap generation check where practical
- otherwise mark as "configured but unverified"

## Suggested UI placement

Chatty-art already has strong local workflow surfaces.
Cloud lanes should extend those surfaces, not replace them.

Recommended UI structure:

### Settings panel or new "Cloud Lanes" section

Add a dedicated area for:

- provider entries
- API key entry
- provider verification
- lane assignment
- privacy copy

This should not live inside the main prompt box.

### Prompt Assist area

Near the existing Prompt Assist controls:

- show whether Prompt Assist is using local or cloud
- offer an explicit lane selector
- keep the selected-model summary visible as the normal workflow
- keep provider setup in a collapsed-by-default disclosure surface

### Vision Assist area

Near the existing Vision Assist model controls:

- show whether image analysis is using local or cloud
- offer an explicit lane selector
- keep the selected-model summary visible as the normal workflow
- keep provider setup in a collapsed-by-default disclosure surface

### Media generation controls

Near the existing model/runtime area:

- keep local backend messaging intact
- add a separate explicit output route selector if cloud generation exists later
- example: `Local` vs `Cloud`
- keep the selected-model summary visible as the normal workflow
- keep provider setup in a collapsed-by-default disclosure surface

Do not hide cloud generation under the same wording as local model selection.

## Suggested UI copy principles

Keep the copy blunt and honest.

Examples:

- "Local is still the default."
- "This cloud lane sends prompt text directly to the provider you selected."
- "Vision Assist cloud mode may upload the selected reference image to that provider."
- "Cloud generation sends the final media prompt, and any required reference assets, to the selected provider."
- "Chatty-art does not automatically fall back to cloud."

## Backend shape recommendation

Chatty-art already has a good route-based local backend.
The cleanest approach is to add cloud functionality as new service modules rather than burying it in the existing generation handlers.

Suggested additions:

- `src/cloud_ai/`
- `src/preferences.rs` or `src/app_config.rs`
- `src/secrets.rs`

Suggested responsibilities:

### `cloud_ai`

- provider kinds
- capability registry
- lane-specific request adapters
- verification helpers

### `preferences` or `app_config`

- load/save non-secret cloud metadata
- lane assignment persistence

### `secrets`

- read/write/delete API keys
- keep secret handling separate from normal config

## Route ideas

Possible new API routes:

- `GET /api/cloud/providers`
- `POST /api/cloud/providers`
- `POST /api/cloud/providers/verify`
- `POST /api/cloud/providers/delete`
- `GET /api/cloud/lanes`
- `POST /api/cloud/lanes`

These routes should manage configuration, not perform final generation themselves.

Generation-time handlers should resolve lane assignments and then call the right local or cloud service.

## How the existing local flow should behave

If cloud lanes are disabled or unconfigured:

- Prompt Assist keeps using local helper selection
- Vision Assist keeps using local helper selection
- final generation keeps using the existing local backends only

That should remain the default first-run experience.

## Special concerns for Chatty-art

### Reference media sensitivity

Vision Assist and cloud media generation may upload user images, audio, or video references.

That means Chatty-art needs:

- explicit disclosure
- no hidden upload path
- likely per-lane warning copy

### Provider fragmentation

Text lanes are much easier to normalize than media-generation lanes.

Prompt Assist and Vision Assist can share a relatively unified adapter model.
Media generation probably cannot.

That means the media lane may need sub-lanes later:

- cloud image generation
- cloud video generation
- cloud audio generation

We should keep that future expansion in mind now.

### Cost clarity

Cloud assistant lanes and cloud generation lanes have different cost profiles.

The UI should not make them feel equivalent.

## Recommended first implementation scope

If we want the safest and most coherent first build:

1. add provider registry and persistence
2. add secret storage
3. add Prompt Assist cloud lane
4. add lane-aware verification
5. add UI status copy for local vs cloud assistant path

Then stop and assess before adding Vision Assist or cloud generation.

## Decision summary

Recommended design direction:

- use separate lanes, not one generic cloud bucket
- keep one shared provider registry underneath
- keep secrets separate from normal settings
- preserve local-first defaults
- ship Prompt Assist cloud support first
- add Vision Assist second
- leave cloud media generation for a later deliberate phase

## Short version

Chatty-art should support API keys as a lane family, not as a single switch.

The right mental model is:

- local media tool first
- optional cloud assistant helpers around it
- optional cloud generation later, clearly separated from the local backends

## Implementation checklist

Use this as the build checklist when work starts.

### Foundation

- [x] Add a non-secret preferences/config file for cloud provider metadata and lane assignments.
- [x] Decide and implement a separate secret-storage path for API keys.
- [x] Add backend modules for provider registry, lane resolution, and verification.
- [x] Keep all current local-only behavior working when no cloud config exists.

### Provider registry

- [x] Define provider kinds and capability flags.
- [x] Add CRUD routes for provider entries.
- [x] Add enable/disable support for saved provider entries.
- [x] Add storage for per-lane verification results and timestamps.
- [x] Add capability filtering so unsupported providers do not appear in the wrong lane selectors.

### Prompt Assist lane

- [x] Add a Prompt Assist lane selector with `local auto`, `local pinned`, and `cloud` options.
- [x] Add backend resolution logic for the Prompt Assist lane.
- [x] Add provider verification for text-assist behavior.
- [x] Add clear UI copy showing whether Prompt Assist is local or cloud for the current prepared job.
- [x] Ensure prepared prompt traces or notes record which lane was used.

### Vision Assist lane

- [x] Add a Vision Assist lane selector with `local auto`, `local pinned`, and `cloud` options.
- [x] Add backend resolution logic for multimodal/image-analysis providers.
- [x] Add verification for image-input capability.
- [x] Add disclosure copy explaining that cloud Vision Assist may upload the selected reference image.
- [x] Ensure prepared prompt traces or notes record which vision lane was used.

### Media generation lane

- [x] Add a separate design-specific output route selector for local vs cloud generation.
- [ ] Keep the current local backend naming intact and separate from remote providers.
- [x] Add provider-specific capability filtering for image, video, and audio generation.
- [x] Add lane-specific generation handlers that do not disturb the current local pipeline.
- [x] Add explicit warning copy before sending prompts or reference assets to cloud media providers.

Current phase-3 scope note:

- still-image, MP4 video, and speech-style audio cloud output are wired now
- Anthropic remains assist-only as of July 19, 2026. The official first-party Claude docs currently describe text output plus image input/vision, but do not expose a first-party still-image, speech-synthesis, or video-generation API for Chatty-art to wire here.
- xAI Grok and DeepSeek remain Prompt Assist-only as of July 19, 2026.
- Gemini speech uses Gemini's native TTS route and currently returns WAV audio built from 24 kHz mono PCM output
- cloud video currently uses OpenAI's deprecated Sora 2 / Videos API path and must be replaced before September 24, 2026. As of July 19, 2026, OpenAI's deprecations page lists that shutdown date but does not name a recommended replacement yet, so Chatty-art should treat the current OpenAI video lane as a temporary adapter with stricter guardrails rather than an evergreen route.
- the current expressive cloud media task selector is still `Image / GIF / Audio`; cloud GIF generation remains future work, so the remote MP4 video family does not currently appear there
- end-frame and control-video references still remain local-only for now

## Verified current UX shape

As of July 19, 2026, the verified cloud UX in Chatty-art is:

- cloud-capable models live inside the existing per-role selectors instead of separate vendor pickers
- cloud entries are grouped under `Cloud routes` alongside `Local GGUFs`
- the selected-model summary panel stays in place and swaps to cloud-facing route details when a cloud entry is selected
- saved provider setup, credentials, verification, and capability declaration live in a separate collapsed-by-default disclosure surface for each lane
- the saved-entry picker inside each lane is now labeled `Saved Cloud Account / Route`, not just `Saved Cloud Provider`
- lane selectors now surface configured remote targets as `Cloud Route: <display name>` so saved accounts and selectable routes stay visually distinct from local GGUF picks
- Prompt Assist provider setup currently offers `OpenAI`, `Anthropic`, `Gemini`, `xAI Grok`, `DeepSeek`, and `OpenAI-compatible`
- Vision Assist provider setup currently offers only truthfully wired vision families
- Media Generation provider setup currently offers only `OpenAI` and `Gemini`
- saved-provider dropdowns are lane-filtered so a shared-registry provider does not imply support on every lane
- Prompt Assist can be switched on with no image present and still surface Vision Assist route controls in a preselected-but-inactive state
- Vision Assist stays visible when Prompt Assist is on, keeps its provider disclosure closed by default, and disables the actual lane selector until a still image becomes the primary reference
- populated cloud summaries now name the exact configured model plus provider family plus saved account, instead of falling back to generic provider-only wording

## Verified adapter coverage

As of July 19, 2026, the following provider-family paths are now backed by direct adapter tests in `src/cloud_ai.rs`, not only by UI smoke passes:

- Anthropic Prompt Assist verify path
- Anthropic Prompt Assist JSON generation path
- Anthropic Vision Assist verify path
- Anthropic Vision Assist reference-image JSON generation path
- xAI Grok Prompt Assist verify path
- xAI Grok Prompt Assist JSON generation path
- DeepSeek Prompt Assist verify path
- DeepSeek Prompt Assist JSON generation path
- OpenAI Vision Assist verify path
- OpenAI Vision Assist reference-image JSON generation path
- OpenAI image-generation request shape
- OpenAI speech-generation request shape and default `marin` fallback
- OpenAI-compatible Vision Assist verify path
- OpenAI-compatible Vision Assist reference-image JSON generation path
- OpenAI-compatible media-generation block path, confirming that this family stays assist-only in practice
- Gemini Vision Assist verify path
- Gemini Vision Assist reference-image JSON generation path
- Gemini image-generation request shape
- Gemini native TTS request shape and default `Kore` voice fallback
- Gemini video-generation request shape, polling flow, and normalized-duration behavior

This means the currently surfaced cloud families are no longer relying only on selector presence plus manual smoke confidence. Their lane adapters now have direct request-shape coverage for the supported families we expose in Chatty-art today.

### UX and trust

- [x] Add low-noise privacy copy for each lane.
- [x] Add lane-specific status badges or summaries.
- [x] Avoid wording that makes cloud output sound identical to local generation.
- [x] Keep local as the default first-run path.
- [ ] Avoid any automatic local-to-cloud fallback.

### Observability

- [x] Include lane origin in saved sidecars or run metadata.
- [x] Record whether prompt prep, vision analysis, and final generation were local or cloud.
- [x] Surface lane choice in UI summaries for prepared and completed jobs.
- [ ] Keep failure messages provider-aware and fix-oriented where practical.

### Safety and guardrails

- [x] Make sure reference uploads only happen when the selected lane actually needs them.
- [x] Keep cancellation behavior sane for cloud requests.
- [x] Prevent unsupported lane/provider combinations at selection time, not only at run time.
- [x] Make sure missing keys, bad endpoints, and unsupported models fail clearly.

## Recommended implementation sequence

This is the suggested build order.

### Step 1. Add config and secret plumbing

Goal:

- give Chatty-art a stable place to persist cloud provider metadata
- keep key storage separate from regular settings if possible

Deliverables:

- config file shape
- load/save helpers
- secret read/write/delete helpers

Do not add UI complexity yet beyond what is needed to prove persistence.

### Step 2. Add shared provider registry

Goal:

- create a single source of truth for saved providers

Deliverables:

- provider kinds
- capability flags
- backend routes for save/list/delete/update
- verification status fields

This is the layer both assistant lanes and future media-generation lanes will reuse.

### Step 3. Ship Prompt Assist cloud lane first

Goal:

- add the least disruptive, highest-value cloud lane first

Deliverables:

- Prompt Assist lane selector
- text-provider resolution
- verification flow
- UI copy explaining local vs cloud Prompt Assist

Why first:

- easiest to fit into the current architecture
- strongest practical value
- lowest risk to the core local media identity

### Step 4. Add Vision Assist cloud lane

Goal:

- support cloud image/reference interpretation when the user wants it

Deliverables:

- Vision Assist lane selector
- multimodal capability checks
- image-analysis verification path
- explicit upload disclosure

This should reuse as much of the provider registry as possible while remaining stricter about capabilities.

### Step 5. Strengthen run metadata and UX summaries

Goal:

- make it obvious what path each job used

Deliverables:

- lane info in prepared-job summaries
- lane info in output sidecars
- lane info in completion notes and failure messages

This protects trust and helps debugging.

### Step 6. Design and ship cloud media generation separately

Goal:

- add remote final-output generation only after the assistant lanes are stable

Deliverables:

- explicit local vs cloud output route model
- provider-specific remote generation adapters
- media-type capability gating
- asset-upload disclosure and cost/trust copy

This should be treated as a separate product milestone, not just "one more provider field."

## Suggested first build milestone

If we want a disciplined first milestone, keep it small:

1. provider registry
2. secret storage
3. Prompt Assist cloud lane
4. Prompt Assist verification
5. lane provenance in prepared-job notes

That gives real user value without prematurely committing Chatty-art to remote final generation.

## Out of scope for the first pass

To keep the first implementation sane, avoid these initially:

- automatic fallback from local Prompt Assist to cloud Prompt Assist
- automatic fallback from local generation to cloud generation
- one universal "cloud model" selector shared across every lane
- pretending that remote media providers fit the same abstraction as local `stable-diffusion.cpp`
- trying to support every possible image, video, and audio provider in the first release

## File-level planning hints

Likely new or changed areas:

- `src/main.rs`
- `src/types.rs`
- new config/preferences module
- new secrets module
- new cloud provider module(s)
- `static/index.html`
- `static/app.js`

Likely responsibilities:

- backend config and provider logic should stay in Rust
- frontend should focus on lane selection, disclosure, status, and verification surfaces
- current local generation code paths should stay intact and remain the default branch

## Definition of done for phase 1

Phase 1 should count as done when:

- a user can save a cloud provider for Prompt Assist
- the API key path works
- the provider can be verified
- Prompt Assist can explicitly run through that cloud lane
- the UI clearly says that Prompt Assist is using cloud
- the final media generation path remains local unless the user deliberately chose otherwise

## Engineering task map

This section translates the design into likely code work areas for phase 1.

## Phase 1 target

Phase 1 means:

- shared provider registry
- secret storage
- Prompt Assist cloud lane only
- provider verification for Prompt Assist
- lane provenance in prepared-job notes

Vision Assist and cloud media generation stay out of scope for this pass.

## Archive note

The large phase-by-phase build checklist that used to live below this point is now mostly historical.

Those implementation notes were useful while the lane system was being built, but they are no longer a good source of truth because the following areas are now already in the app:

- provider CRUD
- lane assignment persistence
- per-lane verification
- Prompt Assist cloud routing
- Vision Assist cloud routing
- cloud media routing for still images, MP4 video, and speech-style audio
- lane provenance in prepared output, saved output metadata, and sidecars
- local-first trust and disclosure copy
- provider enable/disable support

Leaving the old unchecked phase-1 task lists in place made the document read as if core plumbing was still missing when it is not.

## Current state summary

As of July 19, 2026, Chatty-art already has:

- a shared saved-provider registry
- separate Prompt Assist, Vision Assist, and Media Generation lanes
- lane-specific provider filtering
- lane-specific verification status
- explicit local-vs-cloud output disclosure
- guardrails for reference uploads on the cloud media lane
- provider-aware cloud failure copy
- the current temporary OpenAI video route, with an explicit September 24, 2026 replacement deadline

## Remaining future work

The meaningful remaining work is now feature expansion or policy choice, not core lane scaffolding.

### High-value future items

- decide whether any explicit local fallback policy should ever exist for cloud Prompt Assist failures
- replace the deprecated OpenAI cloud video path before September 24, 2026
- decide whether cloud GIF output should ever piggyback on a remote video route or stay local-only
- expand cloud media support beyond still images, the temporary MP4 video bridge, and speech only if a healthy provider target exists
- choose whether saved provider secrets should stay file-backed or move to a platform credential store later
- consider splitting cloud/provider types out of `src/types.rs` if the file grows much further
- add a short regression checklist for lane behavior instead of the old phase-1 implementation checklist

### Recommended future testing focus

- fresh local-first startup still behaves cleanly when no cloud providers exist
- disabled providers stay saved but cannot be selected into active lanes
- cloud-lane disclosure stays accurate when switching kinds, references, and providers
- provider verification failures remain understandable for bad key, bad endpoint, and bad model name
- no cloud lane ever triggers unless explicitly selected
