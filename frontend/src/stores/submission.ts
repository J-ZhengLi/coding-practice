import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  submitCode as submitCodeApi,
  getSubmission as getSubmissionApi,
  getSubmissionsByExercise as getSubmissionsByExerciseApi,
  getSolution as getSolutionApi,
  type SubmissionResponse,
  type SubmitCodeRequest,
  type SolutionResponse,
} from '../api/submissions';

export const useSubmissionStore = defineStore('submission', () => {
  const currentSubmission = ref<SubmissionResponse | null>(null);
  const submissions = ref<SubmissionResponse[]>([]);
  const solution = ref<SolutionResponse | null>(null);
  const submitting = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const submitCode = async (exerciseId: number, code: string): Promise<SubmissionResponse | null> => {
    submitting.value = true;
    error.value = null;
    try {
      const request: SubmitCodeRequest = { exercise_id: exerciseId, code };
      currentSubmission.value = await submitCodeApi(request);
      submissions.value.unshift(currentSubmission.value);
      return currentSubmission.value;
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Evaluation failed';
      return null;
    } finally {
      submitting.value = false;
    }
  };

  const getSubmission = async (id: number) => {
    loading.value = true;
    error.value = null;
    try {
      currentSubmission.value = await getSubmissionApi(id);
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to load submission';
    } finally {
      loading.value = false;
    }
  };

  const getSubmissionsByExercise = async (exerciseId: number) => {
    loading.value = true;
    error.value = null;
    try {
      const newSubmissions = await getSubmissionsByExerciseApi(exerciseId);
      // Merge instead of replace to preserve submissions from other exercises
      submissions.value = [
        ...submissions.value.filter(s => s.exercise_id !== exerciseId),
        ...newSubmissions,
      ];
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to load submissions';
    } finally {
      loading.value = false;
    }
  };

  const getSolution = async (submissionId: number) => {
    loading.value = true;
    error.value = null;
    try {
      solution.value = await getSolutionApi(submissionId);
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to load solution';
    } finally {
      loading.value = false;
    }
  };

  return {
    currentSubmission, submissions, solution, submitting, loading, error,
    submitCode, getSubmission, getSubmissionsByExercise, getSolution,
  };
});