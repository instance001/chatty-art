use std::{f32::consts::PI, fs::File, path::Path};

use anyhow::{Context, Result};
use gif::{Encoder, Frame, Repeat};
use hound::{SampleFormat, WavSpec, WavWriter};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::types::{GenerationSettings, MediaKind, ReferenceSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePlan {
    pub background_top: String,
    pub background_bottom: String,
    pub accent: String,
    pub shapes: Vec<ShapePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPlan {
    pub background_top: String,
    pub background_bottom: String,
    pub accent: String,
    pub fps: u16,
    pub frames: u16,
    pub shapes: Vec<MotionShapePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPlan {
    pub bpm: u16,
    pub duration_seconds: f32,
    pub layers: Vec<AudioLayerPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapePlan {
    pub kind: ShapeKind,
    #[serde(default)]
    pub role: SceneRole,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub aspect: f32,
    pub rotation: f32,
    pub color: String,
    pub secondary_color: String,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionShapePlan {
    #[serde(flatten)]
    pub base: ShapePlan,
    pub drift_x: f32,
    pub drift_y: f32,
    pub pulse: f32,
    pub spin: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLayerPlan {
    pub wave: Waveform,
    pub gain: f32,
    pub pan: f32,
    pub octave: i32,
    pub notes: Vec<i32>,
    pub rhythm: Vec<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeKind {
    Circle,
    Rectangle,
    Line,
    Ring,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneRole {
    Background,
    Horizon,
    Ground,
    Subject,
    Celestial,
    Reflection,
    #[default]
    Detail,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

pub fn render_image(
    output_path: &Path,
    plan: &ImagePlan,
    settings: &GenerationSettings,
    reference: Option<&ReferenceSummary>,
) -> Result<()> {
    let (width, height) = settings.dimensions_for(MediaKind::Image);
    let mut canvas = RgbaImage::new(width, height);
    draw_scene(
        &mut canvas,
        &plan.background_top,
        &plan.background_bottom,
        &plan.accent,
        &plan.shapes,
        reference,
        None,
    );
    canvas
        .save(output_path)
        .with_context(|| format!("failed to save {}", output_path.display()))
}

pub fn render_video(
    output_path: &Path,
    plan: &VideoPlan,
    settings: &GenerationSettings,
    reference: Option<&ReferenceSummary>,
) -> Result<()> {
    let (width, height) = settings.dimensions_for(MediaKind::Gif);
    let file = File::create(output_path)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    let mut encoder = Encoder::new(file, width as u16, height as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    let fps = settings.video_fps.clamp(8, 24) as u16;
    let frame_count = settings.video_frame_count().clamp(16, 480) as u16;

    for frame_index in 0..frame_count {
        let phase = frame_index as f32 / frame_count as f32;
        let animated_shapes = plan
            .shapes
            .iter()
            .map(|shape| animate_shape(shape, phase))
            .collect::<Vec<_>>();

        let mut canvas = RgbaImage::new(width, height);
        draw_scene(
            &mut canvas,
            &plan.background_top,
            &plan.background_bottom,
            &plan.accent,
            &animated_shapes,
            reference,
            Some(phase),
        );

        let mut pixels = canvas.into_raw();
        let mut gif_frame = Frame::from_rgba_speed(width as u16, height as u16, &mut pixels, 10);
        gif_frame.delay = ((100.0 / fps as f32).round() as u16).max(1);
        encoder.write_frame(&gif_frame)?;
    }

    Ok(())
}

pub fn render_audio(output_path: &Path, plan: &AudioPlan) -> Result<()> {
    let spec = WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output_path, spec)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    let total_samples = (plan.duration_seconds.clamp(2.5, 12.0) * spec.sample_rate as f32) as usize;

    for index in 0..total_samples {
        let t = index as f32 / spec.sample_rate as f32;
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for layer in &plan.layers {
            let (sample_l, sample_r) = render_audio_layer(layer, plan.bpm, t);
            left += sample_l;
            right += sample_r;
        }

        let left = (left * 0.65).clamp(-1.0, 1.0);
        let right = (right * 0.65).clamp(-1.0, 1.0);
        writer.write_sample((left * i16::MAX as f32) as i16)?;
        writer.write_sample((right * i16::MAX as f32) as i16)?;
    }

    writer.finalize()?;
    Ok(())
}

fn animate_shape(shape: &MotionShapePlan, phase: f32) -> ShapePlan {
    let swing = (phase * PI * 2.0).sin();
    let bob = (phase * PI * 2.0).cos();

    ShapePlan {
        kind: shape.base.kind,
        role: shape.base.role,
        x: (shape.base.x + shape.drift_x * swing).clamp(0.05, 0.95),
        y: (shape.base.y + shape.drift_y * bob).clamp(0.05, 0.95),
        size: (shape.base.size * (1.0 + shape.pulse * swing.abs())).clamp(0.04, 0.65),
        aspect: shape.base.aspect.clamp(0.2, 2.2),
        rotation: shape.base.rotation + (shape.spin * phase * 360.0),
        color: shape.base.color.clone(),
        secondary_color: shape.base.secondary_color.clone(),
        opacity: (shape.base.opacity * (0.8 + 0.2 * bob.abs())).clamp(0.15, 1.0),
    }
}

fn draw_scene(
    canvas: &mut RgbaImage,
    background_top: &str,
    background_bottom: &str,
    accent: &str,
    shapes: &[ShapePlan],
    reference: Option<&ReferenceSummary>,
    animation_phase: Option<f32>,
) {
    let top = blend_with_reference(
        background_top,
        reference.and_then(|r| r.palette.first()),
        0.25,
    );
    let bottom = blend_with_reference(
        background_bottom,
        reference.and_then(|r| r.palette.get(1)),
        0.2,
    );
    let accent = blend_with_reference(accent, reference.and_then(|r| r.palette.get(2)), 0.35);

    fill_gradient(canvas, parse_hex_color(&top), parse_hex_color(&bottom));

    if let Some(phase) = animation_phase {
        add_vignette(canvas, phase);
    } else {
        add_vignette(canvas, 0.35);
    }

    let mut ordered_shapes = shapes.to_vec();
    ordered_shapes.sort_by_key(shape_role_priority);

    for (index, shape) in ordered_shapes.iter().enumerate() {
        let role = effective_scene_role(shape);
        let styled_shape = stylize_shape_for_role(shape, role, index);
        let base_color = parse_hex_color(&styled_shape.color);
        let secondary = blend_hex_colors(&styled_shape.secondary_color, &accent, 0.22);
        let tint = parse_hex_color(&secondary);
        let opacity = styled_shape.opacity.clamp(0.1, 1.0);

        match styled_shape.kind {
            ShapeKind::Circle => draw_circle(
                canvas,
                styled_shape.x,
                styled_shape.y,
                styled_shape.size,
                base_color,
                tint,
                opacity,
                role_ring_bias(role, index),
            ),
            ShapeKind::Rectangle => {
                draw_rotated_rect(canvas, &styled_shape, base_color, tint, opacity, false);
            }
            ShapeKind::Line => draw_line_shape(canvas, &styled_shape, base_color, opacity),
            ShapeKind::Ring => draw_ring(canvas, &styled_shape, base_color, tint, opacity),
        }
    }
}

fn shape_role_priority(shape: &ShapePlan) -> u8 {
    match effective_scene_role(shape) {
        SceneRole::Background => 0,
        SceneRole::Horizon => 1,
        SceneRole::Ground => 2,
        SceneRole::Celestial => 3,
        SceneRole::Reflection => 4,
        SceneRole::Detail => 5,
        SceneRole::Subject => 6,
    }
}

fn effective_scene_role(shape: &ShapePlan) -> SceneRole {
    if shape.role != SceneRole::Detail {
        return shape.role;
    }

    match shape.kind {
        ShapeKind::Circle | ShapeKind::Ring if shape.y <= 0.35 && shape.size >= 0.08 => {
            SceneRole::Celestial
        }
        ShapeKind::Line | ShapeKind::Rectangle if shape.aspect >= 3.5 && shape.y >= 0.78 => {
            SceneRole::Ground
        }
        ShapeKind::Line | ShapeKind::Rectangle if shape.aspect >= 3.5 && shape.y >= 0.42 => {
            SceneRole::Horizon
        }
        ShapeKind::Line | ShapeKind::Rectangle if shape.y >= 0.6 && shape.opacity <= 0.45 => {
            SceneRole::Reflection
        }
        _ if shape.size >= 0.18 => SceneRole::Subject,
        _ => SceneRole::Detail,
    }
}

fn stylize_shape_for_role(shape: &ShapePlan, role: SceneRole, index: usize) -> ShapePlan {
    let mut styled = shape.clone();

    match role {
        SceneRole::Background => {
            styled.opacity = (styled.opacity * 0.6).clamp(0.12, 0.45);
            styled.size = (styled.size + 0.08).clamp(0.12, 0.65);
            styled.aspect = (styled.aspect * 1.5).clamp(0.6, 3.0);
        }
        SceneRole::Horizon => {
            styled.kind = match styled.kind {
                ShapeKind::Circle | ShapeKind::Ring => ShapeKind::Line,
                other => other,
            };
            styled.y = styled.y.clamp(0.38, 0.68);
            styled.rotation = ((styled.rotation + 90.0) % 180.0) - 90.0;
            styled.size = styled.size.clamp(0.16, 0.45);
            styled.aspect = styled.aspect.max(5.0).min(14.0);
            styled.opacity = styled.opacity.clamp(0.24, 0.5);
        }
        SceneRole::Ground => {
            styled.kind = match styled.kind {
                ShapeKind::Circle | ShapeKind::Ring => ShapeKind::Rectangle,
                other => other,
            };
            styled.y = styled.y.clamp(0.72, 0.94);
            styled.rotation = ((styled.rotation + 90.0) % 180.0) - 90.0;
            styled.size = styled.size.clamp(0.16, 0.38);
            styled.aspect = styled.aspect.max(4.5).min(16.0);
            styled.opacity = styled.opacity.clamp(0.2, 0.46);
        }
        SceneRole::Subject => {
            styled.x = styled.x.clamp(0.18, 0.82);
            styled.y = styled.y.clamp(0.18, 0.86);
            styled.size = styled.size.clamp(0.12, 0.42);
            styled.opacity = styled.opacity.clamp(0.42, 0.95);
        }
        SceneRole::Celestial => {
            styled.kind = match styled.kind {
                ShapeKind::Rectangle | ShapeKind::Line => {
                    if index.is_multiple_of(2) {
                        ShapeKind::Ring
                    } else {
                        ShapeKind::Circle
                    }
                }
                other => other,
            };
            styled.x = styled.x.clamp(0.08, 0.92);
            styled.y = styled.y.clamp(0.06, 0.32);
            styled.size = styled.size.clamp(0.05, 0.22);
            styled.opacity = styled.opacity.clamp(0.3, 0.8);
        }
        SceneRole::Reflection => {
            styled.kind = match styled.kind {
                ShapeKind::Circle | ShapeKind::Ring => ShapeKind::Line,
                other => other,
            };
            styled.y = styled.y.clamp(0.58, 0.96);
            styled.rotation = if matches!(styled.kind, ShapeKind::Line) {
                90.0
            } else {
                styled.rotation
            };
            styled.size = styled.size.clamp(0.08, 0.28);
            styled.aspect = styled.aspect.clamp(0.15, 0.8);
            styled.opacity = (styled.opacity * 0.55).clamp(0.12, 0.35);
        }
        SceneRole::Detail => {
            styled.size = styled.size.clamp(0.04, 0.18);
            styled.opacity = styled.opacity.clamp(0.18, 0.75);
        }
    }

    styled
}

fn role_ring_bias(role: SceneRole, index: usize) -> f32 {
    match role {
        SceneRole::Celestial => 0.18,
        SceneRole::Reflection => 0.08,
        SceneRole::Background => 0.02,
        _ => index as f32 * 0.03,
    }
}

fn fill_gradient(canvas: &mut RgbaImage, top: [u8; 3], bottom: [u8; 3]) {
    let width = canvas.width();
    let height = canvas.height();

    for y in 0..height {
        let t = y as f32 / height.saturating_sub(1).max(1) as f32;
        let color = mix_rgb(top, bottom, t);
        for x in 0..width {
            canvas.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
        }
    }
}

fn add_vignette(canvas: &mut RgbaImage, amount: f32) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;
    let center_x = width * 0.5;
    let center_y = height * 0.5;
    let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt();

    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let shade = 1.0 - dist.powf(1.7) * amount.clamp(0.1, 0.55);
            let pixel = canvas.get_pixel_mut(x, y);
            pixel.0[0] = (pixel.0[0] as f32 * shade) as u8;
            pixel.0[1] = (pixel.0[1] as f32 * shade) as u8;
            pixel.0[2] = (pixel.0[2] as f32 * shade) as u8;
        }
    }
}

fn draw_circle(
    canvas: &mut RgbaImage,
    x: f32,
    y: f32,
    size: f32,
    color: [u8; 3],
    tint: [u8; 3],
    opacity: f32,
    ring_bias: f32,
) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;
    let min_dim = width.min(height);
    let radius = (size.clamp(0.04, 0.65) * min_dim * 0.42).max(6.0);
    let cx = x.clamp(0.02, 0.98) * width;
    let cy = y.clamp(0.02, 0.98) * height;

    let min_x = (cx - radius - 2.0).floor().max(0.0) as u32;
    let max_x = (cx + radius + 2.0).ceil().min(width - 1.0) as u32;
    let min_y = (cy - radius - 2.0).floor().max(0.0) as u32;
    let max_y = (cy + radius + 2.0).ceil().min(height - 1.0) as u32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= radius {
                let edge = 1.0 - (dist / radius);
                let tone = mix_rgb(color, tint, (dist / radius + ring_bias).min(1.0));
                blend_pixel(
                    canvas,
                    px,
                    py,
                    Rgba([
                        tone[0],
                        tone[1],
                        tone[2],
                        ((opacity * (0.55 + edge * 0.45)) * 255.0) as u8,
                    ]),
                );
            }
        }
    }
}

fn draw_rotated_rect(
    canvas: &mut RgbaImage,
    shape: &ShapePlan,
    color: [u8; 3],
    tint: [u8; 3],
    opacity: f32,
    hollow: bool,
) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;
    let min_dim = width.min(height);
    let cx = shape.x.clamp(0.02, 0.98) * width;
    let cy = shape.y.clamp(0.02, 0.98) * height;
    let half_h = (shape.size.clamp(0.03, 0.6) * min_dim * 0.26).max(4.0);
    let half_w = (half_h * shape.aspect.clamp(0.2, 2.5)).max(6.0);
    let radians = shape.rotation.to_radians();
    let cos_r = radians.cos();
    let sin_r = radians.sin();
    let bound = (half_w.max(half_h) * 1.5).ceil();

    let min_x = (cx - bound).floor().max(0.0) as u32;
    let max_x = (cx + bound).ceil().min(width - 1.0) as u32;
    let min_y = (cy - bound).floor().max(0.0) as u32;
    let max_y = (cy + bound).ceil().min(height - 1.0) as u32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            let xr = dx * cos_r + dy * sin_r;
            let yr = -dx * sin_r + dy * cos_r;
            let inside = xr.abs() <= half_w && yr.abs() <= half_h;
            let border = xr.abs() >= half_w - 5.0 || yr.abs() >= half_h - 5.0;

            if inside && (!hollow || border) {
                let t = ((xr + half_w) / (half_w * 2.0)).clamp(0.0, 1.0);
                let tone = mix_rgb(color, tint, t);
                blend_pixel(
                    canvas,
                    px,
                    py,
                    Rgba([tone[0], tone[1], tone[2], (opacity * 255.0) as u8]),
                );
            }
        }
    }
}

