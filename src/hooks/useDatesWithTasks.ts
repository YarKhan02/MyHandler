import { useState, useEffect, useCallback } from 'react';
import { tauriCommands } from '@/lib/tauri';

export const useDatesWithTasks = () => {
  const [dates, setDates] = useState<Date[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchDates = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await tauriCommands.getAllDatesWithTasks();
      setDates(result);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load dates';
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDates();
  }, [fetchDates]);

  return {
    dates,
    isLoading,
    error,
    refetch: fetchDates,
  };
};
