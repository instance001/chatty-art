<script setup lang="ts">
import {
    ElButton,
    ElUpload,
    ElIcon,
    ElImage,
    type UploadFile,
    type UploadRawFile,
} from 'element-plus';
import {
    UploadFilled,
    Refresh,
} from '@element-plus/icons-vue';
import { computed, ref } from 'vue';
import { useInterrogationStore } from '@/stores/interrogation';
import { convertToBase64 } from '@/utils/base64';
import { useGeneratorStore } from '@/stores/generator';
import { useUIStore } from '@/stores/ui';
import { useEllipsis } from '@/utils/useEllipsis';

const store = useInterrogationStore();
const genStore = useGeneratorStore();
const uiStore = useUIStore();

const upload = ref();

async function handleChange(uploadFile: UploadFile) {
    upload.value!.clearFiles();
    if (!(uploadFile.raw as UploadRawFile).type.includes("image")) {
        uiStore.raiseError("Uploaded file needs to be a image!", false);
        return;
    }
    const base64File = await convertToBase64(uploadFile.raw as UploadRawFile) as string;
    store.currentInterrogation.source_image = base64File;
    store.interrogateImage();
}

function useInterrogationCaption() {
    genStore.generateText2Img({
        prompt: captionForm.value,
    } as any);
}

const captionForm = computed(() => store.getFormStatus());

const { ellipsis } = useEllipsis();
</script>

<template>
    <div v-if="!store.currentInterrogation.source_image" style="margin-top: 16px;">
        <div>
            <el-upload
                @change="handleChange"
                :auto-upload="false"
                :limit="1"
                class="interrogation-upload"
                ref="upload"
                multiple
                drag
            >
                <el-icon :size="100"><upload-filled /></el-icon>
                <div>Drop file here OR <em>click to upload</em></div>
            </el-upload>
        </div>
    </div>
    <div v-else-if="!store.currentInterrogation.status" style="margin-top: 16px;">
        <strong>Uploading image{{ellipsis}}</strong>
    </div>
    <div v-else>
        <div style="margin-top: 8px">
            <el-button :icon="Refresh" @click="store.resetInterrogation">New Interrogation</el-button>
            <el-button :icon="Refresh" @click="useInterrogationCaption" :disabled="!captionForm" v-if="captionForm">Text2Img (Caption)</el-button>
        </div>
        <h2 style="margin: 16px 0 8px 0;">Interrogation Results</h2>
        <el-image :src="store.currentInterrogation.source_image" alt="Uploaded Image" />
        <div v-if="captionForm">
            <h3>Caption</h3>
            <div v-if="!captionForm">Processing{{ellipsis}}</div>
            <div v-else><strong>{{ captionForm }}</strong></div>
        </div>
    </div>
</template>

<style scoped>
h3 {
    margin-bottom: 0;
}

.interrogation-upload {
    max-width: 720px;
}
</style>