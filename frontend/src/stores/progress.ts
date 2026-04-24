import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  getDailyProgress as getDailyProgressApi,
  getScoreTrend as getScoreTrendApi,
  type DailyProgressResponse,
  type ScoreTrendResponse,
} from '../api/progress';

export const useProgressStore = defineStore('progress', () => {
  const dailyProgress = ref<DailyProgressResponse | null>(null);
  const scoreTrend = ref<ScoreTrendResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const fetchDailyProgress = async () => {
    loading.value = true;
    error.value = null;
    try {
      dailyProgress.value = await getDailyProgressApi();
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to load progress';
    } finally {
      loading.value = false;
    }
  };

  const fetchScoreTrend = async (days: number = 7) => {
    loading.value = true;
    error.value = null;
    try {
      scoreTrend.value = await getScoreTrendApi(days);
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to load trend';
    } finally {
      loading.value = false;
    }
  };

  return { dailyProgress, scoreTrend, loading, error, fetchDailyProgress, fetchScoreTrend };
});