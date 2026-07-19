# Chatty-art Regression Checklist

Last updated: July 19, 2026

This is a practical smoke-test checklist for the current cloud-lane system in Chatty-art.
It is meant for quick manual verification after lane, provider, reference, or output-route changes.

## Current provider matrix

Use this as the source of truth for what should work right now:

- `OpenAI`
  - Prompt Assist
  - Vision Assist
  - cloud image generation
  - cloud speech generation
  - cloud MP4 video generation on the current deprecated OpenAI Videos API path
- `Anthropic`
  - Prompt Assist
  - Vision Assist
- `Gemini`
  - Prompt Assist
  - Vision Assist
  - cloud image generation
  - cloud speech generation on Gemini's native TTS route
  - cloud MP4 video generation on the Gemini OpenAI-compatible Veo route
- `OpenAI-compatible`
  - Prompt Assist
  - Vision Assist when the chosen endpoint accepts image input on its OpenAI-style chat route
  - cloud image, speech, and video lanes are intentionally blocked until a specific compatible-media adapter is wired
- `xAI Grok`
  - Prompt Assist
- `DeepSeek`
  - Prompt Assist

## Scope

This checklist covers:

- local-first startup behavior
- saved cloud provider management
- Prompt Assist lane routing
- Vision Assist lane routing
- cloud media routing for still images, MP4 video, and speech-style audio
- reference-assignment guardrails
- provenance and saved-output surfaces
- cloud failure and cancellation behavior

It does not try to validate future cloud GIF work, character uploads, extensions, remix, generic OpenAI-compatible media adapters, or post-September-24-2026 replacements for the current OpenAI video path.

## Verified UX shape

- [ ] Each lane keeps its selector and selected-model summary visible even when provider setup is untouched.
- [ ] Each lane's provider setup is wrapped in a collapsed-by-default disclosure surface.
- [ ] The saved-entry picker title in each lane reads `Saved Cloud Account / Route`.
- [ ] Prompt Assist and Vision Assist model selectors group remote entries under `Cloud routes` and local entries under `Local GGUFs`.
- [ ] Lane selectors render configured remote targets as `Cloud Route: <display name>`.
- [ ] Prompt Assist provider setup offers `OpenAI`, `Anthropic`, `Gemini`, `xAI Grok`, `DeepSeek`, and `OpenAI-compatible`.
- [ ] Vision Assist provider setup offers only truthfully wired vision families.
- [ ] Media provider setup offers only `OpenAI` and `Gemini`.
- [ ] Saved-provider dropdowns are lane-filtered so non-media families do not appear in the media setup surface.

## Adapter coverage baseline

These are already covered by direct cloud-adapter tests in `src/cloud_ai.rs` and should stay green before deeper manual smoke:

- [ ] `Anthropic` Prompt Assist verify and generation paths.
- [ ] `Anthropic` Vision Assist verify and generation paths.
- [ ] `xAI Grok` Prompt Assist verify and generation paths.
- [ ] `DeepSeek` Prompt Assist verify and generation paths.
- [ ] `OpenAI` Vision Assist verify and generation paths.
- [ ] `OpenAI` image-generation request shape.
- [ ] `OpenAI` speech-generation request shape and default `marin` fallback.
- [ ] `OpenAI-compatible` Vision Assist verify and generation paths.
- [ ] `OpenAI-compatible` media-generation blocking path.
- [ ] `Gemini` Vision Assist verify and generation paths.
- [ ] `Gemini` image-generation request shape.
- [ ] `Gemini` native TTS request shape and default `Kore` fallback.
- [ ] `Gemini` video-generation request shape, polling flow, and normalized-duration behavior.

## Test setup

Before running the checklist:

- start from the current `chatty-art` folder
- keep at least one working local generation model available
- keep one valid cloud provider entry available for each lane you want to test
- keep one intentionally broken provider entry available for failure checks
- keep one still image in `input/images`
- keep one short audio file in `input/audio`
- keep one short video or GIF in `input/video`

Recommended broken-provider cases:

- bad API key
- bad base URL
- unsupported provider/lane pairing
- empty or wrong model name

## Practical smoke order

Run the smoke pass in this order so each provider family proves one thing at a time:

0. Selector wording sanity pass
   - Confirm all three provider disclosures are closed by default after reload
   - Confirm each saved-entry picker reads `Saved Cloud Account / Route`
   - Confirm populated lane selectors render `Cloud Route: <display name>`
   - Confirm populated Vision Assist model selector uses `Local GGUFs` and `Cloud routes` optgroup labels
1. `OpenAI` Prompt Assist
   - Save a valid OpenAI Prompt Assist entry
   - Verify it
   - Switch Prompt Assist to that cloud lane
   - Confirm a prepared handoff shows cloud Prompt Assist provenance
