import apiClient from './client';

export interface Material {
  id: number;
  source_url: string;
  source_type: string;
  language: string;
  title: string;
  difficulty: string;
  local_path: string;
  fetched_at: string;
}

/**
 * Fetch cached materials with optional language/difficulty filtering.
 */
export async function fetchMaterials(language?: string, difficulty?: string): Promise<Material[]> {
  const params: Record<string, string> = {};
  if (language) params.language = language;
  if (difficulty) params.difficulty = difficulty;

  const response = await apiClient.get<Material[]>('/api/materials', { params });
  return response.data;
}

/**
 * Trigger material fetching for a language and difficulty.
 */
export async function triggerFetchMaterials(language: string, difficulty: string): Promise<Material[]> {
  const response = await apiClient.post<Material[]>('/api/materials/fetch', {
    language,
    difficulty,
  });
  return response.data;
}

/**
 * Refresh a specific cached material by re-fetching from source.
 */
export async function refreshMaterial(id: number): Promise<Material> {
  const response = await apiClient.post<Material>(`/api/materials/${id}/refresh`);
  return response.data;
}

/**
 * Delete a cached material.
 */
export async function deleteMaterial(id: number): Promise<void> {
  await apiClient.delete(`/api/materials/${id}`);
}