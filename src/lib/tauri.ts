import { invoke } from '@tauri-apps/api/core';
import { Task, TaskFormData, TaskStatus } from '@/interfaces/task';
import { DateSection } from '@/interfaces/date-section';
import { Settings, SettingsUpdateData, CalendarCredentials } from '@/interfaces/settings';

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
  // Create a new task
  createTask: async (title: string, taskDate: Date): Promise<Task> => {
    const result = await invoke('create_task', { 
      payload: {
        title, 
        createdAt: taskDate.toISOString()
      }
    });
    return parseTask(result);
  },

  // Update an existing task
  updateTask: async (id: string, data: Partial<TaskFormData>): Promise<Task> => {
    const result = await invoke('update_task', { 
      payload: {
        id,
        data: {
          ...data,
          deadline: data.deadline?.toISOString(),
        }
      }
    });
    return parseTask(result);
  },

  // Delete a task by ID
  deleteTask: async (id: string): Promise<void> => {
    await invoke('delete_task', { 
      payload: { id }
    });
  },

  // Task status updates to handle start, pause, resume, complete
  startTask: async (id: string): Promise<Task> => {
    const result = await invoke('start_task', { 
      payload: { id }
    });
    return parseTask(result);
  },

  pauseTask: async (id: string): Promise<Task> => {
    const result = await invoke('pause_task', { 
      payload: { id }
    });
    return parseTask(result);
  },

  resumeTask: async (id: string): Promise<Task> => {
    const result = await invoke('resume_task', { 
      payload: { id }
    });
    return parseTask(result);
  },

  completeTask: async (id: string): Promise<Task> => {
    const result = await invoke('complete_task', { 
      payload: { id }
    });
    return parseTask(result);
  },

  // Get tasks by date (excluding completed)
  getTasksByDateNotCompleted: async (date: Date): Promise<Task[]> => {
    const result = await invoke<any[]>('get_tasks_by_date_not_completed', { 
      payload: { date: date.toISOString() }
    });
    return result.map(parseTask);
  },

  // Get tasks by date
  getTasksByDate: async (date: Date): Promise<Task[]> => {
    const result = await invoke<any[]>('get_tasks_by_date', { 
      payload: { date: date.toISOString() }
    });
    return result.map(parseTask);
  },

  // Get a single task by ID
  getTaskById: async (id: string): Promise<Task | null> => {
    const result = await invoke('get_task_by_id', { 
      payload: { id }
    });
    return result ? parseTask(result) : null;
  },

  getOngoingTask: async (): Promise<Task | null> => {
    const result = await invoke('get_ongoing_task');
    return result ? parseTask(result) : null;
  },

  searchTasks: async (query: string): Promise<Task[]> => {
    const result = await invoke<any[]>('search_tasks', { 
      payload: { query }
    });
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

  // Settings Commands
  getSettings: async (): Promise<Settings> => {
    const result = await invoke<any>('get_settings');
    return {
      ...result,
      createdAt: new Date(result.createdAt),
      updatedAt: new Date(result.updatedAt),
    };
  },

  updateSettings: async (data: SettingsUpdateData): Promise<Settings> => {
    const result = await invoke<any>('update_settings', { 
      payload: data
    });
    return {
      ...result,
      createdAt: new Date(result.createdAt),
      updatedAt: new Date(result.updatedAt),
    };
  },

  // Calendar Commands
  startCalendarAuth: async (): Promise<CalendarCredentials> => {
    const result = await invoke<CalendarCredentials>('start_calendar_auth');
    return {
      ...result,
      tokenExpiry: new Date(result.tokenExpiry),
    };
  },

  getCalendarStatus: async (): Promise<CalendarCredentials | null> => {
    const result = await invoke<CalendarCredentials | null>('get_calendar_status');
    if (result) {
      return {
        ...result,
        tokenExpiry: new Date(result.tokenExpiry),
      };
    }
    return null;
  },

  disconnectCalendar: async (): Promise<void> => {
    await invoke('disconnect_calendar');
  },
};