2. `Anthropic` Prompt Assist
   - Save a valid Anthropic Prompt Assist entry
   - Verify it
   - Confirm Prompt Assist can switch to it cleanly
   - Confirm the saved verification note stays compact and does not dump raw provider payloads
3. `xAI Grok` Prompt Assist
   - Save a valid xAI Grok Prompt Assist entry
   - Verify it
   - Confirm Prompt Assist can switch to it cleanly
4. `DeepSeek` Prompt Assist
   - Save a valid DeepSeek Prompt Assist entry
   - Verify it
   - Confirm Prompt Assist can switch to it cleanly
5. `Anthropic` Vision Assist
   - Save or reuse a valid Anthropic Vision Assist entry
   - Verify it
   - Assign a still image reference
   - Confirm prepared handoff shows cloud Vision Assist provenance
6. `OpenAI` Vision Assist
   - Save or reuse a valid OpenAI Vision Assist entry
   - Verify it
   - Assign a still image reference
   - Confirm prepared handoff shows cloud Vision Assist provenance
7. `Gemini` Vision Assist
   - Save a valid Gemini Vision Assist entry
   - Verify it
   - Assign a still image reference
   - Confirm prepared handoff shows cloud Vision Assist provenance
6. `OpenAI-compatible` Vision Assist
   - Save a valid OpenAI-compatible Vision Assist entry backed by an endpoint that accepts image input
   - Verify it
   - Assign a still image reference
   - Confirm prepared handoff shows cloud Vision Assist provenance
7. `OpenAI` cloud image
   - Save a valid OpenAI media entry with image capability
   - Verify it
   - Switch Media Generation to that cloud provider
   - Confirm image generation works with and without one still-image guide
8. `OpenAI` cloud speech
   - Reuse or save a valid OpenAI media entry with speech capability
   - Verify it
   - Confirm speech generation works and rejects tray references early
9. `OpenAI` cloud video
   - Reuse or save a valid OpenAI media entry with video capability
   - Verify it
   - Confirm MP4 generation works
   - Confirm `4`, `8`, `12`, `16`, and `20` second durations are accepted
   - Confirm `1920x1080` and `1080x1920` are accepted only on `sora-2-pro`
   - Confirm saved notes mention the deprecated OpenAI Videos API path
10. `Gemini` cloud image
   - Save a valid Gemini media entry with image capability
   - Verify it
   - Confirm still-image generation works
11. `Gemini` cloud speech
   - Save a valid Gemini media entry with a TTS model and voice name
   - Verify it
   - Confirm speech generation works and returns a playable WAV
   - Confirm saved notes mention the chosen Gemini voice or default voice fallback
12. `Gemini` cloud video
   - Save a valid Gemini media entry with video capability
   - Verify it
   - Confirm prompt-only MP4 generation works
   - Confirm `4`, `6`, and `8` second durations are the only accepted values
   - Confirm still-image guide mode only works at `8` seconds
13. Blocked-family checks
   - Confirm `Anthropic` media save/verify stays blocked as not wired
   - Confirm generic `OpenAI-compatible` media save/verify stays blocked as not wired

## Suggested provider fixtures

Keep the fixture set small so you can move quickly:

- one working `OpenAI` Prompt Assist model
- one working `Anthropic` Prompt Assist model
- one working `xAI Grok` Prompt Assist model
- one working `DeepSeek` Prompt Assist model
- one working `Anthropic` Vision Assist model
- one working `OpenAI` Vision Assist model
- one working `Gemini` Vision Assist model
- one working `OpenAI-compatible` Vision Assist endpoint
- one working `OpenAI` image model
- one working `OpenAI` speech model
- one working `OpenAI` video model
- one working `Gemini` image model
- one working `Gemini` speech model
- one working `Gemini` video model
- one intentionally bad entry per provider family for failure text checks

## Local-first baseline

- [ ] Fresh startup with no active cloud lane still reads as local-first in the UI.
- [ ] Prompt Assist defaults to local auto.
- [ ] Vision Assist defaults to local auto.
- [ ] Media Generation defaults to local only.
- [ ] No cloud wording implies that a remote lane is active when no cloud lane is selected.
- [ ] A normal local generation request still works without touching any cloud controls.

## Provider management

- [ ] Create a new Prompt Assist provider and save it successfully.
- [ ] Create a new Vision Assist provider and save it successfully.
- [ ] Create a new cloud media provider and save it successfully.
- [ ] A newly saved provider immediately appears as a `Cloud Route:` option on every capability-appropriate lane and nowhere else.
- [ ] Edit an existing provider and confirm the changes persist after reload.
- [ ] Save a provider with a replacement API key and confirm it still verifies.
- [ ] Delete a provider and confirm it disappears from saved-provider lists and lane selectors.
- [ ] Disable a provider and confirm it stays editable but drops out of lane selectors.
- [ ] Re-enable a disabled provider and confirm it returns to eligible lane selectors.
- [ ] Disabled providers are visibly marked as disabled in saved-provider dropdowns.

