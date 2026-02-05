import { TaskStatus } from '@/interfaces/task';
import { Button } from '@/components/ui/button';
import { 
  Play, 
  Pause, 
  CheckCircle2, 
  Pencil, 
  Trash2,
  RotateCcw 
} from 'lucide-react';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

interface TaskActionsProps {
  status: TaskStatus;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onComplete: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export const TaskActions = ({
  status,
  onStart,
  onPause,
  onResume,
  onComplete,
  onEdit,
  onDelete,
}: TaskActionsProps) => {
  return (
    <div className="flex items-center gap-1">
      {/* Status-specific actions */}
      {status === 'not-started' && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-primary hover:text-primary hover:bg-primary-muted"
              onClick={onStart}
            >
              <Play className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Start task</TooltipContent>
        </Tooltip>
      )}

      {status === 'ongoing' && (
        <>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-status-paused hover:text-status-paused hover:bg-accent"
                onClick={onPause}
              >
                <Pause className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Pause task</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-status-completed hover:text-status-completed hover:bg-accent"
                onClick={onComplete}
              >
                <CheckCircle2 className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Complete task</TooltipContent>
          </Tooltip>
        </>
      )}

      {status === 'paused' && (
        <>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-primary hover:text-primary hover:bg-primary-muted"
                onClick={onResume}
              >
                <RotateCcw className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Resume task</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-status-completed hover:text-status-completed hover:bg-accent"
                onClick={onComplete}
              >
                <CheckCircle2 className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Complete task</TooltipContent>
          </Tooltip>
        </>
      )}

      {/* Common actions */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-muted-foreground hover:text-foreground"
            onClick={onEdit}
          >
            <Pencil className="h-4 w-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Edit task</TooltipContent>
      </Tooltip>

      {status === 'not-started' && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-muted-foreground hover:text-destructive"
              onClick={onDelete}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Delete task</TooltipContent>
        </Tooltip>
      )}
    </div>
  );
};
