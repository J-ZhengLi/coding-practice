<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useSubmissionStore } from '../stores/submission';
import GradeDisplay from '../components/GradeDisplay.vue';
import FeedbackSection from '../components/FeedbackSection.vue';
import SolutionComparison from '../components/SolutionComparison.vue';
import ErrorBanner from '../components/ErrorBanner.vue';

const route = useRoute();
const router = useRouter();
const submissionStore = useSubmissionStore();

const showSolution = ref(false);
const submissionId = computed(() => Number(route.params.id));

onMounted(async () => {
  await submissionStore.getSubmission(submissionId.value);
});

const submission = computed(() => submissionStore.currentSubmission);
const loading = computed(() => submissionStore.loading);
const error = computed(() => submissionStore.error);
const solution = computed(() => submissionStore.solution);

const handleViewSolution = async () => {
  showSolution.value = true;
  await submissionStore.getSolution(submissionId.value);
};

const handleRetryExercise = () => {
  if (submission.value) {
    router.push({ name: 'exercise-editor', params: { id: submission.value.exercise_id } });
  }
};

const handleRetry = () => {
  submissionStore.error = null;
  submissionStore.getSubmission(submissionId.value);
};

const dismissError = () => {
  submissionStore.error = null;
};
</script>

<template>
  <div class="results-page">
    <div class="results-container">
      <!-- Error state -->
      <ErrorBanner v-if="error" :message="error" @retry="handleRetry" @dismiss="dismissError" />

      <!-- Loading state -->
      <div v-if="loading && !submission" class="loading-state">Loading results...</div>

      <!-- Loaded state -->
      <div v-else-if="submission" class="results-content">
        <!-- Grade display per D-06: large letter grade prominently above numeric score -->
        <GradeDisplay
          :letter-grade="submission.letter_grade"
          :score="submission.score"
          :is-partial="submission.is_partial"
        />

        <!-- Feedback sections per D-06: strengths (green) + improvements (amber) -->
        <FeedbackSection
          :strengths="submission.strengths"
          :improvements="submission.improvements"
        />

        <!-- Summary -->
        <div class="summary-section">
          <h3>Summary</h3>
          <p>{{ submission.summary }}</p>
        </div>

        <!-- Action buttons per D-08: View Solution, D-05: Retry Exercise -->
        <div class="actions">
          <button class="view-solution-btn" @click="handleViewSolution">
            View Solution
          </button>
          <button class="retry-btn" @click="handleRetryExercise">
            Retry Exercise
          </button>
        </div>

        <!-- Solution comparison per D-08: side-by-side diff view -->
        <div v-if="showSolution && solution" class="solution-section">
          <h3>Solution Comparison</h3>
          <SolutionComparison
            :user-code="solution.user_code"
            :original-code="solution.original_code"
            :language="solution.language"
          />
        </div>
        <div v-if="showSolution && submissionStore.loading" class="solution-loading">
          Loading solution...
        </div>
      </div>

      <!-- Not found state -->
      <div v-else class="not-found">
        Submission not found.
      </div>
    </div>
  </div>
</template>

<style scoped>
.results-page {
  min-height: 100vh;
  background-color: #f9fafb;
  padding: 2rem 1rem;
}

.results-container {
  max-width: 800px;
  margin: 0 auto;
}

.results-content {
  background-color: white;
  border-radius: 1rem;
  padding: 2rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.summary-section {
  margin-top: 1.5rem;
  padding: 1rem;
  background-color: #f9fafb;
  border-radius: 0.75rem;
}

.summary-section h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 0.5rem 0;
}

.summary-section p {
  color: #374151;
  font-size: 0.95rem;
  line-height: 1.6;
  margin: 0;
}

.actions {
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
}

.view-solution-btn {
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem 1.25rem;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.view-solution-btn:hover {
  background-color: #2563eb;
}

.retry-btn {
  background-color: white;
  color: #374151;
  border: 1px solid #d1d5db;
  border-radius: 0.5rem;
  padding: 0.625rem 1.25rem;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.retry-btn:hover {
  background-color: #f9fafb;
}

.solution-section {
  margin-top: 2rem;
  border-top: 1px solid #e5e7eb;
  padding-top: 1.5rem;
}

.solution-section h3 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 1rem;
}

.loading-state {
  text-align: center;
  padding: 3rem;
  color: #6b7280;
}

.solution-loading {
  text-align: center;
  padding: 1rem;
  color: #6b7280;
  font-size: 0.9rem;
}

.not-found {
  text-align: center;
  padding: 3rem;
  color: #9ca3af;
}

@media (max-width: 768px) {
  .results-page {
    padding: 1rem 0.5rem;
  }

  .actions {
    flex-direction: column;
  }

  .results-content {
    padding: 1.5rem;
  }
}
</style>