## Validation and config errors

- [ ] Saving a provider with no model names fails clearly.
- [ ] Saving a provider with no stored API key fails clearly.
- [ ] Saving a provider with an invalid base URL fails clearly.
- [ ] Saving unsupported lane fields on a provider kind fails early with lane-specific `not wired yet` wording.
- [ ] Verifying a provider with a bad key fails with understandable provider-aware text.
- [ ] Verifying a provider with a bad model name fails with understandable provider-aware text.
- [ ] Verifying a provider against an unsupported lane fails with clear `not wired yet` wording.

## Prompt Assist lane

- [ ] Prompt Assist can stay local while other lanes use cloud providers.
- [ ] Prompt Assist can switch to a valid `OpenAI` provider.
- [ ] Prompt Assist can switch to a valid `Anthropic` provider.
- [ ] Prompt Assist can switch to a valid `Gemini` provider.
- [ ] Prompt Assist can switch to a valid `xAI Grok` provider.
- [ ] Prompt Assist can switch to a valid `DeepSeek` provider.
- [ ] Prompt Assist can switch to a valid `OpenAI-compatible` provider.
- [ ] Prepared handoff output clearly shows local Prompt Assist when local is used.
- [ ] Prepared handoff output clearly shows cloud Prompt Assist when cloud is used.
- [ ] The Prompt Assist lane summary names the exact configured cloud model plus provider family plus saved account when a cloud route is selected.
- [ ] A cloud Prompt Assist provider failure does not silently fall back to cloud output on another lane.
- [ ] A Prompt Assist verification failure does not break unrelated local generation.

## Vision Assist lane

- [ ] Vision Assist stays local by default.
- [ ] Turning Prompt Assist on with no still image still surfaces Vision Assist cloud setup and model selection instead of hiding them.
- [ ] With Prompt Assist on but no still image, Vision Assist route/model choices can be preselected while the lane selector itself stays disabled.
- [ ] Vision Assist cloud mode only becomes active when a still image is assigned as the primary reference.
- [ ] Vision Assist can switch to a valid `Anthropic` provider and still keep Prompt Assist/output on their own lanes.
- [ ] Vision Assist can switch to a valid `OpenAI` provider and still keep Prompt Assist/output on their own lanes.
- [ ] Vision Assist can switch to a valid `Gemini` provider and still keep Prompt Assist/output on their own lanes.
- [ ] Vision Assist can switch to a valid `OpenAI-compatible` provider when that endpoint accepts image input on its OpenAI-style chat route.
- [ ] Prepared handoff output clearly shows local Vision Assist when local is used.
- [ ] Prepared handoff output clearly shows cloud Vision Assist when cloud is used.
- [ ] The Vision Assist summary names the exact configured cloud model plus provider family plus saved account when a cloud route is selected or preselected.
- [ ] Cloud Vision Assist only attempts to inspect a still-image reference, not audio or video.

## Cloud media lane: still image

- [ ] A valid `OpenAI` cloud media provider with image capability appears in the media lane selector.
- [ ] A valid `Gemini` cloud media provider with image capability appears in the media lane selector.
- [ ] `Anthropic` media entries stay blocked as not wired.
- [ ] Generic `OpenAI-compatible` media entries stay blocked as not wired.
- [ ] Selecting the cloud media lane disables local final-model selection as expected.
- [ ] `Prepare Kind` only offers kinds supported by the active cloud media provider.
- [ ] The final-generation summary names the exact configured cloud model plus provider family plus saved account when a cloud route is selected.
- [ ] Cloud image generation works with no references.
- [ ] Cloud image generation works with one still-image guide/edit reference if supported by the current lane rules.
- [ ] End-frame references are blocked before generation on the cloud image lane.
- [ ] Control-video references are blocked before generation on the cloud image lane.
- [ ] The UI makes it clear that cloud image output is a remote lane, not the same runtime as local generation.

## Cloud media lane: speech audio