fn draw_line_shape(canvas: &mut RgbaImage, shape: &ShapePlan, color: [u8; 3], opacity: f32) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;
    let min_dim = width.min(height);
    let cx = shape.x.clamp(0.02, 0.98) * width;
    let cy = shape.y.clamp(0.02, 0.98) * height;
    let length = (shape.size.clamp(0.05, 0.8) * min_dim * 0.7).max(18.0);
    let thickness = (shape.aspect.clamp(0.1, 1.2) * 12.0).max(2.0);
    let radians = shape.rotation.to_radians();
    let x1 = cx - radians.cos() * length * 0.5;
    let y1 = cy - radians.sin() * length * 0.5;
    let x2 = cx + radians.cos() * length * 0.5;
    let y2 = cy + radians.sin() * length * 0.5;

    let min_x = x1.min(x2).floor().max(0.0) as u32;
    let max_x = x1.max(x2).ceil().min(width - 1.0) as u32;
    let min_y = y1.min(y2).floor().max(0.0) as u32;
    let max_y = y1.max(y2).ceil().min(height - 1.0) as u32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dist = distance_to_segment(px as f32, py as f32, x1, y1, x2, y2);
            if dist <= thickness {
                let alpha = (1.0 - dist / thickness).clamp(0.0, 1.0) * opacity;
                blend_pixel(
                    canvas,
                    px,
                    py,
                    Rgba([color[0], color[1], color[2], (alpha * 255.0) as u8]),
                );
            }
        }
    }
}

