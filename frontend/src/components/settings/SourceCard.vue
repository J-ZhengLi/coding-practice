<script setup lang="ts">
import ToggleSwitch from './ToggleSwitch.vue';

interface Props {
  sourceKey: string;
  label: string;
  enabled: boolean;
  disabled: boolean;
  disabledTooltip?: string;
}

interface Emits {
  (e: 'update:enabled', value: boolean): void;
  (e: 'dragstart', event: DragEvent): void;
  (e: 'dragover', event: DragEvent): void;
  (e: 'drop', event: DragEvent): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();
</script>

<template>
  <div
    class="source-card"
    :class="{ collapsed: !enabled }"
    draggable="true"
    @dragstart="emit('dragstart', $event)"
    @dragover="emit('dragover', $event)"
    @drop="emit('drop', $event)"
  >
    <div class="card-header">
      <span class="drag-handle" title="Drag to reorder">&#9776;</span>
      <span class="source-label">{{ label }}</span>
      <div class="toggle-wrapper" :title="disabled ? disabledTooltip : undefined">
        <ToggleSwitch
          :model-value="enabled"
          :disabled="disabled"
          @update:model-value="emit('update:enabled', $event)"
        />
      </div>
    </div>
    <div v-if="enabled && sourceKey !== 'ai_generated'" class="card-body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.source-card {
  background-color: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  margin-bottom: 1.5rem;
  cursor: default;
}

.source-card.collapsed {
  opacity: 0.85;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0.75rem 1rem;
}

.drag-handle {
  cursor: grab;
  color: #9ca3af;
  user-select: none;
  font-size: 1.125rem;
  padding: 0 0.25rem;
  line-height: 1;
}

.drag-handle:hover {
  color: #6b7280;
}

.source-label {
  font-weight: 600;
  color: #1f2937;
  font-size: 0.875rem;
  margin-left: 0.25rem;
  flex: 1;
}

.toggle-wrapper {
  flex-shrink: 0;
}

.card-body {
  padding: 0 1rem 1rem 1rem;
  padding-top: 0.5rem;
  border-top: 1px solid #f3f4f6;
}
</style>
