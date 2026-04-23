<script setup lang="ts">
interface Props {
  config: {
    preferredLanguage: string;
    skillLevel: string;
    dailyQuotas: {
      python: number;
      rust: number;
      go: number;
      cpp: number;
    };
    aiModel: string;
    aiModelType: 'local' | 'api';
  };
}

interface Emits {
  (e: 'finish'): void;
  (e: 'back'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const finish = () => {
  emit('finish');
};

const goBack = () => {
  emit('back');
};

const skillLevelLabels: Record<string, string> = {
  beginner: 'Beginner',
  intermediate: 'Intermediate',
  advanced: 'Advanced',
};

const languageLabels: Record<string, string> = {
  python: 'Python',
  rust: 'Rust',
  go: 'Go',
  cpp: 'C++',
};
</script>

<template>
  <div class="review-and-confirm">
    <h2 class="step-title">Review and Confirm</h2>
    <p class="step-description">Please review your configuration before saving.</p>

    <div class="review-card">
      <div class="review-section">
        <h3>Preferred Language</h3>
        <p>{{ languageLabels[config.preferredLanguage] || config.preferredLanguage }}</p>
      </div>

      <div class="review-section">
        <h3>Skill Level</h3>
        <p>{{ skillLevelLabels[config.skillLevel] || config.skillLevel }}</p>
      </div>

      <div class="review-section">
        <h3>Daily Exercise Quotas</h3>
        <ul>
          <li v-for="(quota, lang) in config.dailyQuotas" :key="lang">
            <strong>{{ languageLabels[lang] || lang }}:</strong> {{ quota }} exercises/day
          </li>
        </ul>
      </div>

      <div class="review-section">
        <h3>AI Model</h3>
        <p>
          <span class="model-name">{{ config.aiModel }}</span>
          <span class="model-badge" :class="config.aiModelType">
            {{ config.aiModelType }}
          </span>
        </p>
      </div>
    </div>

    <div class="button-group">
      <button class="btn btn-secondary" @click="goBack">Back</button>
      <button class="btn btn-primary" @click="finish">
        Complete Configuration
      </button>
    </div>
  </div>
</template>

<style scoped>
.review-and-confirm {
  max-width: 600px;
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

.review-card {
  background-color: #f9fafb;
  border: 2px solid #e5e7eb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  margin-bottom: 2rem;
}

.review-section {
  padding: 1rem 0;
  border-bottom: 1px solid #e5e7eb;
}

.review-section:last-child {
  border-bottom: none;
}

.review-section h3 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.5rem;
}

.review-section p,
.review-section ul {
  font-size: 1.1rem;
  color: #1f2937;
  margin: 0;
}

.review-section ul {
  list-style: none;
  padding: 0;
}

.review-section li {
  padding: 0.5rem 0;
}

.model-name {
  font-weight: 600;
}

.model-badge {
  display: inline-block;
  margin-left: 0.5rem;
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.model-badge.local {
  background-color: #d1fae5;
  color: #065f46;
}

.model-badge.api {
  background-color: #dbeafe;
  color: #1e40af;
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

.btn-primary:hover {
  background-color: #2563eb;
}
</style>