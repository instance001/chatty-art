# ECG Window

## What It Is

The `ECG Window` is a small live activity monitor for generation workloads.

It is meant to feel like a medical ECG monitor:

- flat line = little or no activity
- spikes = work is happening
- repeated rhythm = the backend is actively chewing through a job

In Chatty-art, it is used as a simple visual bridge between:

- "the job is running"
- "the GPU is actually doing something"

That makes it much easier for non-technical users to understand whether a generation job is:

- active
- idle
- stalled
- or simply in a slow phase

## What It Does

The ECG Window is not a full hardware dashboard.

It deliberately keeps the scope narrow:

- show a small live line graph
- show current activity percentage
- update during generation
- stay readable at a glance

It is not trying to replace Task Manager, MSI Afterburner, or a full telemetry panel.

Its job is reassurance and fast feedback.

## Why It Exists

AI generation tools often feel "frozen" while working, especially during:

- model loading
- shader compilation
- denoising
- video frame generation
- VAE decode

For pedestrian users, a tiny moving graph is much easier to interpret than logs or counters.

The ECG analogy gives a natural mental model:

- movement means life
- flatline means nothing is happening

That is why the feature is named `ECG Window` instead of something more technical like `GPU Telemetry Sparkline`.

## Current Chatty-art Shape

In Chatty-art, the ECG Window has 3 parts:

1. A sampler
   - polls local GPU activity on Windows
   - currently uses Windows performance counters

2. A tiny API contract
   - backend exposes the current sample plus recent history

3. A lightweight frontend renderer
   - draws a sparkline line and fill area
   - updates a small percentage label

Current backend shape:

```json
{
  "supported": true,
  "label": "ECG Window - AMD Radeon RX 9060 XT",
  "note": "ECG-style view of the busiest Windows GPU engine, similar to Task Manager.",
  "current_percent": 73.0,
  "history": [12.0, 31.0, 78.0, 64.0, 81.0]
}
```

## Standalone Function Outline

If someone wants to reuse this in another project, the feature can be lifted out as a standalone pattern:

### Input

- local hardware activity samples
- ideally a single normalized percentage from `0` to `100`

### Internal State

- a short rolling history buffer
- usually 30 to 60 samples is enough

### Output

- current percentage
- optional label
- optional note
- history array for drawing

### UI

- small chart area
- single line path
- optional filled area under the line
- one text label for current value

## Minimal Architecture

The smallest useful version looks like this:

1. Sample hardware activity every `1` to `2` seconds
2. Clamp the value to `0..100`
3. Push it into a fixed-length queue
4. Expose it through a tiny local endpoint or local state store
5. Render the history as a sparkline

That means the ECG Window can be implemented in:

- desktop apps
- local web dashboards
- Electron apps
- Tauri apps
- game tools
- AI runners
- render queues

## Why This Pattern Transfers Well

This feature is useful anywhere a job can take time and users need confidence that work is still happening.

Examples:

- local image generation
- local video generation
- LLM inference
- audio synthesis
- file encoding
- simulation tools
- batch processing dashboards

The pattern is generic because it does not care what the workload is.
It only cares whether the machine is active.

## Design Principles

If another project adopts this idea, these are the important constraints:

- keep it small
- keep it passive
- keep it glanceable
- avoid turning it into a full monitoring suite
- use human-friendly language

The feature works best when it answers one question:

"Is the machine alive and working?"

## Good Defaults

Recommended defaults for a reusable implementation:

- poll interval: `1200ms` to `2000ms`
- history length: `40` to `60`
- chart width: small, status-bar sized
- chart height: short enough to feel like an instrument panel
- fallback behavior: hide or show "temporarily unavailable"

## Failure Handling

A good ECG Window should fail gently.

If telemetry is unavailable:

- do not crash the app
- show a neutral note
- keep generation working

This is especially important because telemetry APIs vary across:

- Windows
- Linux
- macOS
- different GPU vendors

## Porting Checklist

If someone wants to reuse this feature in another project, the practical checklist is:

1. Choose a hardware signal source
   - GPU is ideal for generation tools
   - CPU is acceptable if GPU telemetry is unavailable

2. Normalize the samples
   - always emit a simple percentage

3. Keep a rolling buffer

4. Expose a tiny data contract

5. Render a sparkline

6. Label it with a human-friendly term
   - `ECG Window`
   - not a deeply technical term unless the app is aimed at engineers

## Summary

The ECG Window is a tiny live activity monitor for long-running local jobs.

Its value is not deep hardware analysis.
Its value is user confidence.

It gives people a fast visual answer to:

"Is this thing still alive?"
