import { Task } from '@/interfaces/task';
import { StatusBadge } from './StatusBadge';
import { TaskActions } from './TaskActions';
import { format } from 'date-fns';
import { Calendar } from 'lucide-react';
import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';

interface TaskRowProps {
  task: Task;
  onStart: (id: string) => void;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onComplete: (id: string) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  isReadOnly?: boolean;
}

export const TaskRow = ({
  task,
  onStart,
  onPause,
  onResume,
  onComplete,
  onEdit,
  onDelete,
  isReadOnly = false,
}: TaskRowProps) => {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.2 }}
      className={cn(
        'task-row group px-4 py-3 rounded-lg flex items-start justify-between gap-4',
        task.status === 'ongoing' && 'bg-primary-muted/50 border border-primary/20'
      )}
    >
      {/* Left Side - Task Info */}
      <div className="flex-1 min-w-0">
        <h3 className="text-sm font-medium text-foreground truncate">
          {task.title}
        </h3>
        
        <div className="flex items-center gap-2 mt-1.5 flex-wrap">
          <StatusBadge status={task.status} />
          
          {task.deadline && (
            <span className="text-xs text-muted-foreground flex items-center gap-1">
              Due: {format(task.deadline, 'MMM d, h:mm a')}
            </span>
          )}
          
          {task.hasCalendarIntegration && (
            <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
          )}
        </div>
      </div>

      {/* Right Side - Actions */}
      {!isReadOnly && (
        <div className="opacity-0 group-hover:opacity-100 transition-opacity flex items-center">
          <TaskActions
            status={task.status}
            onStart={() => onStart(task.id)}
            onPause={() => onPause(task.id)}
            onResume={() => onResume(task.id)}
            onComplete={() => onComplete(task.id)}
            onEdit={() => onEdit(task.id)}
            onDelete={() => onDelete(task.id)}
          />
        </div>
      )}
    </motion.div>
  );
};
