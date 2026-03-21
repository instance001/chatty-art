<script setup lang="ts">
import { ElProgress, ElIcon } from 'element-plus';
import { Right } from '@element-plus/icons-vue';
import { useEllipsis } from '@/utils/useEllipsis';

const { ellipsis } = useEllipsis();

defineProps<{
    generated?: number;
    total?: number;
    elapsed?: string;
}>();

defineEmits(["showGenerated"]);
</script>

<template>
    <div style="text-align: center;">
        <el-progress
            type="circle"
            :percentage="100 * (generated ?? 0) / (total ?? 1)"
            :width="200"
        >
            <template #default>
                <span>{{ generated }} / {{ total }}</span><br>
            </template>
        </el-progress>
        <div class="gen-text">
            <span v-if="generated === total">All done!</span>
            <span v-else>Generating{{ellipsis}}{{'&nbsp;'.repeat(3 - ellipsis.length)}}<br><sup>{{ elapsed }}</sup></span>
        </div>
        <div @click="$emit('showGenerated')" v-if="generated" class="view-images">
            <span>View image{{ total === 1 ? "" : "s" }}</span>
            <el-icon><Right /></el-icon>
        </div>
    </div>
</template>

<style scoped>
.view-images, .gen-text {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--el-color-info);
    font-weight: 500;
    margin-top: 8px;
    gap: 8px;
}

.gen-text {
    font-weight: 400;
}

.view-images:hover {
    cursor: pointer;
    text-decoration: underline;
}
</style>