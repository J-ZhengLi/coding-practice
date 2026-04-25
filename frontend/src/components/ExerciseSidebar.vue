<script setup lang="ts">
import { computed } from 'vue';
import { useExerciseStore } from '../stores/exercise';
import { useScheduleStore } from '../stores/schedule';

const props = defineProps<{
  selectedId: number | null;
  collapsed: boolean;
}>();

const emit = defineEmits<{
  select: [exerciseId: number];
}>();

const exerciseStore = useExerciseStore();
const scheduleStore = useScheduleStore();

const languageLabel = (lang: string): string => {
  const labels: Record<string, string> = { python: 'Python', rust: 'Rust', go: 'Go', cpp: 'C++' };
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
</script>

<template>
  <aside :class="['exercise-sidebar', { collapsed }]" :aria-label="collapsed ? 'Exercise list (collapsed)' : 'Exercise list'">
    <div v-if="!collapsed" class="sidebar-header">
      <h3>Exercises</h3>
    </div>
    <div v-if="exerciseStore.loading" class="sidebar-loading">Loading...</div>
    <div v-else-if="exerciseStore.exercises.length === 0" class="sidebar-empty">No exercises</div>
    <ul v-else class="exercise-list">
      <li
        v-for="exercise in exerciseStore.exercises"
        :key="exercise.id"
        :class="['exercise-item', { active: exercise.id === selectedId }]"
        :aria-label="`${exercise.title} - ${languageLabel(exercise.language)} - ${exercise.difficulty}`"
        @click="emit('select', exercise.id)"
      >
        <template v-if="!collapsed">
          <span class="exercise-item-title">{{ exercise.title }}</span>
          <div class="exercise-item-badges">
            <span v-if="scheduleStore.isReviewExercise(exercise.id)" class="badge bg-amber-100 text-amber-800">Review</span>
            <span v-if="exercise.source === 'ai_generated'" class="badge bg-purple-100 text-purple-800">AI Generated</span>
            <span :class="['badge', languageBadgeColor(exercise.language)]">{{ languageLabel(exercise.language) }}</span>
            <span :class="['badge', difficultyColor(exercise.difficulty)]">{{ exercise.difficulty }}</span>
          </div>
        </template>
        <template v-else>
          <span class="exercise-item-icon" :title="exercise.title">{{ exercise.title.charAt(0) }}</span>
        </template>
      </li>
    </ul>
  </aside>
</template>

<style scoped>
.exercise-sidebar {
  width: 280px;
  background-color: white;
  border-right: 1px solid #e5e7eb;
  overflow-y: auto;
  transition: width 0.2s ease;
  display: flex;
  flex-direction: column;
}
.exercise-sidebar.collapsed {
  width: 48px;
}
.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid #e5e7eb;
}
.sidebar-header h3 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}
.sidebar-loading, .sidebar-empty {
  padding: 24px 16px;
  text-align: center;
  color: #9ca3af;
  font-size: 0.875rem;
}
.exercise-list {
  list-style: none;
  padding: 0;
  margin: 0;
}
.exercise-item {
  padding: 12px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background-color 0.15s, border-color 0.15s;
  min-height: 48px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.exercise-item:hover {
  background-color: #f9fafb;
}
.exercise-item.active {
  background-color: #eff6ff;
  border-left-color: #3b82f6;
}
.exercise-item-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: #1f2937;
}
.exercise-item-badges {
  display: flex;
  gap: 4px;
}
.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: capitalize;
}
.exercise-item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: #f3f4f6;
  font-size: 0.75rem;
  font-weight: 600;
  color: #374151;
  margin: 0 auto;
}
@media (max-width: 768px) {
  .exercise-sidebar { width: 48px; }
  .sidebar-header, .sidebar-loading, .sidebar-empty,
  .exercise-item-title, .exercise-item-badges { display: none; }
  .exercise-item-icon { display: flex; }
}
</style>