import apiClient from './client';

export interface DailyProgressResponse {
  date: string;
  avg_score: number | null;
  letter_grade: string | null;
  completion_count: number;
}

export interface DailyScoreEntry {
  date: string;
  avg_score: number;
}

export interface ScoreTrendResponse {
  scores: DailyScoreEntry[];
}

export async function getDailyProgress(): Promise<DailyProgressResponse> {
  const response = await apiClient.get<DailyProgressResponse>('/api/progress/daily');
  return response.data;
}

export async function getScoreTrend(days: number = 7): Promise<ScoreTrendResponse> {
  const response = await apiClient.get<ScoreTrendResponse>('/api/progress/trend', {
    params: { days },
  });
  return response.data;
}