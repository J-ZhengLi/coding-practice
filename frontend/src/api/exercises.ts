import apiClient from './client';

export interface ExerciseResponse {
  id: number;
  title: string;
  description: string;
  language: string;
  difficulty: string;
  todo_comment: string;
  exercise_code: string;
  concept: string;
  source: string;
}

export interface GenerateRequest {
  language: string;
  difficulty: string;
  count?: number;
}

export interface GenerateResponse {
  exercises: ExerciseResponse[];
  generated_count: number;
  cached: boolean;
}

/**
 * Fetch exercises with optional language/difficulty filtering.
 */
export async function fetchExercises(language?: string, difficulty?: string): Promise<ExerciseResponse[]> {
  const params: Record<string, string> = {};
  if (language) params.language = language;
  if (difficulty) params.difficulty = difficulty;

  const response = await apiClient.get<ExerciseResponse[]>('/api/exercises', { params });
  return response.data;
}

/**
 * Trigger exercise generation for a language and difficulty.
 */
export async function generateExercises(request: GenerateRequest): Promise<GenerateResponse> {
  const response = await apiClient.post<GenerateResponse>('/api/exercises/generate', request);
  return response.data;
}

/**
 * Fetch a single exercise by ID.
 */
export async function fetchExerciseById(id: number): Promise<ExerciseResponse> {
  const response = await apiClient.get<ExerciseResponse>(`/api/exercises/${id}`);
  return response.data;
}

/**
 * Delete an exercise by ID.
 */
export async function deleteExercise(id: number): Promise<void> {
  await apiClient.delete(`/api/exercises/${id}`);
}