## Interface Documentation

```python
import outetts

# Initialize the interface
interface = outetts.Interface(
    config=outetts.ModelConfig.auto_config(
        model=outetts.Models.VERSION_1_0_SIZE_1B,
        # For llama.cpp backend
        backend=outetts.Backend.LLAMACPP,
        quantization=outetts.LlamaCppQuantization.FP16
        # For transformers backend
        # backend=outetts.Backend.HF,
    )
)

# Load the default speaker profile
speaker = interface.load_default_speaker("EN-FEMALE-1-NEUTRAL")

# Or create your own speaker profiles in seconds and reuse them instantly
# speaker = interface.create_speaker("path/to/audio.wav")
# interface.save_speaker(speaker, "speaker.json")
# speaker = interface.load_speaker("speaker.json")

# Generate speech
output = interface.generate(
    config=outetts.GenerationConfig(
        text="Hello, how are you doing?",
        generation_type=outetts.GenerationType.CHUNKED,
        speaker=speaker,
        sampler_config=outetts.SamplerConfig(
            temperature=0.4
        ),
    )
)

# Save to file
output.save("output.wav")
```

## Configuration Options

### Model Configuration

You can configure the model in two ways:

#### 1. Using auto_config (Recommended)

```python
config = outetts.ModelConfig.auto_config(
    model=outetts.Models.VERSION_1_0_SIZE_1B,
    backend=outetts.Backend.LLAMACPP,
    quantization=outetts.LlamaCppQuantization.FP16
)
```

#### 2. Manual Configuration

```python
config = outetts.ModelConfig(
    model_path="OuteAI/Llama-OuteTTS-1.0-1B",
    tokenizer_path="OuteAI/Llama-OuteTTS-1.0-1B",
    interface_version=outetts.InterfaceVersion.V3,
    backend=outetts.Backend.HF,
    additional_model_config={
        "attn_implementation": "flash_attention_2"  # Enable flash attention if compatible
    },
    device="cuda",
    dtype=torch.bfloat16
)
```

### Available Models

| Model | Description |
|-------|-------------|
| `VERSION_0_1_SIZE_350M` | OuteTTS v0.1 (350M parameters) |
| `VERSION_0_2_SIZE_500M` | OuteTTS v0.2 (500M parameters) |
| `VERSION_0_3_SIZE_500M` | OuteTTS v0.3 (500M parameters) |
| `VERSION_0_3_SIZE_1B` | OuteTTS v0.3 (1B parameters) |
| `VERSION_1_0_SIZE_1B` | Llama-OuteTTS v1.0 (1B parameters) |
| `VERSION_1_0_SIZE_0_6B` | OuteTTS v1.0 (0.6B parameters) |

### Backend Options

| Backend                      | Description                                                                                                |
|------------------------------|------------------------------------------------------------------------------------------------------------|
| `Backend.HF`                 | Hugging Face Transformers backend                                                                          |
| `Backend.LLAMACPP`           | LLAMA.cpp backend (for GGUF models)                                                                        |
| `Backend.EXL2`               | EXL2 backend                                                                                               |
| `Backend.EXL2ASYNC`          | EXL2 backend with asynchronous batch processing                                                            |
| `Backend.VLLM`               | VLLM backend for efficient batch processing (Experimental)                                                 |
| `Backend.LLAMACPP_SERVER`    | Direct connection to a running LLAMA.cpp server (synchronous stream)                                       |
| `Backend.LLAMACPP_ASYNC_SERVER`| Direct connection to a running LLAMA.cpp server (asynchronous batch)                                       |

**Note on VLLM Backend:** The `Backend.VLLM` option is currently experimental and may sometimes result in missing audio chunks or static noise during generation. To enable this backend, you must set the environment variable `OUTETTS_ALLOW_VLLM=1` before running your application.

### Interface Versions

OuteTTS supports different interface versions for different model generations:

