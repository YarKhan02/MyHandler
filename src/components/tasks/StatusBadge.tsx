import { TaskStatus } from '@/interfaces/task';
import { cn } from '@/lib/utils';

interface StatusBadgeProps {
  status: TaskStatus;
  className?: string;
}

const statusConfig: Record<TaskStatus, { label: string; className: string }> = {
  'not-started': {
    label: 'Not Started',
    className: 'status-not-started',
  },
  'ongoing': {
    label: 'Ongoing',
    className: 'status-ongoing',
  },
  'paused': {
    label: 'Paused',
    className: 'status-paused',
  },
  'completed': {
    label: 'Completed',
    className: 'bg-status-completed/15 text-status-completed',
  },
};

export const StatusBadge = ({ status, className }: StatusBadgeProps) => {
  const config = statusConfig[status];

  return (
    <span className={cn('status-badge', config.className, className)}>
      {config.label}
    </span>
  );
};
