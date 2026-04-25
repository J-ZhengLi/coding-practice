import { describe, it, expect, vi, beforeEach } from 'vitest';
import apiClient from '../client';
import { getDailyPlan, getScheduleStatus } from '../schedule';

vi.mock('../client', () => ({
  default: {
    get: vi.fn(),
  },
}));

describe('schedule API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getDailyPlan', () => {
    it('calls GET /api/schedule/daily-plan without params when no language given', async () => {
      const mockResponse = {
        exercises: [],
        summary: { new_count: 0, review_count: 0 },
      };
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });
      const result = await getDailyPlan();
      expect(apiClient.get).toHaveBeenCalledWith('/api/schedule/daily-plan', { params: {} });
      expect(result).toEqual(mockResponse);
    });

    it('calls GET /api/schedule/daily-plan with language param', async () => {
      const mockResponse = {
        exercises: [{ id: 1, is_review: true, title: 'Test' }],
        summary: { new_count: 0, review_count: 1 },
      };
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });
      const result = await getDailyPlan('python');
      expect(apiClient.get).toHaveBeenCalledWith('/api/schedule/daily-plan', { params: { language: 'python' } });
      expect(result.summary.review_count).toBe(1);
    });
  });

  describe('getScheduleStatus', () => {
    it('calls GET /api/schedule/status', async () => {
      const mockResponse = { active_concepts: 3, completed_concepts: 1, overdue_reviews: 0 };
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });
      const result = await getScheduleStatus();
      expect(apiClient.get).toHaveBeenCalledWith('/api/schedule/status');
      expect(result.active_concepts).toBe(3);
    });
  });
});