import { useState, useEffect, useCallback } from 'react';
import { MainLayout, PageHeader } from '@/components/layout';
import { Calendar } from '@/components/ui/calendar';
import { useDatesWithTasks } from '@/hooks';
import { Task } from '@/interfaces/task';
import { tauriCommands } from '@/lib/tauri';
import { format } from 'date-fns';
import { motion, AnimatePresence } from 'framer-motion';
import { CalendarDays } from 'lucide-react';

const CalendarPage = () => {
  const [selectedDate, setSelectedDate] = useState<Date>(new Date());
  const [selectedDateTasks, setSelectedDateTasks] = useState<Task[]>([]);
  const [isLoadingTasks, setIsLoadingTasks] = useState(false);
  
  const { dates: datesWithTasks } = useDatesWithTasks();

  // Fetch tasks for selected date from backend
  const fetchTasksForDate = useCallback(async (date: Date) => {
    setIsLoadingTasks(true);
    try {
      const tasks = await tauriCommands.getTasksByDate(date);
      setSelectedDateTasks(tasks);
    } catch (error) {
      console.error('Failed to fetch tasks:', error);
      setSelectedDateTasks([]);
    } finally {
      setIsLoadingTasks(false);
    }
  }, []);

  // Fetch tasks when selected date changes
  useEffect(() => {
    fetchTasksForDate(selectedDate);
  }, [selectedDate, fetchTasksForDate]);

  // Convert dates to strings for calendar highlighting
  const datesWithTasksStrings = datesWithTasks.map((d) => d.toDateString());

  const modifiers = {
    hasTask: (date: Date) => datesWithTasksStrings.includes(date.toDateString()),
  };

  const modifiersClassNames = {
    hasTask: 'bg-primary/10 text-primary font-medium',
  };

  return (
    <MainLayout>
      <PageHeader title="Calendar" />

      <div className="flex h-[calc(100vh-4rem)]">
        {/* Left Panel - Calendar */}
        <div className="w-80 border-r border-border p-6 bg-card/50">
          <Calendar
            mode="single"
            selected={selectedDate}
            onSelect={(date) => date && setSelectedDate(date)}
            modifiers={modifiers}
            modifiersClassNames={modifiersClassNames}
            className="pointer-events-auto"
          />
        </div>

        {/* Right Panel - Tasks for selected date */}
        <div className="flex-1 p-6 overflow-y-auto custom-scrollbar">
          <div className="max-w-2xl mx-auto">
            {/* Selected Date Header */}
            <div className="flex items-center gap-3 mb-6">
              <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                <CalendarDays className="h-5 w-5 text-primary" />
              </div>
              <div>
                <h2 className="text-lg font-semibold">
                  {format(selectedDate, 'EEEE')}
                </h2>
                <p className="text-sm text-muted-foreground">
                  {format(selectedDate, 'MMMM d, yyyy')}
                </p>
              </div>
            </div>

            {/* Tasks List */}
            <div className="space-y-2">
              <AnimatePresence mode="popLayout">
                {isLoadingTasks ? (
                  <div className="text-center py-12">
                    <p className="text-muted-foreground">Loading tasks...</p>
                  </div>
                ) : selectedDateTasks.length > 0 ? (
                  selectedDateTasks.map((task) => (
                    <motion.div
                      key={task.id}
                      initial={{ opacity: 0, y: 8 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -8 }}
                      className="soft-card px-4 py-3"
                    >
                      <p className="text-sm font-medium">{task.title}</p>
                      <p className="text-sm text-muted-foreground/70 mt-1">{task.notes}</p>
                      <div className="flex items-center gap-2 mt-1.5">
                        <span className={`status-badge ${
                          task.status === 'completed' 
                            ? 'bg-status-completed/15 text-status-completed'
                            : task.status === 'ongoing'
                            ? 'status-ongoing'
                            : task.status === 'paused'
                            ? 'status-paused'
                            : 'status-not-started'
                        }`}>
                          {task.status.replace('-', ' ')}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {format(task.createdAt, 'h:mm a')}
                        </span>
                      </div>
                    </motion.div>
                  ))
                ) : (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="text-center py-12"
                  >
                    <div className="h-16 w-16 rounded-full bg-muted/50 flex items-center justify-center mx-auto mb-4">
                      <CalendarDays className="h-8 w-8 text-muted-foreground" />
                    </div>
                    <p className="text-muted-foreground">
                      No tasks for this date
                    </p>
                    <p className="text-sm text-muted-foreground/70 mt-1">
                      Tasks created on this day will appear here
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>
        </div>
      </div>
    </MainLayout>
  );
};

export default CalendarPage;
