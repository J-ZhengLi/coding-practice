import apiClient from './client';

export interface DailyPlanExercise {
  id: number;
  title: string;
  description: string;
  language: string;
  difficulty: string;
  concept: string;
  is_review: boolean;
  review_interval?: number;
  todo_comment: string;
  exercise_code: string;
}

export interface DailyPlanSummary {
  new_count: number;
  review_count: number;
}

export interface DailyPlanResponse {
  exercises: DailyPlanExercise[];
  summary: DailyPlanSummary;
}

export interface ScheduleStatusResponse {
  active_concepts: number;
  completed_concepts: number;
  overdue_reviews: number;
}

export async function getDailyPlan(language?: string): Promise<DailyPlanResponse> {
  const params: Record<string, string> = {};
  if (language) params.language = language;
  const response = await apiClient.get<DailyPlanResponse>('/api/schedule/daily-plan', { params });
  return response.data;
}

export async function getScheduleStatus(): Promise<ScheduleStatusResponse> {
  const response = await apiClient.get<ScheduleStatusResponse>('/api/schedule/status');
  return response.data;
}