fn draw_ring(
    canvas: &mut RgbaImage,
    shape: &ShapePlan,
    color: [u8; 3],
    tint: [u8; 3],
    opacity: f32,
) {
    let width = canvas.width() as f32;
    let height = canvas.height() as f32;
    let min_dim = width.min(height);
    let radius = (shape.size.clamp(0.05, 0.65) * min_dim * 0.36).max(14.0);
    let thickness = (shape.aspect.clamp(0.2, 1.6) * 18.0).max(4.0);
    let cx = shape.x.clamp(0.02, 0.98) * width;
    let cy = shape.y.clamp(0.02, 0.98) * height;

    let min_x = (cx - radius - thickness).floor().max(0.0) as u32;
    let max_x = (cx + radius + thickness).ceil().min(width - 1.0) as u32;
    let min_y = (cy - radius - thickness).floor().max(0.0) as u32;
    let max_y = (cy + radius + thickness).ceil().min(height - 1.0) as u32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let edge_distance = (dist - radius).abs();
            if edge_distance <= thickness {
                let alpha = (1.0 - edge_distance / thickness).clamp(0.0, 1.0) * opacity;
                let tone = mix_rgb(color, tint, (dx.atan2(dy).sin() + 1.0) * 0.5);
                blend_pixel(
                    canvas,
                    px,
                    py,
                    Rgba([tone[0], tone[1], tone[2], (alpha * 255.0) as u8]),
                );
            }
        }
    }
}