| Interface Version | Supported Models |
|-------------------|-----------------|
| `InterfaceVersion.V1` | OuteTTS v0.1 models (VERSION_0_1_SIZE_350M) |
| `InterfaceVersion.V2` | OuteTTS v0.2 and v0.3 models (VERSION_0_2_SIZE_500M, VERSION_0_3_SIZE_500M, VERSION_0_3_SIZE_1B) |
| `InterfaceVersion.V3` | OuteTTS v1.0 models (VERSION_1_0_SIZE_1B) |


### LLAMA.cpp Quantization Options

For the LLAMA.cpp backend, you can choose from multiple quantization options:

```python
quantization=outetts.LlamaCppQuantization.FP16  # No quantization
quantization=outetts.LlamaCppQuantization.Q8_0  # 8-bit quantization
# Other options: Q6_K, Q5_K_S, Q5_K_M, Q5_1, Q5_0, Q4_K_S, Q4_K_M, Q4_1, Q4_0, Q3_K_S, Q3_K_M, Q3_K_L, Q2_K
```

## Speaker Profiles

### Using Default Speakers

```python
# List available default speakers
interface.print_default_speakers()

# Load a default speaker
speaker = interface.load_default_speaker("EN-FEMALE-1-NEUTRAL")
```

### Creating Custom Speaker Profiles

To create a speaker profile, point to your audio file. The system will use the Whisper model to transcribe the audio and audio codec model to generate a profile. Once done, save the speaker to a JSON file so you can reuse it later.
> [!NOTE]
> This step only needs to be run once per speaker. It is meant for initial profile creation.

```python
speaker = interface.create_speaker("path/to/audio.wav")
interface.save_speaker(speaker, "my_speaker.json")
```

You can then load your custom speaker from the JSON file at any time:

```python
speaker = interface.load_speaker("my_speaker.json")
```

For older model versions, you can also provide a transcript:

```python
speaker = interface.create_speaker(
    audio_path="path/to/audio.wav",
    transcript="What is being said in the audio",
    whisper_model="turbo",
    whisper_device="cuda"
)
```

### Decoding and Saving Speaker Audio

For V3 interface models, you can decode and save the speaker audio:

```python
interface.decode_and_save_speaker(speaker, "speaker_audio.wav")
```

## Generation Options

### Generation Types

| Type                      | Description                                                                                                |
|---------------------------|------------------------------------------------------------------------------------------------------------|
| `GenerationType.REGULAR`  | Standard generation for shorter texts                                                                      |
| `GenerationType.CHUNKED`  | Splits text into chunks for better processing of longer texts (recommended for single-text generation)     |
| `GenerationType.GUIDED_WORDS`| Experimental guided generation by words                                                                    |
| `GenerationType.STREAM`   | Streaming generation **(Not Implemented)**                                                                 |
| `GenerationType.BATCH`    | Optimized batch generation for processing multiple text chunks simultaneously (requires specific backends) |

### Sampler Configuration

You can customize the generation parameters:

```python
sampler_config = outetts.SamplerConfig(
    temperature=0.4,
    repetition_penalty=1.1,
    top_k=40,
    top_p=0.9,
    min_p=0.05,
    mirostat=False,
    mirostat_tau=5,
    mirostat_eta=0.1
)
```

### Complete Generation Example

```python
output = interface.generate(
    config=outetts.GenerationConfig(
        text="Hello, how are you doing today? I hope you're having a wonderful day!",
        speaker=speaker,
        generation_type=outetts.GenerationType.CHUNKED,
        sampler_config=outetts.SamplerConfig(
            temperature=0.4,
            top_k=40,
            top_p=0.9
        ),
        max_length=8192
    )
)

# Play the audio
output.play()

# Save to file
output.save("output.wav")
```

### Batch Generation

For backends optimized for batch processing (`EXL2ASYNC`, `VLLM`, `LLAMACPP_ASYNC_SERVER`), you can leverage `GenerationType.BATCH`. This mode processes multiple text chunks in parallel, which can significantly improve throughput for longer texts or when generating multiple audio outputs.

When using these backends, `GenerationType.BATCH` is the only supported type and will be automatically selected even if a different type is specified in `GenerationConfig`.

