<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useConfigStore } from '../stores/config';
import { useExerciseStore } from '../stores/exercise';
import { useMaterialStore } from '../stores/material';
import { useRouter } from 'vue-router';

const configStore = useConfigStore();
const exerciseStore = useExerciseStore();
const materialStore = useMaterialStore();
const router = useRouter();

const generationError = ref<string | null>(null);

onMounted(async () => {
  // Ensure configuration is loaded
  if (!configStore.config) {
    await configStore.fetchConfig();
  }

  // Redirect to configuration if not configured
  if (!configStore.isConfigured) {
    router.push({ name: 'configuration' });
    return;
  }

  // Load existing exercises and materials for dashboard display
  if (configStore.config) {
    const language = configStore.config.preferred_language;
    const difficulty = configStore.config.skill_level;
    await Promise.all([
      exerciseStore.fetchExercises(language, difficulty),
      materialStore.fetchMaterials(language, difficulty),
    ]);
  }
});

const skillLevelLabel = (level: string): string => {
  const labels: Record<string, string> = {
    beginner: 'Beginner',
    intermediate: 'Intermediate',
    advanced: 'Advanced',
  };
  return labels[level] || level;
};

const languageLabel = (lang: string): string => {
  const labels: Record<string, string> = {
    python: 'Python',
    rust: 'Rust',
    go: 'Go',
    cpp: 'C++',
  };
  return labels[lang] || lang;
};

const difficultyColor = (difficulty: string): string => {
  const colors: Record<string, string> = {
    beginner: 'bg-green-100 text-green-800',
    intermediate: 'bg-yellow-100 text-yellow-800',
    advanced: 'bg-red-100 text-red-800',
  };
  return colors[difficulty] || 'bg-gray-100 text-gray-800';
};

const languageBadgeColor = (lang: string): string => {
  const colors: Record<string, string> = {
    python: 'bg-blue-100 text-blue-800',
    rust: 'bg-orange-100 text-orange-800',
    go: 'bg-cyan-100 text-cyan-800',
    cpp: 'bg-purple-100 text-purple-800',
  };
  return colors[lang] || 'bg-gray-100 text-gray-800';
};

const materialCountByLanguage = computed(() => {
  if (!configStore.config) return 0;
  return materialStore.materialCountByLanguage(configStore.config.preferred_language);
});

const exerciseCountByLanguage = computed(() => {
  return exerciseStore.exercises.length;
});

const handleGenerateExercises = async () => {
  if (!configStore.config) return;

  generationError.value = null;
  const { preferred_language, skill_level } = configStore.config;

  const result = await exerciseStore.generateExercises(preferred_language, skill_level);
  if (!result && exerciseStore.error) {
    generationError.value = exerciseStore.error;
  }
};

const dismissError = () => {
  generationError.value = null;
  exerciseStore.error = null;
  materialStore.error = null;
};

const truncateDescription = (desc: string, maxLength: number = 100): string => {
  if (desc.length <= maxLength) return desc;
  return desc.substring(0, maxLength) + '...';
};
</script>

