<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import * as monaco from 'monaco-editor';
import { useExerciseStore } from '../stores/exercise';
import { useSubmissionStore } from '../stores/submission';
import { useConfigStore } from '../stores/config';
import ExerciseSidebar from '../components/ExerciseSidebar.vue';
import ErrorBanner from '../components/ErrorBanner.vue';
import { fetchExerciseById, type ExerciseResponse } from '../api/exercises';

const route = useRoute();
const router = useRouter();
const exerciseStore = useExerciseStore();
const submissionStore = useSubmissionStore();
const configStore = useConfigStore();

// Editor state
const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let decorationsCollection: monaco.editor.IEditorDecorationsCollection | null = null;

const currentExercise = ref<ExerciseResponse | null>(null);
const sidebarCollapsed = ref(false);
const showError = ref(true);

// Language mapping for Monaco
const languageMap: Record<string, string> = {
  python: 'python',
  rust: 'rust',
  go: 'go',
  cpp: 'cpp',
};

// Responsive: collapse sidebar on tablet
const isTablet = ref(false);
const checkTablet = () => {
  isTablet.value = window.innerWidth <= 768;
  if (isTablet.value) {
    sidebarCollapsed.value = true;
  }
};

// Editor height
const editorHeight = computed(() => isTablet.value ? '70vh' : '80vh');

// Find TODO lines and create decorations
const updateTodoDecorations = () => {
  if (!editor) return;
  const model = editor.getModel();
  if (!model) return;

  const lineCount = model.getLineCount();
  const decorations: monaco.editor.IModelDeltaDecoration[] = [];

  for (let lineNumber = 1; lineNumber <= lineCount; lineNumber++) {
    const lineContent = model.getLineContent(lineNumber);
    if (lineContent.includes('TODO:')) {
      decorations.push({
        range: new monaco.Range(lineNumber, 1, lineNumber, 1),
        options: {
          isWholeLine: true,
          glyphMarginClassName: 'todo-glyph-icon',
          glyphMarginHoverMessage: { value: 'TODO: Complete this section' },
          stickiness: monaco.editor.TrackedRangeStickiness.AlwaysGrowsWhenTypingAtEdges,
        },
      });
    }
  }

  if (decorationsCollection) {
    decorationsCollection.clear();
  }
  decorationsCollection = editor.createDecorationsCollection(decorations);
};

// Load exercise into editor
const loadExercise = async (exerciseId: number) => {
  try {
    const exercise = await fetchExerciseById(exerciseId);
    currentExercise.value = exercise;

    if (editor) {
      const lang = languageMap[exercise.language] || 'plaintext';
      const model = editor.getModel();
      if (model) {
        monaco.editor.setModelLanguage(model, lang);
        model.setValue(exercise.exercise_code);
      }
      updateTodoDecorations();
    }
  } catch (err: any) {
    console.error('Failed to load exercise:', err);
  }
};

// Initialize Monaco Editor
const initEditor = () => {
  if (!editorContainer.value) return;

  editor = monaco.editor.create(editorContainer.value, {
    value: currentExercise.value?.exercise_code || '// Select an exercise from the sidebar to begin',
    language: languageMap[currentExercise.value?.language || ''] || 'plaintext',
    theme: 'vs',
    fontSize: 14,
    lineNumbers: 'on',
    minimap: { enabled: false },
    wordWrap: 'on',
    autoIndent: 'advanced',
    tabSize: 4,
    scrollBeyondLastLine: false,
    glyphMargin: true,
    automaticLayout: true,
  });

  // Re-scan decorations on content change
  editor.onDidChangeModelContent(() => {
    updateTodoDecorations();
  });
};

// Handle exercise selection from sidebar
const handleExerciseSelect = (exerciseId: number) => {
  router.push({ name: 'exercise-editor', params: { id: exerciseId } });
};

// Submit code
const handleSubmit = async () => {
  if (!currentExercise.value || !editor || submissionStore.submitting) return;

  const code = editor.getValue();
  if (!code.trim()) return;

  showError.value = true;
  const result = await submissionStore.submitCode(currentExercise.value.id, code);

  if (result) {
    router.push({ name: 'results', params: { id: result.id } });
  }
};

// Retry submission
const handleRetry = () => {
  showError.value = true;
  handleSubmit();
};

// Dismiss error
const handleDismissError = () => {
  showError.value = false;
  submissionStore.error = null;
};

// Navigate to generate exercises (empty state CTA)
const goToDashboard = () => {
  router.push({ name: 'dashboard' });
};

// Toggle sidebar
const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value;
};

