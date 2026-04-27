<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  accept?: string;
  disabled?: boolean;
}

interface Emits {
  (e: 'file-selected', file: File): void;
}

const props = withDefaults(defineProps<Props>(), {
  accept: '.json',
  disabled: false,
});
const emit = defineEmits<Emits>();

const isDragging = ref(false);
const selectedFile = ref<File | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);

const onDragOver = (event: DragEvent) => {
  event.preventDefault();
  if (!props.disabled) isDragging.value = true;
};

const onDragLeave = () => {
  isDragging.value = false;
};

const onDrop = (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = false;
  if (props.disabled) return;

  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    selectedFile.value = files[0];
    emit('file-selected', files[0]);
  }
};

const onFileInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    selectedFile.value = target.files[0];
    emit('file-selected', target.files[0]);
  }
};

const openFilePicker = () => {
  if (!props.disabled) fileInput.value?.click();
};

const clearFile = () => {
  selectedFile.value = null;
  if (fileInput.value) fileInput.value.value = '';
};
</script>

<template>
  <div>
    <div
      @click="openFilePicker"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
      :class="[
        'border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors',
        disabled ? 'border-gray-200 bg-gray-50 cursor-not-allowed opacity-60' :
        isDragging ? 'border-blue-400 bg-blue-50' :
        selectedFile ? 'border-green-400 bg-green-50' :
        'border-gray-300 hover:border-blue-400 hover:bg-blue-50'
      ]"
    >
      <input
        ref="fileInput"
        type="file"
        :accept="accept"
        @change="onFileInput"
        class="hidden"
        :disabled="disabled"
      />

      <div v-if="selectedFile">
        <svg class="w-8 h-8 mx-auto mb-2 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
        <p class="text-sm font-medium text-green-700">{{ selectedFile.name }}</p>
        <p class="text-xs text-gray-500 mt-1">{{ (selectedFile.size / 1024).toFixed(1) }} KB</p>
        <button @click.stop="clearFile" class="mt-2 text-xs text-red-500 hover:text-red-700 underline">
          Remove file
        </button>
      </div>

      <div v-else>
        <svg class="w-8 h-8 mx-auto mb-2 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
        <p class="text-sm text-gray-500">Drag and drop a file here, or click to browse</p>
        <p class="text-xs text-gray-400 mt-1">Accepted: {{ accept }}</p>
      </div>
    </div>
  </div>
</template>