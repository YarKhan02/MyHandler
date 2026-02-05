import { useState, useEffect, useCallback } from 'react';
import { Task } from '@/interfaces/task';
import { tauriCommands } from '@/lib/tauri';

export const useOngoingTask = () => {
  const [task, setTask] = useState<Task | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchOngoingTask = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await tauriCommands.getOngoingTask();
      setTask(result);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load ongoing task';
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchOngoingTask();
  }, [fetchOngoingTask]);

  return {
    task,
    isLoading,
    error,
    refetch: fetchOngoingTask,
  };
};
