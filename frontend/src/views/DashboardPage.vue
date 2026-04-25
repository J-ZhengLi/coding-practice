<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useConfigStore, type LanguageSkillLevel } from '../stores/config';
import { useExerciseStore } from '../stores/exercise';
import { useMaterialStore } from '../stores/material';
import { useProgressStore } from '../stores/progress';
import { useSubmissionStore } from '../stores/submission';
import { useScheduleStore } from '../stores/schedule';
import { useRouter } from 'vue-router';
import ProgressMetrics from '../components/ProgressMetrics.vue';
import ExerciseHistoryCard from '../components/ExerciseHistoryCard.vue';

const configStore = useConfigStore();
const exerciseStore = useExerciseStore();
const materialStore = useMaterialStore();
const progressStore = useProgressStore();
const submissionStore = useSubmissionStore();
const scheduleStore = useScheduleStore();
const router = useRouter();

const generationError = ref<string | null>(null);

/** Look up the skill level for a given language from the skill_levels array. */
const getSkillLevelForLanguage = (language: string): string | undefined => {
  return configStore.config?.skill_levels?.find((s: LanguageSkillLevel) => s.language === language)?.skill_level;
};

/** Get the skill level for the preferred language, falling back to the first skill level. */
const preferredDifficulty = computed(() => {
  if (!configStore.config) return undefined;
  return getSkillLevelForLanguage(configStore.config.preferred_language)
    || configStore.config.skill_levels?.[0]?.skill_level;
});

