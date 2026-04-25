<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useConfigStore } from '../stores/config';

interface Props {
  modelValue: string;
  type: string;
}

interface Emits {
  (e: 'update:modelValue', value: string): void;
  (e: 'update:type', value: string): void;
  (e: 'next'): void;
  (e: 'back'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Map model_type to badge label
const badgeLabel = (type: string) => {
  if (type === 'ollama_local' || type === 'local') return '(OLLAMA, LOCAL)';
  if (type === 'ollama_cloud' || type === 'cloud') return '(OLLAMA, CLOUD)';
  return '(API)';
};

// Map model_type to CSS class
const badgeClass = (type: string) => {
  if (type === 'ollama_local' || type === 'local') return 'ollama_local';
  if (type === 'ollama_cloud' || type === 'cloud') return 'ollama_cloud';
  return 'api';
};

const configStore = useConfigStore();
const loading = ref(true);
const ollamaModels = ref<Array<{ name: string; model_type: string }>>([]);
const error = ref<string | null>(null);

// API models (can be extended in future phases)
const apiModels = ref([
  { name: 'gpt-4o', model_type: 'api' },
  { name: 'gpt-4o-mini', model_type: 'api' },
  { name: 'claude-3-5-sonnet', model_type: 'api' },
  { name: 'claude-3-5-haiku', model_type: 'api' },
]);

const allModels = computed(() => {
  // Normalize model_type for display: map 'local' → 'ollama_local', 'api' stays 'api'
  // This handles both old and new backend responses
  const normalize = (m: { name: string; model_type: string }) => ({
    ...m,
    model_type: m.model_type === 'local' ? 'ollama_local' : m.model_type === 'cloud' ? 'ollama_cloud' : m.model_type,
  });

  // Strip "(OLLAMA, LOCAL)" / "(OLLAMA, CLOUD)" suffix from display names
  const cleanName = (m: { name: string; model_type: string }) => ({
    ...m,
    name: m.name.replace(/ \(OLLAMA, (?:LOCAL|CLOUD)\)$/, ''),
  });

  const ollamaLocal = [...ollamaModels.value]
    .map(normalize)
    .filter(m => m.model_type === 'ollama_local')
    .map(cleanName)
    .sort((a, b) => a.name.localeCompare(b.name));

  const ollamaCloud = [...ollamaModels.value]
    .map(normalize)
    .filter(m => m.model_type === 'ollama_cloud')
    .map(cleanName)
    .sort((a, b) => a.name.localeCompare(b.name));

  const apiList = [...apiModels.value]
    .sort((a, b) => a.name.localeCompare(b.name));

  return [...ollamaLocal, ...ollamaCloud, ...apiList];
});

const selectedModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const selectedType = computed({
  get: () => props.type,
  set: (value) => emit('update:type', value),
});

const canProceed = computed(() => {
  return props.modelValue.length > 0;
});

onMounted(async () => {
  try {
    ollamaModels.value = await configStore.fetchOllamaModels();
  } catch (err: any) {
    error.value = err.message || 'Failed to fetch Ollama models';
  } finally {
    loading.value = false;
  }
});

const nextStep = () => {
  if (canProceed.value) {
    emit('next');
  }
};

const goBack = () => {
  emit('back');
};

const selectModel = (model: { name: string; model_type: string }) => {
  selectedModel.value = model.name;
  selectedType.value = model.model_type;
};
</script>

<template>
  <div class="ai-model-selection">
    <h2 class="step-title">Select AI Model</h2>
    <p class="step-description">Choose the AI model that will generate and grade your exercises.</p>

    <div v-if="loading" class="loading-message">
      Loading available models...
    </div>

    <div v-else-if="error" class="error-message">
      {{ error }}
      <p class="error-note">You can still select an API model below.</p>
    </div>

    <div v-else class="model-list">
      <div
        v-for="model in allModels"
        :key="model.name"
        class="model-card"
        :class="{ selected: modelValue === model.name }"
        @click="selectModel(model)"
      >
        <div class="model-header">
          <h3>{{ model.name }}</h3>
          <span class="model-badge" :class="badgeClass(model.model_type)">
            {{ badgeLabel(model.model_type) }}
          </span>
        </div>
        <p v-if="model.model_type === 'ollama_local' || model.model_type === 'local'" class="model-description">
          Running locally on your machine via Ollama. No API costs.
        </p>
        <p v-else-if="model.model_type === 'ollama_cloud' || model.model_type === 'cloud'" class="model-description">
          Cloud-hosted Ollama model. Requires network access.
        </p>
        <p v-else class="model-description">
          API-based model. Requires API key configuration (Phase 2).
        </p>
      </div>
    </div>

    <div class="button-group">
      <button class="btn btn-secondary" @click="goBack">Back</button>
      <button class="btn btn-primary" :disabled="!canProceed" @click="nextStep">
        Next
      </button>
    </div>
  </div>
</template>

<style scoped>
.ai-model-selection {
  max-width: 700px;
  margin: 0 auto;
}

.step-title {
  font-size: 1.75rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #1f2937;
}

.step-description {
  color: #6b7280;
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

.loading-message,
.error-message {
  text-align: center;
  padding: 2rem;
  color: #6b7280;
  font-size: 1.1rem;
}

.error-message {
  color: #ef4444;
}

.error-note {
  font-size: 0.9rem;
  color: #6b7280;
  margin-top: 0.5rem;
}

.model-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-bottom: 2rem;
}

.model-card {
  padding: 1.5rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
}

.model-card:hover {
  border-color: #3b82f6;
  background-color: #f0f9ff;
}

.model-card.selected {
  border-color: #3b82f6;
  background-color: #eff6ff;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.model-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.model-header h3 {
  font-size: 1.1rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.model-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.model-badge.ollama_local {
  background-color: #d1fae5;
  color: #065f46;
}

.model-badge.ollama_cloud {
  background-color: #fef3c7;
  color: #92400e;
}

.model-badge.api {
  background-color: #dbeafe;
  color: #1e40af;
}

.model-description {
  color: #6b7280;
  font-size: 0.9rem;
  margin: 0;
}

.button-group {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}

.btn {
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
  font-size: 1rem;
}

.btn-secondary {
  background-color: #e5e7eb;
  color: #1f2937;
}

.btn-secondary:hover {
  background-color: #d1d5db;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2563eb;
}

.btn-primary:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
}
</style>