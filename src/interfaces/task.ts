export type TaskStatus = 'not-started' | 'ongoing' | 'paused' | 'completed';

export type ReminderFrequency = 'none' | 'hourly' | 'every-3-hours' | 'daily';

export interface Task {
  id: string;
  title: string;
  notes?: string;
  status: TaskStatus;
  createdAt: Date;
  updatedAt: Date;
  deadline?: Date;
  hasCalendarIntegration: boolean;
  calendarEmail?: string;
  reminderFrequency: ReminderFrequency;
  completedAt?: Date;
  startedAt?: Date;
  pausedAt?: Date;
}

export interface TaskFormData {
  title: string;
  notes?: string;
  hasDeadline: boolean;
  deadline?: Date;
  hasCalendarIntegration: boolean;
  calendarEmail?: string;
  reminderFrequency: ReminderFrequency;
}