- [ ] A valid `OpenAI` cloud media provider with audio capability appears in the media lane selector.
- [ ] A valid `Gemini` cloud media provider with audio capability appears in the media lane selector.
- [ ] `Prepare Kind` offers `Audio` only when the active cloud media provider actually supports cloud speech audio.
- [ ] Cloud speech generation works from prompt plus spoken text/script content.
- [ ] Cloud speech generation works when Prompt Assist is local.
- [ ] Cloud speech generation works when Prompt Assist is cloud.
- [ ] Cloud speech output is labeled as cloud speech, not just generic audio, in prepared/saved surfaces.
- [ ] `Gemini` cloud speech returns a playable WAV and respects the saved voice name or the default `Kore` fallback.
- [ ] `Anthropic` cloud speech stays blocked as not wired.
- [ ] Generic `OpenAI-compatible` cloud speech stays blocked as not wired.
- [ ] Still-image guide references are blocked on the cloud speech lane before generation.
- [ ] Voice-reference audio files are blocked on the cloud speech lane before generation.
- [ ] End-frame and control-video references are blocked on the cloud speech lane before generation.

## Cloud media lane: MP4 video

- [ ] A valid `OpenAI` cloud media provider with video capability appears in the media lane selector.
- [ ] A valid `Gemini` cloud media provider with video capability appears in the media lane selector.
- [ ] In the current expressive build, `Prepare Kind` remains `Image / GIF / Audio`, so cloud MP4 video does not currently appear there.
- [ ] Cloud GIF generation remains blocked as future work instead of silently mapping to the cloud MP4 path.
- [ ] `OpenAI` cloud video generation works with one still-image guide reference when the image matches the chosen cloud video size.
- [ ] `OpenAI` cloud video accepts only `4`, `8`, `12`, `16`, or `20` second durations.
- [ ] `OpenAI` cloud video rejects `1920x1080` and `1080x1920` unless the model is `sora-2-pro`.
- [ ] `Gemini` cloud video generation works with one still-image guide reference only when duration is `8` seconds.
- [ ] Cloud video generation rejects unsupported video sizes before submission with clear guidance.
- [ ] `Gemini` cloud video rejects durations other than `4`, `6`, or `8` seconds before submission.
- [ ] End-frame references are blocked on the cloud video lane before generation.
- [ ] Control-video references are blocked on the cloud video lane before generation.
- [ ] Non-image primary references are blocked on the cloud video lane before generation.
- [ ] `OpenAI` prepared and saved output notes clearly say the video used the current deprecated OpenAI Videos API path.
- [ ] `Gemini` prepared and saved output notes clearly identify the Gemini Veo route instead of the deprecated OpenAI path.

## Reference guardrails

- [ ] Switching from a compatible local reference setup to cloud speech clears incompatible references.
- [ ] Switching from a compatible local reference setup to cloud image clears end/control references but preserves a valid still-image primary reference.
- [ ] Disabled reference buttons cannot still assign an invalid slot through the tray UI.
- [ ] The tray helper text changes to match the active lane rules.
- [ ] No reference asset is uploaded unless the selected lane actually needs it.

## Output metadata and provenance

- [ ] Saved output JSON includes prompt-assist, vision-assist, and output-route provenance where expected.
- [ ] Planner sidecars include lane provenance.
- [ ] Compiler sidecars include lane provenance.
- [ ] Preview panel shows the correct route labels for local lane vs cloud lane runs.
- [ ] History cards show route-aware summaries for cloud speech and cloud media outputs.

## Cancellation and failure behavior

- [ ] Canceling a local job still works.
- [ ] Canceling an in-flight cloud image job stops cleanly and reports cancellation clearly.
- [ ] Canceling an in-flight cloud speech job stops cleanly and reports cancellation clearly.
- [ ] Batch jobs stop queued follow-up items after cancellation.
- [ ] Cloud provider request failures use provider-aware wording instead of generic `health check failed` phrasing during real generation.
- [ ] Provider verify failures stay understandable for bad endpoint, bad key, and bad model name.

## Final confidence pass

- [ ] No cloud lane ever triggers unless it is explicitly selected.
- [ ] Local-only users can ignore the cloud controls without confusing regressions.
- [ ] Disabled providers never appear as valid active lane targets.
- [ ] The app still feels local-first even after all cloud controls are present.

## Suggested quick-run subset

If you only have time for a short smoke test, run these first:

- [ ] Local generation still works untouched.
- [ ] Prompt Assist cloud lane still prepares a handoff on at least one `OpenAI` or `Anthropic` provider.
- [ ] Vision Assist cloud lane still analyzes a still image on at least one `Anthropic`, `OpenAI`, or `Gemini` provider.
- [ ] Cloud image output still generates a still image on at least one `OpenAI` or `Gemini` provider.
- [ ] Cloud speech output still generates spoken audio.
- [ ] Cloud speech lane rejects tray references early.
- [ ] Gemini cloud video rejects bad durations early and accepts `8` seconds with a still-image guide.
- [ ] Provider save/verify errors stay understandable.
- [ ] Canceling an in-flight cloud request still behaves sanely.
