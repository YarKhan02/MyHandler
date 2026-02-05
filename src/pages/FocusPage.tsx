import { useCallback } from 'react';
import { MainLayout, PageHeader } from '@/components/layout';
import { useTasks, useOngoingTask, useTimer } from '@/hooks';
import { Button } from '@/components/ui/button';
import { Pause, CheckCircle2, Target } from 'lucide-react';
import { motion } from 'framer-motion';
import { useToast } from '@/hooks/use-toast';

const FocusPage = () => {
  const { toast } = useToast();
  const { pauseTask, completeTask } = useTasks();
  const { task: ongoingTask, isLoading, refetch } = useOngoingTask();
  
  const { formattedTime } = useTimer(
    !!ongoingTask,
    ongoingTask?.startedAt
  );

  const handlePause = useCallback(async () => {
    if (ongoingTask) {
      try {
        await pauseTask(ongoingTask.id);
        await refetch();
        toast({
          title: 'Task paused',
          description: 'You can resume anytime from the Daily view.',
        });
      } catch {
        toast({
          title: 'Error',
          description: 'Failed to pause task.',
          variant: 'destructive',
        });
      }
    }
  }, [ongoingTask, pauseTask, refetch, toast]);

  const handleComplete = useCallback(async () => {
    if (ongoingTask) {
      try {
        await completeTask(ongoingTask.id);
        await refetch();
        toast({
          title: 'Task completed 🎉',
          description: 'Great job! Keep the momentum going.',
        });
      } catch {
        toast({
          title: 'Error',
          description: 'Failed to complete task.',
          variant: 'destructive',
        });
      }
    }
  }, [ongoingTask, completeTask, refetch, toast]);

  if (isLoading) {
    return (
      <MainLayout>
        <PageHeader title="Focus" />
        <div className="flex items-center justify-center min-h-[calc(100vh-4rem)] p-6">
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </MainLayout>
    );
  }

  return (
    <MainLayout>
      <PageHeader title="Focus" />

      <div className="flex items-center justify-center min-h-[calc(100vh-4rem)] p-6">
        {ongoingTask ? (
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="text-center max-w-md mx-auto"
          >
            {/* Timer Card */}
            <div className="soft-card p-8 mb-8">
              <div className="h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-6">
                <Target className="h-8 w-8 text-primary animate-pulse" />
              </div>

              <h2 className="text-2xl font-semibold text-foreground mb-2">
                {ongoingTask.title}
              </h2>
              
              {ongoingTask.notes && (
                <p className="text-sm text-muted-foreground mb-6">
                  {ongoingTask.notes}
                </p>
              )}

              {/* Timer Display */}
              <div className="py-8">
                <p className="timer-display">{formattedTime}</p>
                <p className="text-sm text-muted-foreground mt-2">
                  Time elapsed
                </p>
              </div>

              {/* Actions */}
              <div className="flex items-center justify-center gap-3 mt-6">
                <Button
                  variant="outline"
                  size="lg"
                  onClick={handlePause}
                  className="gap-2"
                >
                  <Pause className="h-4 w-4" />
                  Pause
                </Button>
                <Button
                  size="lg"
                  onClick={handleComplete}
                  className="gap-2 bg-status-completed hover:bg-status-completed/90"
                >
                  <CheckCircle2 className="h-4 w-4" />
                  Complete
                </Button>
              </div>
            </div>

            <p className="text-sm text-muted-foreground">
              Stay focused. You're doing great.
            </p>
          </motion.div>
        ) : (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="text-center max-w-md mx-auto"
          >
            <div className="h-20 w-20 rounded-full bg-muted/50 flex items-center justify-center mx-auto mb-6">
              <Target className="h-10 w-10 text-muted-foreground" />
            </div>
            <h2 className="text-xl font-semibold text-foreground mb-2">
              No task in progress
            </h2>
            <p className="text-muted-foreground mb-6">
              Start a task from the Daily page to focus on it here.
            </p>
            <Button asChild variant="outline">
              <a href="/">Go to Daily</a>
            </Button>
          </motion.div>
        )}
      </div>
    </MainLayout>
  );
};

export default FocusPage;
