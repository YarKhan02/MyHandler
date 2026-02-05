import { useState, useEffect } from 'react';
import { Task, TaskFormData, ReminderFrequency } from '@/interfaces/task';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Calendar } from '@/components/ui/calendar';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { format } from 'date-fns';
import { CalendarIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

interface TaskEditModalProps {
  task: Task | null;
  isOpen: boolean;
  onClose: () => void;
  onSave: (id: string, data: Partial<TaskFormData>) => void;
}

const reminderOptions: { value: ReminderFrequency; label: string }[] = [
  { value: 'none', label: 'None' },
  { value: 'hourly', label: 'Every 1 hour' },
  { value: 'every-3-hours', label: 'Every 3 hours' },
  { value: 'daily', label: 'Daily' },
];

export const TaskEditModal = ({
  task,
  isOpen,
  onClose,
  onSave,
}: TaskEditModalProps) => {
  const [formData, setFormData] = useState<TaskFormData>({
    title: '',
    notes: '',
    hasDeadline: false,
    deadline: undefined,
    hasCalendarIntegration: false,
    calendarEmail: '',
    reminderFrequency: 'none',
  });

  useEffect(() => {
    if (task) {
      setFormData({
        title: task.title,
        notes: task.notes || '',
        hasDeadline: !!task.deadline,
        deadline: task.deadline,
        hasCalendarIntegration: task.hasCalendarIntegration,
        calendarEmail: task.calendarEmail || '',
        reminderFrequency: task.reminderFrequency,
      });
    }
  }, [task]);

  const handleSave = () => {
    if (!task || !formData.title.trim()) return;
    onSave(task.id, formData);
    onClose();
  };

  const updateField = <K extends keyof TaskFormData>(
    field: K,
    value: TaskFormData[K]
  ) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Edit Task</DialogTitle>
        </DialogHeader>

        <div className="space-y-5 py-4">
          {/* Title */}
          <div className="space-y-2">
            <Label htmlFor="title">Title *</Label>
            <Input
              id="title"
              value={formData.title}
              onChange={(e) => updateField('title', e.target.value)}
              placeholder="Task title"
            />
          </div>

          {/* Notes */}
          <div className="space-y-2">
            <Label htmlFor="notes">Notes (optional)</Label>
            <Textarea
              id="notes"
              value={formData.notes}
              onChange={(e) => updateField('notes', e.target.value)}
              placeholder="Add notes..."
              rows={3}
            />
          </div>

          {/* Deadline Toggle */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label htmlFor="deadline-toggle">Deadline</Label>
              <Switch
                id="deadline-toggle"
                checked={formData.hasDeadline}
                onCheckedChange={(checked) => {
                  updateField('hasDeadline', checked);
                  if (!checked) updateField('deadline', undefined);
                }}
              />
            </div>

            {formData.hasDeadline && (
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    variant="outline"
                    className={cn(
                      'w-full justify-start text-left font-normal',
                      !formData.deadline && 'text-muted-foreground'
                    )}
                  >
                    <CalendarIcon className="mr-2 h-4 w-4" />
                    {formData.deadline
                      ? format(formData.deadline, 'PPP p')
                      : 'Pick a date & time'}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-auto p-0" align="start">
                  <Calendar
                    mode="single"
                    selected={formData.deadline}
                    onSelect={(date) => updateField('deadline', date)}
                    initialFocus
                    className="p-3 pointer-events-auto"
                  />
                </PopoverContent>
              </Popover>
            )}
          </div>

          {/* Calendar Integration Toggle */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label htmlFor="calendar-toggle">Calendar Integration</Label>
              <Switch
                id="calendar-toggle"
                checked={formData.hasCalendarIntegration}
                onCheckedChange={(checked) =>
                  updateField('hasCalendarIntegration', checked)
                }
              />
            </div>

            {formData.hasCalendarIntegration && (
              <Input
                value={formData.calendarEmail}
                onChange={(e) => updateField('calendarEmail', e.target.value)}
                placeholder="Email account (future feature)"
                disabled
              />
            )}
          </div>

          {/* Reminder Frequency */}
          <div className="space-y-2">
            <Label>Reminder Frequency</Label>
            <Select
              value={formData.reminderFrequency}
              onValueChange={(value: ReminderFrequency) =>
                updateField('reminderFrequency', value)
              }
            >
              <SelectTrigger>
                <SelectValue placeholder="Select frequency" />
              </SelectTrigger>
              <SelectContent>
                {reminderOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={!formData.title.trim()}>
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
