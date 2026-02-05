import { invoke } from '@tauri-apps/api/core';
import { Task, TaskFormData, TaskStatus } from '@/interfaces/task';
import { DateSection } from '@/interfaces/date-section';

// Helper to convert backend task dates to Date objects
const parseTask = (task: any): Task => ({
  ...task,
  createdAt: new Date(task.createdAt),
  updatedAt: new Date(task.updatedAt),
  deadline: task.deadline ? new Date(task.deadline) : undefined,
  completedAt: task.completedAt ? new Date(task.completedAt) : undefined,
  startedAt: task.startedAt ? new Date(task.startedAt) : undefined,
  pausedAt: task.pausedAt ? new Date(task.pausedAt) : undefined,
});

const parseDateSection = (section: any): DateSection => ({
  ...section,
  date: new Date(section.date),
});

// Task Commands
export const tauriCommands = {
  createTask: async (title: string, taskDate: Date): Promise<Task> => {
    const result = await invoke('create_task', { 
      title, 
      taskDate: taskDate.toISOString() 
    });
    return parseTask(result);
  },

  updateTask: async (taskId: string, data: Partial<TaskFormData>): Promise<Task> => {
    const payload = {
      ...data,
      deadline: data.deadline?.toISOString(),
    };
    const result = await invoke('update_task', { taskId, data: payload });
    return parseTask(result);
  },

  deleteTask: async (taskId: string): Promise<void> => {
    await invoke('delete_task', { taskId });
  },

  startTask: async (taskId: string): Promise<Task> => {
    const result = await invoke('start_task', { taskId });
    return parseTask(result);
  },

  pauseTask: async (taskId: string): Promise<Task> => {
    const result = await invoke('pause_task', { taskId });
    return parseTask(result);
  },

  resumeTask: async (taskId: string): Promise<Task> => {
    const result = await invoke('resume_task', { taskId });
    return parseTask(result);
  },

  completeTask: async (taskId: string): Promise<Task> => {
    const result = await invoke('complete_task', { taskId });
    return parseTask(result);
  },

  getTasksByDate: async (date: Date): Promise<Task[]> => {
    const result = await invoke<any[]>('get_tasks_by_date', { 
      date: date.toISOString() 
    });
    return result.map(parseTask);
  },

  getTaskById: async (taskId: string): Promise<Task | null> => {
    const result = await invoke('get_task_by_id', { taskId });
    return result ? parseTask(result) : null;
  },

  getOngoingTask: async (): Promise<Task | null> => {
    const result = await invoke('get_ongoing_task');
    return result ? parseTask(result) : null;
  },

  searchTasks: async (query: string): Promise<Task[]> => {
    const result = await invoke<any[]>('search_tasks', { query });
    return result.map(parseTask);
  },

  getDateSections: async (): Promise<DateSection[]> => {
    const result = await invoke<any[]>('get_date_sections');
    return result.map(parseDateSection);
  },

  getAllDatesWithTasks: async (): Promise<Date[]> => {
    const result = await invoke<string[]>('get_all_dates_with_tasks');
    return result.map((d) => new Date(d));
  },

  getCompletedTasks: async (): Promise<Task[]> => {
    const result = await invoke<any[]>('get_completed_tasks');
    return result.map(parseTask);
  },
};
