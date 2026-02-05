import { useState, useCallback } from 'react';
import { Task, TaskFormData } from '@/interfaces/task';
import { tauriCommands } from '@/lib/tauri';

export const useTasks = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleError = (err: unknown) => {
    const message = err instanceof Error ? err.message : 'An error occurred';
    setError(message);
    throw err;
  };

  const addTask = useCallback(async (title: string, date: Date = new Date()): Promise<Task> => {
    setIsLoading(true);
    setError(null);
    try {
      const task = await tauriCommands.createTask(title.trim(), date);
      return task;
    } catch (err) {
      return handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const updateTask = useCallback(async (id: string, data: Partial<TaskFormData>): Promise<Task> => {
    setIsLoading(true);
    setError(null);
    try {
      const task = await tauriCommands.updateTask(id, data);
      return task;
    } catch (err) {
      return handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const deleteTask = useCallback(async (id: string): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      await tauriCommands.deleteTask(id);
    } catch (err) {
      handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const startTask = useCallback(async (id: string): Promise<Task> => {
    setIsLoading(true);
    setError(null);
    try {
      const task = await tauriCommands.startTask(id);
      return task;
    } catch (err) {
      return handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const pauseTask = useCallback(async (id: string): Promise<Task> => {
    setIsLoading(true);
    setError(null);
    try {
      const task = await tauriCommands.pauseTask(id);
      return task;
    } catch (err) {
      return handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const resumeTask = useCallback(async (id: string): Promise<Task> => {
    setIsLoading(true);
    setError(null);
    try {
      const task = await tauriCommands.resumeTask(id);
      return task;
    } catch (err) {
      return handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const completeTask = useCallback(async (id: string): Promise<Task> => {
    setIsLoading(true);
    setError(null);
    try {
      const task = await tauriCommands.completeTask(id);
      return task;
    } catch (err) {
      return handleError(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const getTasksByDate = useCallback(async (date: Date): Promise<Task[]> => {
    setError(null);
    try {
      return await tauriCommands.getTasksByDate(date);
    } catch (err) {
      handleError(err);
      return [];
    }
  }, []);

  const getTaskById = useCallback(async (id: string): Promise<Task | null> => {
    setError(null);
    try {
      return await tauriCommands.getTaskById(id);
    } catch (err) {
      handleError(err);
      return null;
    }
  }, []);

  const getOngoingTask = useCallback(async (): Promise<Task | null> => {
    setError(null);
    try {
      return await tauriCommands.getOngoingTask();
    } catch (err) {
      handleError(err);
      return null;
    }
  }, []);

  const searchTasks = useCallback(async (query: string): Promise<Task[]> => {
    setError(null);
    try {
      return await tauriCommands.searchTasks(query);
    } catch (err) {
      handleError(err);
      return [];
    }
  }, []);

  const getCompletedTasks = useCallback(async (): Promise<Task[]> => {
    setError(null);
    try {
      return await tauriCommands.getCompletedTasks();
    } catch (err) {
      handleError(err);
      return [];
    }
  }, []);

  return {
    isLoading,
    error,
    addTask,
    updateTask,
    deleteTask,
    startTask,
    pauseTask,
    resumeTask,
    completeTask,
    getTasksByDate,
    getTaskById,
    getOngoingTask,
    searchTasks,
    getCompletedTasks,
  };
};
