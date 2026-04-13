import argparse
import json
import os
import random
import sys
from pathlib import Path

import soundfile as sf


def parse_args():
    parser = argparse.ArgumentParser(description="Chatty-art OuteTTS runner")
    parser.add_argument("--request", required=True, help="Path to a JSON request file.")
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def add_outetts_source_to_path() -> Path:
    source_dir = repo_root() / "audio_runtime" / "outetts" / "OuteTTS-main"
    if not source_dir.exists():
        raise FileNotFoundError(
            f"OuteTTS source tree not found at '{source_dir}'."
        )
    sys.path.insert(0, str(source_dir))
    return source_dir


def patch_torch_distributed_for_audiotools():
    try:
        import torch.distributed as dist
    except Exception:
        return

    if hasattr(dist, "ReduceOp"):
        return

    class _ReduceOp:
        AVG = "avg"
        SUM = "sum"
        MIN = "min"
        MAX = "max"
        PRODUCT = "product"

    dist.ReduceOp = _ReduceOp


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


def default_sampler(outetts, temperature: float):
    return outetts.SamplerConfig(
        temperature=float(temperature),
        repetition_penalty=1.1,
        repetition_range=64,
        top_k=40,
        top_p=0.9,
        min_p=0.05,
    )


def build_interface(outetts, request: dict):
    low_vram_mode = bool(request.get("low_vram_mode", False))
    model_path = request["model_path"]
    tokenizer_repo = request["tokenizer_repo"]

    config = outetts.ModelConfig(
        model_path=model_path,
        tokenizer_path=tokenizer_repo,
        interface_version=outetts.InterfaceVersion.V3,
        backend=outetts.Backend.LLAMACPP,
        verbose=False,
        max_seq_length=8192,
        n_gpu_layers=0 if low_vram_mode else 99,
    )
    return outetts.Interface(config=config)


def resolve_speaker(interface, request: dict):
    speaker_audio_path = request.get("speaker_audio_path")
    if speaker_audio_path:
        return interface.create_speaker(audio_path=speaker_audio_path)
    speaker_mode = str(request.get("speaker_mode", "default")).strip().lower()
    if speaker_mode in {"random", "characterized"}:
        return None
    return interface.load_default_speaker(
        request.get("default_speaker", "en-female-1-neutral")
    )


def save_output_audio(output, output_path: Path):
    audio = output.audio.detach().cpu()
    if audio.dim() == 1:
        audio = audio.unsqueeze(0)
    elif audio.dim() > 2:
        audio = audio[0] if audio.dim() == 3 else audio[0, 0]
        if audio.dim() == 1:
            audio = audio.unsqueeze(0)

    waveform = audio.transpose(0, 1).numpy()
    sf.write(str(output_path), waveform, output.sr, subtype="PCM_16")


def main():
    args = parse_args()
    add_outetts_source_to_path()
    patch_torch_distributed_for_audiotools()

    import outetts

    request = load_request(args.request)
    configure_seed(request.get("seed"))
    interface = build_interface(outetts, request)
    speaker = resolve_speaker(interface, request)

    output = interface.generate(
        config=outetts.GenerationConfig(
            text=request["text"],
            voice_characteristics=request.get("voice_characteristics"),
            speaker=speaker,
            generation_type=outetts.GenerationType.CHUNKED,
            sampler_config=default_sampler(outetts, request.get("temperature", 0.4)),
            max_length=8192,
        )
    )

    output_path = Path(request["output_path"])
    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_path.suffix.lower() != ".wav":
        output_path = output_path.with_suffix(".wav")
    save_output_audio(output, output_path)
    print(
        json.dumps(
            {
                "output_path": str(output_path),
                "speaker_mode": (
                    "reference"
                    if request.get("speaker_audio_path")
                    else str(request.get("speaker_mode", "default"))
                ),
            }
        )
    )


if __name__ == "__main__":
    main()
