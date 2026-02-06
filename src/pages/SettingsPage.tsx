import { MainLayout, PageHeader } from '@/components/layout';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Moon, Bell, Clock, Calendar, Download, Upload } from 'lucide-react';
import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { tauriCommands } from '@/lib/tauri';
import type { Settings } from '@/interfaces/settings';

const SettingsPage = () => {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [connecting, setConnecting] = useState(false);


  // Fetch settings on mount
  useEffect(() => {
    const fetchSettings = async () => {
      try {
        const result = await tauriCommands.getSettings();
        setSettings(result);
      } catch (error) {
        console.error('Failed to fetch settings:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchSettings();
  }, []);

  const updateSetting = async (data: Partial<Settings>) => {
    if (!settings) return;

    try {
      const updated = await tauriCommands.updateSettings({
        darkMode: data.darkMode,
        notificationsEnabled: data.notificationsEnabled,
        defaultReminderFrequency: data.defaultReminderFrequency,
      });
      setSettings(updated);
    } catch (error) {
      console.error('Failed to update settings:', error);
    }
  };

  const handleDarkModeChange = (enabled: boolean) => {
    updateSetting({ darkMode: enabled });
    if (enabled) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  };

  const handleNotificationsChange = (enabled: boolean) => {
    updateSetting({ notificationsEnabled: enabled });
  };

  const handleReminderChange = (frequency: 'none' | 'hourly' | 'every-3-hours' | 'daily') => {
    updateSetting({ defaultReminderFrequency: frequency });
  };

  const startCalendarConnection = async () => {
    console.log('startCalendarConnection called');
    setConnecting(true);
    try {
      console.log('Calling startCalendarAuth...');
      const credentials = await tauriCommands.startCalendarAuth();
      console.log('Calendar connected:', credentials);
      
      // Refresh settings to show connected status
      const updatedSettings = await tauriCommands.getSettings();
      setSettings(updatedSettings);
    } catch (error) {
      console.error('Failed to start calendar auth:', error);
    } finally {
      setConnecting(false);
    }
  };

  const disconnectCalendar = async () => {
    try {
      await tauriCommands.disconnectCalendar();
      const updatedSettings = await tauriCommands.getSettings();
      setSettings(updatedSettings);
    } catch (error) {
      console.error('Failed to disconnect calendar:', error);
    }
  };

  if (loading || !settings) {
    return (
      <MainLayout>
        <PageHeader title="Settings" />
        <div className="p-6 max-w-2xl mx-auto">
          <p className="text-muted-foreground">Loading settings...</p>
        </div>
      </MainLayout>
    );
  }

  return (
    <MainLayout>
      <PageHeader title="Settings" />

      <div className="p-6 max-w-2xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          className="space-y-8"
        >
          {/* Appearance */}
          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Moon className="h-5 w-5" />
              Appearance
            </h2>
            <div className="soft-card p-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="dark-mode" className="text-base">
                    Dark Mode
                  </Label>
                  <p className="text-sm text-muted-foreground mt-0.5">
                    Switch between light and dark theme
                  </p>
                </div>
                <Switch
                  id="dark-mode"
                  checked={settings.darkMode}
                  onCheckedChange={handleDarkModeChange}
                />
              </div>
            </div>
          </section>

          {/* Notifications */}
          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Notifications
            </h2>
            <div className="soft-card p-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="notifications" className="text-base">
                    Enable Notifications
                  </Label>
                  <p className="text-sm text-muted-foreground mt-0.5">
                    Get reminders for your tasks
                  </p>
                </div>
                <Switch
                  id="notifications"
                  checked={settings.notificationsEnabled}
                  onCheckedChange={handleNotificationsChange}
                />
              </div>
            </div>
          </section>

          {/* Default Reminder */}
          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Default Reminder
            </h2>
            <div className="soft-card p-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label className="text-base">Default Reminder Frequency</Label>
                  <p className="text-sm text-muted-foreground mt-0.5">
                    Applied to new tasks by default
                  </p>
                </div>
                <Select 
                  value={settings.defaultReminderFrequency} 
                  onValueChange={(value) => handleReminderChange(value as 'none' | 'hourly' | 'every-3-hours' | 'daily')}
                >
                  <SelectTrigger className="w-40">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">None</SelectItem>
                    <SelectItem value="hourly">Every hour</SelectItem>
                    <SelectItem value="every-3-hours">Every 3 hours</SelectItem>
                    <SelectItem value="daily">Daily</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </section>

          {/* Calendar Integration */}
          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Calendar className="h-5 w-5" />
              Google Calendar Integration
            </h2>
            <div className="soft-card p-4 space-y-4">
              {settings.calendarIntegrationEnabled ? (
                <>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label className="text-base">Connected</Label>
                      <p className="text-sm text-muted-foreground mt-0.5">
                        {settings.calendarEmail}
                      </p>
                    </div>
                    <Button 
                      variant="outline" 
                      size="sm"
                      onClick={disconnectCalendar}
                    >
                      Disconnect
                    </Button>
                  </div>
                </>
              ) : (
                <>
                  <div>
                    <Label className="text-base">Connect Google Calendar</Label>
                    <p className="text-sm text-muted-foreground mt-1">
                      Sync your tasks with Google Calendar events. A browser window will open for authentication.
                    </p>
                  </div>
                  <Button 
                    variant="default"
                    onClick={startCalendarConnection}
                    disabled={connecting}
                  >
                    {connecting ? (
                      <>
                        <div className="animate-spin h-4 w-4 mr-2 border-2 border-background border-t-transparent rounded-full" />
                        Connecting...
                      </>
                    ) : (
                      <>
                        <Calendar className="h-4 w-4 mr-2" />
                        Connect Calendar
                      </>
                    )}
                  </Button>
                </>
              )}
            </div>
          </section>

          {/* Data */}
          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Download className="h-5 w-5" />
              Data
            </h2>
            <div className="soft-card p-4 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label className="text-base">Export Data</Label>
                  <p className="text-sm text-muted-foreground mt-0.5">
                    Download all your tasks as JSON
                  </p>
                </div>
                <Button variant="outline" size="sm" disabled>
                  <Download className="h-4 w-4 mr-2" />
                  Export
                </Button>
              </div>
              
              <Separator />
              
              <div className="flex items-center justify-between">
                <div>
                  <Label className="text-base">Import Data</Label>
                  <p className="text-sm text-muted-foreground mt-0.5">
                    Restore tasks from a backup
                  </p>
                </div>
                <Button variant="outline" size="sm" disabled>
                  <Upload className="h-4 w-4 mr-2" />
                  Import
                </Button>
              </div>
            </div>
          </section>

          {/* Version */}
          <div className="text-center pt-8">
            <p className="text-sm text-muted-foreground">
              MyHandler v1.0.0
            </p>
          </div>
        </motion.div>
      </div>
    </MainLayout>
  );
};

export default SettingsPage;
