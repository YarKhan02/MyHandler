export interface Settings {
  id: number;
  darkMode: boolean;
  notificationsEnabled: boolean;
  defaultReminderFrequency: 'none' | 'hourly' | 'every-3-hours' | 'daily';
  createdAt: Date;
  updatedAt: Date;
}

export interface SettingsUpdateData {
  darkMode?: boolean;
  notificationsEnabled?: boolean;
  defaultReminderFrequency?: 'none' | 'hourly' | 'every-3-hours' | 'daily';
}
