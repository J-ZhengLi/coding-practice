<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import * as monaco from 'monaco-editor';

const props = defineProps<{
  userCode: string;
  originalCode: string;
  language: string;
}>();

const containerRef = ref<HTMLElement | null>(null);
let diffEditor: monaco.editor.IDiffEditor | null = null;
let originalModel: monaco.editor.ITextModel | null = null;
let modifiedModel: monaco.editor.ITextModel | null = null;

const languageMap: Record<string, string> = {
  python: 'python',
  rust: 'rust',
  go: 'go',
  cpp: 'cpp',
};

const initDiffEditor = () => {
  if (!containerRef.value) return;

  const lang = languageMap[props.language] || 'plaintext';

  originalModel = monaco.editor.createModel(props.originalCode, lang);
  modifiedModel = monaco.editor.createModel(props.userCode, lang);

  diffEditor = monaco.editor.createDiffEditor(containerRef.value, {
    readOnly: true,
    renderSideBySide: true,
    theme: 'vs',
    fontSize: 14,
    scrollBeyondLastLine: false,
    automaticLayout: true,
  });

  diffEditor.setModel({
    original: originalModel,
    modified: modifiedModel,
  });
};

const cleanup = () => {
  if (diffEditor) {
    diffEditor.dispose();
    diffEditor = null;
  }
  if (originalModel) {
    originalModel.dispose();
    originalModel = null;
  }
  if (modifiedModel) {
    modifiedModel.dispose();
    modifiedModel = null;
  }
};

onMounted(() => {
  initDiffEditor();
});

onBeforeUnmount(() => {
  cleanup();
});

watch(
  () => [props.userCode, props.originalCode, props.language],
  () => {
    cleanup();
    initDiffEditor();
  }
);
</script>

<template>
  <div ref="containerRef" class="solution-comparison"></div>
</template>

<style scoped>
.solution-comparison {
  min-height: 400px;
  max-height: 600px;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  overflow: hidden;
}
</style>