import os
import polars as pl
import torch
from tqdm import tqdm
from outetts.wav_tokenizer.audio_codec import AudioCodec
from outetts.version.v2.prompt_processor import PromptProcessor
from outetts.version.v2.alignment import CTCForcedAlignment
import io
from loguru import logger

class DataCreation:
    def __init__(
            self, 
            model_tokenizer_path: str, 
            audio_files_path: str,
            save_dir: str,
            save_len: int = 5000,
        ):

        self.device = "cuda"
        self.audio_codec = AudioCodec(
            device=self.device,
            load_decoder=False
        )
        self.prompt_processor = PromptProcessor(model_tokenizer_path)
        self.files = self.get_files(audio_files_path, ".parquet")
        self.ctc = CTCForcedAlignment(self.device)

        self.save_dir = save_dir
        self.save_len = save_len
        self.save_id = 0
        self.data = []

    def get_files(self, folder_path, extension_filter=None):
        if isinstance(extension_filter, str):
            extension_filter = [extension_filter]
        matching_files = []
        for root, _, files in os.walk(folder_path):
            for file in files:
                if extension_filter:
                    if any(file.endswith(ext) for ext in extension_filter):
                        matching_files.append(os.path.join(root, file))
                else:
                    matching_files.append(os.path.join(root, file))
        return matching_files
    
    def create_speaker(self, audio, transcript: str):
        words = self.ctc.align(audio, transcript)

        full_codes = self.audio_codec.encode(
            self.audio_codec.convert_audio_tensor(
                audio=torch.cat([i["audio"] for i in words], dim=1),
                sr=self.ctc.sample_rate
            ).to(self.audio_codec.device)
        ).tolist()

        data = []
        start = 0
        for i in words:
            end = int(round((i["x1"] / self.ctc.sample_rate) * 75))
            word_tokens = full_codes[0][0][start:end]
            start = end
            if not word_tokens:
                word_tokens = [1]

            data.append({
                "word": i["word"],
                "duration": round(len(word_tokens) / 75, 2),
                "codes": word_tokens
            })

        return {
            "text": transcript,
            "words": data,
        }
    
    def save(self):
        os.makedirs(self.save_dir, exist_ok=True)
        path = os.path.join(self.save_dir, f"{self.save_id:06d}.parquet")
        logger.info(f"Saving data: {path}")
        pl.DataFrame(self.data).write_parquet(path)
        self.data = []
        self.save_id += 1

    def run(self):
        for i in self.files:
            df = pl.read_parquet(i)
            for data in tqdm(df.iter_rows(named=True), total=len(df)):
                try:
                    transcript = data['transcript'] 
                    audio = io.BytesIO(data['audio']['bytes'])
                    speaker = self.create_speaker(audio, transcript)
                    prompt = self.prompt_processor.get_training_prompt(speaker)

                    self.data.append({
                        'prompt': prompt,
                    })

                    if len(self.data) == self.save_len:
                        self.save()

                except Exception as e:
                    logger.error(
                        f"{e}\n\n"
                        "Occasional exceptions are expected due to potential inaccuracies or issues in the audio files. "
                        "This is normal and may not indicate a critical problem, but a high frequency of errors might suggest an underlying issue that needs attention."
                    )

        if self.data:
            self.save() 
