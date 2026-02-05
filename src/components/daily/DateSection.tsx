import { Task } from '@/interfaces/task';
import { DateSection as DateSectionType } from '@/interfaces/date-section';
import { TaskRow } from '@/components/tasks/TaskRow';
import { TaskInput } from '@/components/tasks/TaskInput';
import { motion, AnimatePresence } from 'framer-motion';

interface DateSectionProps {
  section: DateSectionType;
  tasks: Task[];
  onAddTask: (title: string) => void;
  onStart: (id: string) => void;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onComplete: (id: string) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
}

export const DateSection = ({
  section,
  tasks,
  onAddTask,
  onStart,
  onPause,
  onResume,
  onComplete,
  onEdit,
  onDelete,
}: DateSectionProps) => {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className="date-section mb-8"
    >
      {/* Date Header */}
      <div className="flex items-center gap-3 mb-3 px-4">
        <div className="h-2.5 w-2.5 rounded-full bg-primary" />
        <h2 className="text-base font-semibold text-foreground">
          {section.label}
        </h2>
      </div>

      {/* Task Input */}
      <div className="ml-4 border-l-2 border-border pl-4">
        <TaskInput onAddTask={onAddTask} />

        {/* Task List */}
        <div className="mt-2 space-y-1">
          <AnimatePresence mode="popLayout">
            {tasks.map((task) => (
              <TaskRow
                key={task.id}
                task={task}
                onStart={onStart}
                onPause={onPause}
                onResume={onResume}
                onComplete={onComplete}
                onEdit={onEdit}
                onDelete={onDelete}
              />
            ))}
          </AnimatePresence>

          {tasks.length === 0 && (
            <p className="text-sm text-muted-foreground px-4 py-3">
              No tasks yet. Add one above!
            </p>
          )}
        </div>
      </div>
    </motion.div>
  );
};
