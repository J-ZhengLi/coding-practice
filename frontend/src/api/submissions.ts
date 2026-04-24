import apiClient from './client';

// Per Pitfall 2: AI evaluation can take 10-30 seconds, increase timeout for submission
const SUBMISSION_TIMEOUT = 60000;

export interface SubmissionResponse {
  id: number;
  exercise_id: number;
  score: number;
  letter_grade: string;
  is_partial: boolean;
  strengths: string[];
  improvements: string[];
  summary: string;
  submitted_at: string;
}

export interface SubmitCodeRequest {
  exercise_id: number;
  code: string;
}

export interface SolutionResponse {
  submission_id: number;
  exercise_id: number;
  user_code: string;
  original_code: string;
  language: string;
  score: number;
  letter_grade: string;
}

export async function submitCode(request: SubmitCodeRequest): Promise<SubmissionResponse> {
  const response = await apiClient.post<SubmissionResponse>('/api/submissions', request, {
    timeout: SUBMISSION_TIMEOUT,
  });
  return response.data;
}

export async function getSubmission(id: number): Promise<SubmissionResponse> {
  const response = await apiClient.get<SubmissionResponse>(`/api/submissions/${id}`);
  return response.data;
}

export async function getSubmissionsByExercise(exerciseId: number): Promise<SubmissionResponse[]> {
  const response = await apiClient.get<SubmissionResponse[]>('/api/submissions', {
    params: { exercise_id: exerciseId },
  });
  return response.data;
}

export async function getSolution(submissionId: number): Promise<SolutionResponse> {
  const response = await apiClient.get<SolutionResponse>(`/api/submissions/${submissionId}/solution`);
  return response.data;
}