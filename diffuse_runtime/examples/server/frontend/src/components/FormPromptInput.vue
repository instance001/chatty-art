<script setup lang="ts">
import { useGeneratorStore } from '@/stores/generator';
import FormInput from './FormInput.vue';
const store = useGeneratorStore();
</script>

<template>
    <form-input
        prop="prompt"
        v-model="store.prompt"
        :autosize="{ minRows: 2, maxRows: 15 }"
        resize="vertical"
        type="textarea"
        placeholder="Enter prompt here"
        label-position="top"
        label-style="justify-content: space-between; width: 100%;"
    >
        <template #label>
            <div>Prompt</div>
        </template>
    </form-input>
</template>

<style scoped>
h3 {
    margin: 0;
}

h4, h5 {
    margin-bottom: 0;
}

.tag-select {
    display: flex;
    justify-content: space-between;
    gap: 32px;
    padding: 0px 8px;
    width: 240px;
}

.tag-select:hover {
    background-color: rgb(100, 100, 100) !important;
}

.trigger-select {
    width: 30px;
    height: 30px;
}

:deep(.trigger-select .el-input__wrapper) {
    padding: 0;
}

:deep(.trigger-select .el-input__suffix, .trigger-select .el-input__suffix-inner) {
    width: 100%
}
</style>