fn render_audio_layer(layer: &AudioLayerPlan, bpm: u16, t: f32) -> (f32, f32) {
    let notes = if layer.notes.is_empty() {
        vec![0, 4, 7, 11]
    } else {
        layer.notes.clone()
    };
    let rhythm = if layer.rhythm.is_empty() {
        vec![1.0, 0.5, 0.5, 1.5]
    } else {
        layer.rhythm.clone()
    };

    let seconds_per_beat = 60.0 / bpm.clamp(50, 180) as f32;
    let pattern_duration = rhythm.iter().copied().sum::<f32>() * seconds_per_beat;
    let wrapped = if pattern_duration > 0.0 {
        t % pattern_duration
    } else {
        0.0
    };

    let mut cursor = 0.0f32;
    let mut event_index = 0usize;
    let mut event_duration = seconds_per_beat;
    for (index, duration_beats) in rhythm.iter().enumerate() {
        let duration_seconds = duration_beats.max(0.15) * seconds_per_beat;
        if wrapped <= cursor + duration_seconds {
            event_index = index;
            event_duration = duration_seconds;
            break;
        }
        cursor += duration_seconds;
    }

    let local_time = (wrapped - cursor).max(0.0);
    let note = notes[event_index % notes.len()];
    let frequency = note_to_frequency(note, layer.octave.clamp(2, 6));
    let envelope = envelope(local_time, event_duration);
    let phase = local_time * frequency;
    let dry = match layer.wave {
        Waveform::Sine => (phase * PI * 2.0).sin(),
        Waveform::Triangle => (phase * PI * 2.0).sin().asin() * (2.0 / PI),
        Waveform::Square => {
            if (phase * PI * 2.0).sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Saw => {
            let frac = phase.fract();
            frac * 2.0 - 1.0
        }
    };

    let filtered = dry * envelope * layer.gain.clamp(0.05, 0.6);
    let pan = layer.pan.clamp(-1.0, 1.0);
    let left = filtered * (1.0 - pan).sqrt();
    let right = filtered * (1.0 + pan).sqrt();
    (left, right)
}

fn envelope(local_time: f32, duration: f32) -> f32 {
    let attack = (duration * 0.12).max(0.015);
    let release = (duration * 0.18).max(0.03);
    let sustain_end = (duration - release).max(attack);

    if local_time < attack {
        local_time / attack
    } else if local_time > sustain_end {
        ((duration - local_time) / release).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn note_to_frequency(note: i32, octave: i32) -> f32 {
    let note = note.clamp(0, 11);
    let midi = 12 * (octave + 1) + note;
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}

fn distance_to_segment(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let denom = dx * dx + dy * dy;
    if denom <= f32::EPSILON {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = (((px - x1) * dx + (py - y1) * dy) / denom).clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

fn blend_pixel(canvas: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    let dst = canvas.get_pixel_mut(x, y);
    let alpha = color.0[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    for channel in 0..3 {
        dst.0[channel] = (color.0[channel] as f32 * alpha + dst.0[channel] as f32 * inv) as u8;
    }
}

fn parse_hex_color(value: &str) -> [u8; 3] {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return [96, 111, 132];
    }

    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match (parse(0..2), parse(2..4), parse(4..6)) {
        (Some(r), Some(g), Some(b)) => [r, g, b],
        _ => [96, 111, 132],
    }
}

fn blend_with_reference(base: &str, reference: Option<&String>, amount: f32) -> String {
    if let Some(reference) = reference {
        blend_hex_colors(base, reference, amount)
    } else {
        base.to_string()
    }
}

fn blend_hex_colors(left: &str, right: &str, amount: f32) -> String {
    let mixed = mix_rgb(
        parse_hex_color(left),
        parse_hex_color(right),
        amount.clamp(0.0, 1.0),
    );
    format!("#{:02X}{:02X}{:02X}", mixed[0], mixed[1], mixed[2])
}

fn mix_rgb(left: [u8; 3], right: [u8; 3], t: f32) -> [u8; 3] {
    let blend = t.clamp(0.0, 1.0);
    [
        (left[0] as f32 * (1.0 - blend) + right[0] as f32 * blend) as u8,
        (left[1] as f32 * (1.0 - blend) + right[1] as f32 * blend) as u8,
        (left[2] as f32 * (1.0 - blend) + right[2] as f32 * blend) as u8,
    ]
}
