import argparse
import json
import random
import sys
from pathlib import Path

import soundfile as sf


def parse_args():
    parser = argparse.ArgumentParser(description="Chatty-art Stable Audio runner")
    parser.add_argument("--request", required=True, help="Path to a JSON request file.")
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def add_stable_audio_source_to_path() -> Path:
    source_dir = (
        repo_root()
        / "audio_runtime"
        / "stable_audio_tools"
        / "stable-audio-tools-main"
    )
    if not source_dir.exists():
        raise FileNotFoundError(
            f"Stable Audio Tools source tree not found at '{source_dir}'."
        )
    sys.path.insert(0, str(source_dir))
    return source_dir


def load_request(path: str) -> dict:
    with open(path, "r", encoding="utf-8-sig") as handle:
        return json.load(handle)


def configure_seed(seed: int | None):
    if seed is None:
        return
    random.seed(seed)
    try:
        import numpy as np

        np.random.seed(seed)
    except Exception:
        pass
    try:
        import torch

        torch.manual_seed(seed)
    except Exception:
        pass


def main():
    args = parse_args()
    add_stable_audio_source_to_path()

    import torch
    from einops import rearrange
    from stable_audio_tools.inference.generation import generate_diffusion_cond
    from stable_audio_tools.models.factory import create_model_from_config
    from stable_audio_tools.models.utils import copy_state_dict, load_ckpt_state_dict

    request = load_request(args.request)
    configure_seed(request.get("seed"))

    model_dir = Path(request["model_dir"])
    model_config = json.loads((model_dir / "model_config.json").read_text("utf-8"))
    model = create_model_from_config(model_config)
    copy_state_dict(model, load_ckpt_state_dict(str(model_dir / "model.safetensors")))

    device = "cuda" if torch.cuda.is_available() else "cpu"
    model = model.to(device).eval().requires_grad_(False)
    if device == "cuda" and bool(request.get("low_vram_mode", False)):
        model = model.to(torch.float16)

    sample_rate = int(model_config["sample_rate"])
    sample_size = int(model_config["sample_size"])
    duration_seconds = max(1, int(request.get("duration_seconds", 10)))
    target_samples = min(sample_size, duration_seconds * sample_rate)

    prompt = str(request["prompt"]).strip()
    negative_prompt = str(request.get("negative_prompt") or "").strip()

    conditioning = [
        {
            "prompt": prompt,
            "seconds_start": 0,
            "seconds_total": duration_seconds,
        }
    ]
    negative_conditioning = None
    if negative_prompt:
        negative_conditioning = [
            {
                "prompt": negative_prompt,
                "seconds_start": 0,
                "seconds_total": duration_seconds,
            }
        ]

    with torch.inference_mode():
        output = generate_diffusion_cond(
            model,
            steps=int(request.get("steps", 100)),
            cfg_scale=float(request.get("cfg_scale", 7.0)),
            conditioning=conditioning,
            negative_conditioning=negative_conditioning,
            sample_size=sample_size,
            sigma_min=0.3,
            sigma_max=500,
            sampler_type="dpmpp-3m-sde",
            seed=int(request.get("seed", -1)),
            device=device,
        )

    output = output[:, :, :target_samples]
    output = rearrange(output, "b d n -> d (b n)")
    output = output.to(torch.float32)

    peak = torch.max(torch.abs(output))
    if torch.isfinite(peak) and peak > 0:
        output = output.div(peak)

    waveform = output.clamp(-1, 1).cpu().numpy().T

    output_path = Path(request["output_path"])
    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_path.suffix.lower() != ".wav":
        output_path = output_path.with_suffix(".wav")
    sf.write(str(output_path), waveform, sample_rate, subtype="PCM_16")

    print(
        json.dumps(
            {
                "output_path": str(output_path),
                "duration_seconds": duration_seconds,
                "sample_rate": sample_rate,
            }
        )
    )


if __name__ == "__main__":
    main()