// Lifecycle
onMounted(async () => {
  checkTablet();
  window.addEventListener('resize', checkTablet);

  // Ensure configuration is loaded
  if (!configStore.config) {
    await configStore.fetchConfig();
  }

  // Load exercises for sidebar
  if (configStore.config) {
    const language = configStore.config.preferred_language;
    const difficulty = configStore.config.skill_level;
    if (exerciseStore.exercises.length === 0) {
      await exerciseStore.fetchExercises(language, difficulty);
    }
  }

  // Initialize editor
  await nextTick();
  initEditor();

  // Load the exercise from the route
  const exerciseId = Number(route.params.id);
  if (exerciseId && !isNaN(exerciseId)) {
    await loadExercise(exerciseId);
  }
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', checkTablet);
  if (editor) {
    editor.dispose();
    editor = null;
  }
});

// Watch route changes for exercise switching
watch(() => route.params.id, async (newId) => {
  const id = Number(newId);
  if (id && !isNaN(id)) {
    await loadExercise(id);
  }
});
</script>

<template>
  <div class="exercise-editor-page">
    <!-- Error banner for AI evaluation failures (D-12) -->
    <ErrorBanner
      v-if="submissionStore.error && showError"
      :message="submissionStore.error"
      @retry="handleRetry"
      @dismiss="handleDismissError"
    />

    <div class="editor-layout">
      <!-- Sidebar -->
      <ExerciseSidebar
        :selected-id="currentExercise?.id ?? null"
        :collapsed="sidebarCollapsed"
        @select="handleExerciseSelect"
      />

      <!-- Editor area -->
      <div class="editor-area">
        <!-- Empty state (D-13) -->
        <div v-if="exerciseStore.exercises.length === 0 && !exerciseStore.loading" class="empty-state">
          <h2>No exercises available</h2>
          <p>Generate exercises for your selected language and skill level to start practicing.</p>
          <button class="generate-cta-btn" @click="goToDashboard">Generate Exercises</button>
        </div>

        <!-- Editor with submit -->
        <template v-else>
          <!-- Sidebar toggle button -->
          <div class="editor-toolbar">
            <button class="toggle-sidebar-btn" @click="toggleSidebar" :title="sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'">
              <span v-if="sidebarCollapsed">&#9776;</span>
              <span v-else>&#9664;</span>
            </button>
            <span v-if="currentExercise" class="current-exercise-title">{{ currentExercise.title }}</span>
          </div>

          <!-- Monaco editor container -->
          <div
            ref="editorContainer"
            class="monaco-container"
            :style="{ height: editorHeight }"
          ></div>

          <!-- Submit button area (D-05) -->
          <div class="submit-area">
            <button
              class="submit-btn"
              :disabled="submissionStore.submitting || !currentExercise"
              :aria-disabled="submissionStore.submitting || !currentExercise"
              @click="handleSubmit"
            >
              <span v-if="submissionStore.submitting" class="spinner"></span>
              {{ submissionStore.submitting ? 'Evaluating your code...' : 'Submit Code' }}
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.exercise-editor-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f9fafb;
}

.editor-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background-color: white;
  border-bottom: 1px solid #e5e7eb;
}

.toggle-sidebar-btn {
  background: none;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  padding: 4px 8px;
  cursor: pointer;
  font-size: 0.85rem;
  color: #374151;
  transition: background-color 0.15s;
}
.toggle-sidebar-btn:hover {
  background-color: #f3f4f6;
}

.current-exercise-title {
  font-size: 0.95rem;
  font-weight: 600;
  color: #1f2937;
}

.monaco-container {
  flex: 1;
  min-height: 400px;
}

.submit-area {
  display: flex;
  justify-content: flex-end;
  padding: 12px 16px;
  background-color: white;
  border-top: 1px solid #e5e7eb;
}

.submit-btn {
  background-color: #22c55e;
  color: white;
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem 1.5rem;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.submit-btn:hover:not(:disabled) {
  background-color: #16a34a;
}
.submit-btn:disabled {
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
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  text-align: center;
  padding: 3rem 2rem;
}
.empty-state h2 {
  font-size: 1.5rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 0.5rem;
}
.empty-state p {
  color: #6b7280;
  font-size: 1rem;
  margin-bottom: 1.5rem;
  max-width: 400px;
}
.generate-cta-btn {
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
.generate-cta-btn:hover {
  background-color: #16a34a;
}

/* TODO gutter decoration styling */
:deep(.todo-glyph-icon) {
  background: #f59e0b;
  width: 16px !important;
  height: 16px !important;
  margin-left: 3px;
  border-radius: 3px;
}

@media (max-width: 768px) {
  .exercise-editor-page {
    height: auto;
    min-height: 100vh;
  }
  .editor-layout {
    flex-direction: row;
  }
}
</style>