```python
from outetts import Interface, ModelConfig, GenerationConfig, Backend, GenerationType

if __name__ == "__main__":
    # Initialize the interface with a batch-capable backend
    interface = Interface(
        ModelConfig(
            model_path="OuteAI/Llama-OuteTTS-1.0-0.6B-FP8", # Replace with your model path
            tokenizer_path="OuteAI/Llama-OuteTTS-1.0-0.6B", # Replace with your tokenizer path
            backend=Backend.VLLM
            # For EXL2, use backend=Backend.EXL2ASYNC + exl2_cache_seq_multiply={should be same as max_batch_size in GenerationConfig}
            # For LLAMACPP_ASYNC_SERVER, use backend=Backend.LLAMACPP_ASYNC_SERVER and provide server_host in GenerationConfig
        )
    )

    # Load your speaker profile
    speaker = interface.load_default_speaker("EN-FEMALE-1-NEUTRAL") # Or load/create custom speaker

    # Generate speech using BATCH type
    # Note: For EXL2ASYNC, VLLM, LLAMACPP_ASYNC_SERVER, BATCH is automatically selected.
    output = interface.generate(
        GenerationConfig(
            text="This is a longer text that will be automatically split into chunks and processed in batches.",
            speaker=speaker,
            generation_type=GenerationType.BATCH, # Can often be omitted for batch backends
            max_batch_size=32,       # Adjust based on your GPU memory and server capacity
            dac_decoding_chunk=2048, # Adjust chunk size for DAC decoding
            # If using LLAMACPP_ASYNC_SERVER, add:
            # server_host="http://localhost:8000" # Replace with your server address
        )
    )

    # Save to file
    output.save("output_batch.wav")
```

### Advanced Configuration Options

#### Passing Custom Settings to Backends

OuteTTS allows you to pass custom settings directly to the underlying model backends:

##### Custom Model Initialization Parameters

The `additional_model_config` parameter lets you pass any backend-specific settings when initializing the model:

```python
interface = outetts.Interface(
    config=outetts.ModelConfig(
        model_path="OuteAI/Llama-OuteTTS-1.0-1B",
        # ... other settings
        additional_model_config={
            "attn_implementation": "flash_attention_2",  # HF-specific setting
            "device_map": "auto"  # For multi-GPU setups
            # Any other backend-specific settings
        }
    )
)
```

These parameters are passed directly to the model initialization function of the corresponding backend.

##### Custom Generation Parameters

You can customize the generation process further using these options:

```python
output = interface.generate(
    config=outetts.GenerationConfig(
        text="Hello, this is a test",
        speaker=speaker,
        # Standard generation options
        generation_type=outetts.GenerationType.CHUNKED,
        sampler_config=outetts.SamplerConfig(temperature=0.4),
        max_length=8192,
        # Custom backend-specific generation settings
        additional_gen_config={
            "frequency_penalty": 1.0,
            "presence_penalty": 0.5,
            # Any other backend-specific generation parameters
        }
    )
)
```

The `additional_gen_config` dictionary is passed directly to the model's generation function, allowing you to use any parameters supported by the specific backend.

## Advanced Usage

### Using with Hugging Face Models and Flash Attention

```python
import outetts
import torch

interface = outetts.Interface(
    config=outetts.ModelConfig(
        model_path="OuteAI/Llama-OuteTTS-1.0-1B",
        tokenizer_path="OuteAI/Llama-OuteTTS-1.0-1B",
        interface_version=outetts.InterfaceVersion.V3,
        backend=outetts.Backend.HF,
        additional_model_config={
            "attn_implementation": "flash_attention_2"  # Enable flash attention if compatible
        },
        device="cuda",
        dtype=torch.bfloat16
    )
)
```

### Determining Compatible Data Type

The library can automatically detect the best dtype for your hardware:

```python
from outetts.models.config import get_compatible_dtype
import torch

dtype = get_compatible_dtype()  # Returns torch.bfloat16, torch.float16, or torch.float32
```
