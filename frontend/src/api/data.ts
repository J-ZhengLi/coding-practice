import apiClient from './client';

/**
 * Export all application data as a downloadable JSON file.
 * Returns a Blob that can be used to trigger a browser download.
 */
export async function exportData(): Promise<Blob> {
  const response = await apiClient.get('/api/data/export', {
    responseType: 'blob',
  });
  return response.data;
}

/**
 * Import data from a JSON file, replacing all application data.
 * The file must be a valid export JSON with version, exported_at, and tables.
 * Import is atomic: either all data is replaced or none is.
 */
export async function importData(file: File): Promise<{ status: string }> {
  const text = await file.text();
  const json = JSON.parse(text);
  const response = await apiClient.post<{ status: string }>('/api/data/import', json);
  return response.data;
}