/** Combine languages with their skill levels for the "Languages to Learn" display. */
const languagesWithSkillLevels = computed(() => {
  if (!configStore.config) return [];
  return configStore.config.skill_levels.map((sl: LanguageSkillLevel) => ({
    language: sl.language,
    skill_level: sl.skill_level,
  }));
});

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
    const difficulty = preferredDifficulty.value;
    await Promise.all([
      exerciseStore.fetchExercises(language, difficulty),
      materialStore.fetchMaterials(language, difficulty),
    ]);

    // Load progress history after exercises are loaded
    await loadProgressHistory();

    // Per D-05: on-demand daily plan generation
    await scheduleStore.fetchDailyPlan(configStore.config.preferred_language);
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
  const language = configStore.config.preferred_language;
  const difficulty = preferredDifficulty.value;

  if (!difficulty) {
    generationError.value = 'No skill level found for preferred language. Please reconfigure.';
    return;
  }

  const result = await exerciseStore.generateExercises(language, difficulty);
  if (!result && exerciseStore.error) {
    generationError.value = exerciseStore.error;
  } else if (result && result.generated_count === 0) {
    generationError.value = 'No exercises could be generated. Check that materials are available and the AI provider is running.';
  } else if (result) {
    await scheduleStore.fetchDailyPlan(configStore.config.preferred_language);
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

const progressLoaded = ref(false);

const loadProgressHistory = async () => {
  if (progressLoaded.value) return;
  progressLoaded.value = true;
  // Load progress data
  await Promise.all([
    progressStore.fetchDailyProgress(),
    progressStore.fetchScoreTrend(7),
  ]);
  // Load submissions for each exercise in the current list
  for (const exercise of exerciseStore.exercises) {
    await submissionStore.getSubmissionsByExercise(exercise.id);
  }
};

const dailyAverage = computed(() => progressStore.dailyProgress?.avg_score ?? null);
const dailyLetterGrade = computed(() => progressStore.dailyProgress?.letter_grade ?? null);
const completionCount = computed(() => progressStore.dailyProgress?.completion_count ?? 0);
const scoreTrend = computed(() => progressStore.scoreTrend?.scores ?? []);

const exerciseHistory = computed(() => {
  return exerciseStore.exercises.map(exercise => {
    const exerciseSubmissions = submissionStore.submissions.filter(
      s => s.exercise_id === exercise.id
    );
    const bestSub = exerciseSubmissions.reduce((best, s) =>
      s.score > (best?.score ?? -1) ? s : best, null as any
    );
    return {
      id: exercise.id,
      title: exercise.title,
      language: exercise.language,
      difficulty: exercise.difficulty,
      bestScore: bestSub?.score ?? 0,
      bestLetterGrade: bestSub?.letter_grade ?? '-',
      attemptCount: exerciseSubmissions.length,
      submissions: exerciseSubmissions.map(s => ({
        id: s.id,
        score: s.score,
        letter_grade: s.letter_grade,
        submitted_at: s.submitted_at,
      })),
    };
  }).filter(e => e.attemptCount > 0);
});
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
          <div class="summary-card languages-card">
            <h3>Languages to Learn</h3>
            <ul class="language-skill-list">
              <li v-for="lang in languagesWithSkillLevels" :key="lang.language">
                <span class="language-name">{{ languageLabel(lang.language) }}</span>
                <span :class="['badge', difficultyColor(lang.skill_level)]">
                  {{ skillLevelLabel(lang.skill_level) }}
                </span>
              </li>
            </ul>
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

        <!-- Today's Plan per D-05, D-07, D-08 -->
        <div class="exercises-section">
          <h3>Today's Plan <span v-if="scheduleStore.dailyPlan" class="plan-summary-label">({{ scheduleStore.newCount }} new + {{ scheduleStore.reviewCount }} reviews)</span></h3>
          <div v-if="scheduleStore.loading" class="loading-text">Loading today's plan...</div>
          <div v-else-if="scheduleStore.error" class="error-state">{{ scheduleStore.error }}</div>
          <div v-else-if="scheduleStore.exercises.length === 0" class="empty-state">
            No exercises for today. All caught up! Check back later or generate new exercises.
          </div>
          <div v-else class="exercise-list">
            <div
              v-for="exercise in scheduleStore.exercises"
              :key="exercise.id"
              class="exercise-card"
            >
              <div class="exercise-card-header">
                <h4 class="exercise-title">{{ exercise.title }}</h4>
                <div class="exercise-badges">
                  <span v-if="exercise.is_review" class="badge bg-amber-100 text-amber-800">Review</span>
                  <span :class="['badge', languageBadgeColor(exercise.language)]">{{ languageLabel(exercise.language) }}</span>
                  <span :class="['badge', difficultyColor(exercise.difficulty)]">{{ skillLevelLabel(exercise.difficulty) }}</span>
                </div>
              </div>
              <p class="exercise-description">{{ truncateDescription(exercise.description) }}</p>
              <div class="exercise-concept">
                <span class="concept-label">Concept:</span> {{ exercise.concept }}
              </div>
              <div class="exercise-actions">
                <button class="start-btn" @click="router.push({ name: 'exercise-editor', params: { id: exercise.id } })">
                  Start
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Progress Section per D-09, D-11 -->
        <div class="progress-section">
          <h3>Progress</h3>

          <div v-if="progressStore.loading" class="loading-text">Loading progress...</div>

          <div v-else-if="!dailyAverage && exerciseHistory.length === 0" class="empty-state">
            No submissions yet. Complete an exercise to see your progress.
          </div>

          <div v-else>
            <!-- Metric cards per D-11 -->
            <ProgressMetrics
              :daily-average="dailyAverage"
              :letter-grade="dailyLetterGrade"
              :completion-count="completionCount"
              :score-trend="scoreTrend"
            />

            <!-- Exercise History per D-10 -->
            <div v-if="exerciseHistory.length > 0" class="history-section">
              <h4>Exercise History</h4>
              <div class="history-list">
                <ExerciseHistoryCard
                  v-for="entry in exerciseHistory"
                  :key="entry.id"
                  :exercise-title="entry.title"
                  :language="entry.language"
                  :difficulty="entry.difficulty"
                  :best-score="entry.bestScore"
                  :best-letter-grade="entry.bestLetterGrade"
                  :attempt-count="entry.attemptCount"
                  :submissions="entry.submissions"
                />
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
  grid-template-columns: 2fr 1fr;
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

.languages-card {
  text-align: left;
}

.language-skill-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.language-skill-list li {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.language-name {
  font-weight: 600;
  color: #1f2937;
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

.plan-summary-label {
  font-size: 0.75rem;
  font-weight: 400;
  color: #6b7280;
}

.error-state {
  color: #ef4444;
  text-align: center;
  padding: 1.5rem 0;
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

.exercise-actions {
  margin-top: 0.75rem;
}

.start-btn {
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.start-btn:hover {
  background-color: #2563eb;
}

.progress-section {
  margin-top: 2rem;
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
}

.progress-section h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 1rem;
}

.history-section {
  margin-top: 1.5rem;
}

.history-section h4 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 1rem;
}

.history-list {
  display: grid;
  gap: 0.75rem;
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