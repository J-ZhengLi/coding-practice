<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  exerciseTitle: string;
  language: string;
  difficulty: string;
  bestScore: number;
  bestLetterGrade: string;
  attemptCount: number;
  submissions: {
    id: number;
    score: number;
    letter_grade: string;
    submitted_at: string;
  }[];
}>();

const expanded = ref(false);

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

const formatDate = (isoDate: string): string => {
  const date = new Date(isoDate);
  const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const month = months[date.getMonth()];
  const day = date.getDate();
  const year = date.getFullYear();
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  return `${month} ${day}, ${year} ${hours}:${minutes}`;
};

const gradeBadgeClass = (grade: string): string => {
  const map: Record<string, string> = {
    A: 'grade-mini-a',
    B: 'grade-mini-b',
    C: 'grade-mini-c',
    D: 'grade-mini-d',
    F: 'grade-mini-f',
  };
  return map[grade] || '';
};
</script>

<template>
  <div :class="['history-card', { expanded }]" @click="expanded = !expanded">
    <div class="history-summary">
      <div class="history-header">
        <h4 class="history-title">{{ exerciseTitle }}</h4>
        <div class="history-badges">
          <span :class="['badge', languageBadgeColor(language)]">{{ languageLabel(language) }}</span>
          <span :class="['badge', difficultyColor(difficulty)]">{{ difficulty }}</span>
        </div>
      </div>
      <div class="history-stats">
        <span class="best-score">Best: {{ bestScore }} ({{ bestLetterGrade }})</span>
        <span class="attempt-count">{{ attemptCount }} attempt{{ attemptCount !== 1 ? 's' : '' }}</span>
      </div>
    </div>
    <div v-if="expanded" class="history-detail" @click.stop>
      <h5>Submission History</h5>
      <table class="history-table">
        <thead><tr><th>Date</th><th>Score</th><th>Grade</th></tr></thead>
        <tbody>
          <tr v-for="sub in submissions" :key="sub.id">
            <td>{{ formatDate(sub.submitted_at) }}</td>
            <td>{{ sub.score }}</td>
            <td><span :class="['grade-mini-badge', gradeBadgeClass(sub.letter_grade)]">{{ sub.letter_grade }}</span></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.history-card {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.25rem;
  border: 1px solid #e5e7eb;
  cursor: pointer;
  transition: border-color 0.2s;
}

.history-card:hover {
  border-color: #3b82f6;
}

.history-card.expanded {
  border-color: #3b82f6;
}

.history-summary {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.history-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.history-badges {
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

.history-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.875rem;
  color: #6b7280;
}

.best-score {
  font-weight: 600;
  color: #374151;
}

.attempt-count {
  color: #9ca3af;
}

.history-detail {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #e5e7eb;
}

.history-detail h5 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #374151;
  margin-bottom: 0.75rem;
}

.history-table {
  width: 100%;
  border-collapse: collapse;
}

.history-table th {
  text-align: left;
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid #e5e7eb;
}

.history-table td {
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  color: #374151;
  border-bottom: 1px solid #f3f4f6;
}

.grade-mini-badge {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 9999px;
  font-size: 0.7rem;
  font-weight: 600;
}

.grade-mini-a {
  background-color: #f0fdf4;
  color: #166534;
}

.grade-mini-b {
  background-color: #f7fee7;
  color: #3f6212;
}

.grade-mini-c {
  background-color: #fefce8;
  color: #854d0e;
}

.grade-mini-d {
  background-color: #fff7ed;
  color: #9a3412;
}

.grade-mini-f {
  background-color: #fef2f2;
  color: #991b1b;
}
</style>