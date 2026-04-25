import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import {
  getDailyPlan as getDailyPlanApi,
  getScheduleStatus as getScheduleStatusApi,
  type DailyPlanResponse,
  type DailyPlanExercise,
  type ScheduleStatusResponse,
} from '../api/schedule';

export const useScheduleStore = defineStore('schedule', () => {
  const dailyPlan = ref<DailyPlanResponse | null>(null);
  const scheduleStatus = ref<ScheduleStatusResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const exercises = computed<DailyPlanExercise[]>(() => {
    if (!dailyPlan.value) return [];
    // Per D-07: randomly interleave reviews and new exercises
    // API returns them grouped; frontend shuffles
    const reviewExercises = dailyPlan.value.exercises.filter(e => e.is_review);
    const newExercises = dailyPlan.value.exercises.filter(e => !e.is_review);
    const mixed: DailyPlanExercise[] = [];
    let ri = 0, ni = 0;
    // Interleave: alternate between review and new, randomizing start
    const startWithReview = Math.random() > 0.5;
    let takeReview = startWithReview;
    while (ri < reviewExercises.length || ni < newExercises.length) {
      if (takeReview && ri < reviewExercises.length) {
        mixed.push(reviewExercises[ri++]);
      } else if (!takeReview && ni < newExercises.length) {
        mixed.push(newExercises[ni++]);
      } else if (ri < reviewExercises.length) {
        mixed.push(reviewExercises[ri++]);
      } else if (ni < newExercises.length) {
        mixed.push(newExercises[ni++]);
      }
      takeReview = !takeReview;
    }
    return mixed;
  });

  const newCount = computed(() => dailyPlan.value?.summary.new_count ?? 0);
  const reviewCount = computed(() => dailyPlan.value?.summary.review_count ?? 0);

  /** Check if a given exercise ID is a review exercise from the daily plan.
   *  Used by ExerciseSidebar to show Review badges without requiring parent wiring.
   */
  const isReviewExercise = (exerciseId: number): boolean => {
    if (!dailyPlan.value) return false;
    return dailyPlan.value.exercises.some(e => e.id === exerciseId && e.is_review);
  };

  const fetchDailyPlan = async (language?: string) => {
    loading.value = true;
    error.value = null;
    try {
      dailyPlan.value = await getDailyPlanApi(language);
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to load daily plan';
    } finally {
      loading.value = false;
    }
  };

  const fetchScheduleStatus = async () => {
    try {
      scheduleStatus.value = await getScheduleStatusApi();
    } catch (err: any) {
      // Non-critical -- don't set main error
      console.warn('Failed to load schedule status:', err);
    }
  };

  return {
    dailyPlan, scheduleStatus, exercises,
    newCount, reviewCount,
    loading, error,
    isReviewExercise,
    fetchDailyPlan, fetchScheduleStatus,
  };
});