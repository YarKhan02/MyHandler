export interface Settings {
  id: number;
  darkMode: boolean;
  notificationsEnabled: boolean;
  defaultReminderFrequency: 'none' | 'hourly' | 'every-3-hours' | 'daily';
  calendarIntegrationEnabled: boolean;
  calendarEmail: string | null;
  createdAt: Date;
  updatedAt: Date;
}

export interface SettingsUpdateData {
  darkMode?: boolean;
  notificationsEnabled?: boolean;
  defaultReminderFrequency?: 'none' | 'hourly' | 'every-3-hours' | 'daily';
}

export interface CalendarCredentials {
  email: string;
  accessToken: string;
  refreshToken: string;
  tokenExpiry: Date;
}
