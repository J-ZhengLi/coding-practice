import apiClient from './client';

/**
 * Send a test reminder notification.
 * The server uses the current notification tier (Gmail > SMTP > Desktop)
 * and sends a test notification through the best available channel.
 */
export async function sendTestReminder(): Promise<{ tier: string; message: string }> {
  const response = await apiClient.post<{ tier: string; message: string }>('/api/reminders/test');
  return response.data;
}