<template>
  <div class="dashboard-page">
    <div class="dashboard-container">
      <div class="dashboard-header">
        <h1>Dashboard</h1>
        <p>Your personalized learning space</p>
      </div>

      <!-- Error Alert -->
      <div v-if="generationError || exerciseStore.error || materialStore.error" class="error-alert">
        <span>{{ generationError || exerciseStore.error || materialStore.error }}</span>
        <button @click="dismissError" class="dismiss-btn">&times;</button>
      </div>

      <div v-if="configStore.config" class="config-summary">
        <h2>Configuration Complete</h2>
        <p>You're all set to start your programming journey!</p>

        <div class="summary-grid">
          <div class="summary-card">
            <h3>Preferred Language</h3>
            <p class="summary-value">{{ languageLabel(configStore.config.preferred_language) }}</p>
          </div>

          <div class="summary-card">
            <h3>Skill Level</h3>
            <p class="summary-value">{{ skillLevelLabel(configStore.config.skill_level) }}</p>
          </div>

          <div class="summary-card">
            <h3>AI Model</h3>
            <p class="summary-value">{{ configStore.config.ai_model }}</p>
          </div>
        </div>

        <div class="quota-summary">
          <h3>Daily Exercise Quotas</h3>
          <ul>
            <li v-for="quota in configStore.config.daily_quotas" :key="quota.language">
              <strong>{{ languageLabel(quota.language) }}:</strong> {{ quota.quota }} exercises/day
            </li>
          </ul>
        </div>

        <!-- Generate Exercises Section -->
        <div class="generate-section">
          <h3>Generate Exercises</h3>
          <p>Create coding exercises from source code materials using AI.</p>
          <div class="generate-actions">
            <button
              @click="handleGenerateExercises"
              :disabled="exerciseStore.generating"
              class="generate-btn"
            >
              <span v-if="exerciseStore.generating" class="spinner"></span>
              {{ exerciseStore.generating ? 'Generating...' : 'Generate Exercises' }}
            </button>
            <span v-if="exerciseStore.generating" class="generating-hint">
              This may take a moment as AI analyzes and generates exercises.
            </span>
          </div>
        </div>

        <!-- Materials Section -->
        <div class="materials-section">
          <h3>Materials</h3>
          <div v-if="materialStore.loading" class="loading-text">Loading materials...</div>
          <div v-else-if="materialCountByLanguage === 0" class="empty-state">
            No cached materials yet. Generate exercises to fetch materials automatically.
          </div>
          <div v-else class="material-count">
            <span class="count-number">{{ materialCountByLanguage }}</span>
            <span class="count-label">cached materials for {{ languageLabel(configStore.config.preferred_language) }}</span>
          </div>
        </div>

        <!-- Your Exercises Section -->
        <div class="exercises-section">
          <h3>Your Exercises</h3>
          <div v-if="exerciseStore.loading" class="loading-text">Loading exercises...</div>
          <div v-else-if="exerciseCountByLanguage === 0" class="empty-state">
            No exercises yet. Click "Generate Exercises" to create your first set of exercises.
          </div>
          <div v-else class="exercise-list">
            <div
              v-for="exercise in exerciseStore.exercises"
              :key="exercise.id"
              class="exercise-card"
            >
              <div class="exercise-card-header">
                <h4 class="exercise-title">{{ exercise.title }}</h4>
                <div class="exercise-badges">
                  <span :class="['badge', languageBadgeColor(exercise.language)]">
                    {{ languageLabel(exercise.language) }}
                  </span>
                  <span :class="['badge', difficultyColor(exercise.difficulty)]">
                    {{ skillLevelLabel(exercise.difficulty) }}
                  </span>
                </div>
              </div>
              <p class="exercise-description">{{ truncateDescription(exercise.description) }}</p>
              <div class="exercise-concept">
                <span class="concept-label">Concept:</span> {{ exercise.concept }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="loading">
        Loading your configuration...
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-page {
  min-height: 100vh;
  background-color: #f9fafb;
  padding: 2rem 1rem;
}

.dashboard-container {
  max-width: 1000px;
  margin: 0 auto;
}

.dashboard-header {
  text-align: center;
  margin-bottom: 3rem;
}

.dashboard-header h1 {
  font-size: 2.5rem;
  font-weight: 700;
  color: #1f2937;
  margin-bottom: 0.5rem;
}

.dashboard-header p {
  font-size: 1.25rem;
  color: #6b7280;
}

.config-summary {
  background-color: white;
  border-radius: 1rem;
  padding: 2rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.config-summary h2 {
  font-size: 1.75rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 0.5rem;
}

.config-summary > p {
  font-size: 1.1rem;
  color: #6b7280;
  margin-bottom: 2rem;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.summary-card {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  text-align: center;
}

.summary-card h3 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.5rem;
}

.summary-value {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.quota-summary {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  margin-bottom: 2rem;
}

.quota-summary h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 1rem;
}

.quota-summary ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.quota-summary li {
  padding: 0.5rem 0;
  color: #374151;
  font-size: 1rem;
}

.error-alert {
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 0.75rem;
  padding: 1rem 1.5rem;
  margin-bottom: 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: #991b1b;
  font-size: 0.95rem;
}

.dismiss-btn {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: #991b1b;
  cursor: pointer;
  padding: 0 0.25rem;
}

.generate-section {
  background-color: #f0fdf4;
  border-radius: 0.75rem;
  padding: 1.5rem;
  margin-bottom: 2rem;
  border-left: 4px solid #22c55e;
}

.generate-section h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #166534;
  margin-bottom: 0.5rem;
}

.generate-section p {
  color: #15803d;
  margin-bottom: 1rem;
}

.generate-actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.generate-btn {
  background-color: #22c55e;
  color: white;
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem 1.25rem;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.generate-btn:hover:not(:disabled) {
  background-color: #16a34a;
}

.generate-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.spinner {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  margin-right: 0.5rem;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.generating-hint {
  color: #15803d;
  font-size: 0.85rem;
  font-style: italic;
}

.materials-section {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  margin-bottom: 2rem;
}

.materials-section h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 0.75rem;
}

.material-count {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.count-number {
  font-size: 2rem;
  font-weight: 700;
  color: #3b82f6;
}

.count-label {
  color: #6b7280;
  font-size: 0.95rem;
}

.exercises-section {
  margin-top: 1.5rem;
}

.exercises-section h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 1rem;
}

.exercise-list {
  display: grid;
  gap: 1rem;
}

.exercise-card {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.25rem;
  border: 1px solid #e5e7eb;
  transition: border-color 0.2s;
}

.exercise-card:hover {
  border-color: #3b82f6;
}

.exercise-card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.75rem;
}

.exercise-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.exercise-badges {
  display: flex;
  gap: 0.5rem;
}

.badge {
  display: inline-block;
  padding: 0.2rem 0.6rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: capitalize;
}

.exercise-description {
  color: #6b7280;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
  line-height: 1.5;
}

.exercise-concept {
  font-size: 0.85rem;
  color: #374151;
}

.concept-label {
  font-weight: 600;
  color: #1f2937;
}

.empty-state {
  color: #9ca3af;
  text-align: center;
  padding: 2rem 0;
  font-size: 0.95rem;
}

.loading-text {
  color: #6b7280;
  text-align: center;
  padding: 1.5rem 0;
  font-size: 0.95rem;
}

.loading {
  text-align: center;
  padding: 3rem;
  color: #6b7280;
  font-size: 1.1rem;
}

@media (max-width: 768px) {
  .dashboard-header h1 {
    font-size: 1.75rem;
  }

  .dashboard-header p {
    font-size: 1rem;
  }

  .summary-grid {
    grid-template-columns: 1fr;
  }

  .exercise-card-header {
    flex-direction: column;
    gap: 0.5rem;
  }
}
</style>