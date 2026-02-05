import { useState, useCallback, useEffect } from 'react';
import { MainLayout, PageHeader } from '@/components/layout';
import { DateSection } from '@/components/daily';
import { TaskEditModal } from '@/components/tasks';
import { useTasks, useDateSections, useTasksByDate } from '@/hooks';
import { Task } from '@/interfaces/task';
import { useToast } from '@/hooks/use-toast';
import { tauriCommands } from '@/lib/tauri';

const DailyPage = () => {
  const { toast } = useToast();
  const {
    addTask,
    updateTask,
    deleteTask,
    startTask,
    pauseTask,
    resumeTask,
    completeTask,
  } = useTasks();

  const { sections, refetch: refetchSections } = useDateSections();
  
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Task[] | null>(null);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [sectionTasks, setSectionTasks] = useState<Record<string, Task[]>>({});

  // Fetch tasks for all sections
  const fetchSectionTasks = useCallback(async () => {
    const tasksMap: Record<string, Task[]> = {};
    for (const section of sections) {
      const key = section.date.toISOString();
      tasksMap[key] = await tauriCommands.getTasksByDate(section.date);
    }
    setSectionTasks(tasksMap);
  }, [sections]);

  // Refetch when sections change
  useEffect(() => {
    if (sections.length > 0) {
      fetchSectionTasks();
    }
  }, [sections, fetchSectionTasks]);

  const refreshData = useCallback(async () => {
    await refetchSections();
    await fetchSectionTasks();
  }, [refetchSections, fetchSectionTasks]);

  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query);
    if (query.trim()) {
      const results = await tauriCommands.searchTasks(query);
      setSearchResults(results);
    } else {
      setSearchResults(null);
    }
  }, []);

  const handleAddTask = useCallback((date: Date) => {
    return async (title: string) => {
      try {
        await addTask(title, date);
        await refreshData();
        toast({
          title: 'Task added',
          description: 'Your task has been created.',
        });
      } catch {
        toast({
          title: 'Error',
          description: 'Failed to add task.',
          variant: 'destructive',
        });
      }
    };
  }, [addTask, refreshData, toast]);

  const handleStart = useCallback(async (id: string) => {
    try {
      await startTask(id);
      await refreshData();
    } catch {
      toast({
        title: 'Error',
        description: 'Failed to start task.',
        variant: 'destructive',
      });
    }
  }, [startTask, refreshData, toast]);

  const handlePause = useCallback(async (id: string) => {
    try {
      await pauseTask(id);
      await refreshData();
    } catch {
      toast({
        title: 'Error',
        description: 'Failed to pause task.',
        variant: 'destructive',
      });
    }
  }, [pauseTask, refreshData, toast]);

  const handleResume = useCallback(async (id: string) => {
    try {
      await resumeTask(id);
      await refreshData();
    } catch {
      toast({
        title: 'Error',
        description: 'Failed to resume task.',
        variant: 'destructive',
      });
    }
  }, [resumeTask, refreshData, toast]);

  const handleComplete = useCallback(async (id: string) => {
    try {
      await completeTask(id);
      await refreshData();
      toast({
        title: 'Task completed 🎉',
        description: 'Great job! Keep it up.',
      });
    } catch {
      toast({
        title: 'Error',
        description: 'Failed to complete task.',
        variant: 'destructive',
      });
    }
  }, [completeTask, refreshData, toast]);

  const handleEdit = useCallback(async (id: string) => {
    const task = await tauriCommands.getTaskById(id);
    if (task) {
      setEditingTask(task);
    }
  }, []);

  const handleSaveEdit = useCallback(async (id: string, data: any) => {
    try {
      await updateTask(id, data);
      await refreshData();
      toast({
        title: 'Task updated',
        description: 'Your changes have been saved.',
      });
    } catch {
      toast({
        title: 'Error',
        description: 'Failed to update task.',
        variant: 'destructive',
      });
    }
  }, [updateTask, refreshData, toast]);

  const handleDelete = useCallback(async (id: string) => {
    try {
      await deleteTask(id);
      await refreshData();
      toast({
        title: 'Task deleted',
        description: 'Task has been removed.',
      });
    } catch {
      toast({
        title: 'Error',
        description: 'Failed to delete task.',
        variant: 'destructive',
      });
    }
  }, [deleteTask, refreshData, toast]);

  const getTasksForSection = (date: Date): Task[] => {
    return sectionTasks[date.toISOString()] || [];
  };

  return (
    <MainLayout>
      <PageHeader
        title="Daily"
        showSearch
        onSearch={handleSearch}
      />

      <div className="p-6 max-w-3xl mx-auto custom-scrollbar">
        {/* Search Results */}
        {searchResults && (
          <div className="mb-8">
            <h2 className="text-sm font-medium text-muted-foreground mb-4 px-4">
              Search results for "{searchQuery}" ({searchResults.length})
            </h2>
            {searchResults.length > 0 ? (
              <div className="space-y-2">
                {searchResults.map((task) => (
                  <div
                    key={task.id}
                    className="soft-card px-4 py-3 cursor-pointer hover:bg-accent/50 transition-colors"
                    onClick={() => handleEdit(task.id)}
                  >
                    <p className="text-sm font-medium">{task.title}</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      {task.createdAt.toLocaleDateString()}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground px-4">
                No tasks found matching your search.
              </p>
            )}
          </div>
        )}

        {/* Date Sections */}
        {!searchResults && (
          <div className="space-y-2">
            {sections.map((section) => (
              <DateSection
                key={section.date.toISOString()}
                section={section}
                tasks={getTasksForSection(section.date)}
                onAddTask={handleAddTask(section.date)}
                onStart={handleStart}
                onPause={handlePause}
                onResume={handleResume}
                onComplete={handleComplete}
                onEdit={handleEdit}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>

      {/* Edit Modal */}
      <TaskEditModal
        task={editingTask}
        isOpen={!!editingTask}
        onClose={() => setEditingTask(null)}
        onSave={handleSaveEdit}
      />
    </MainLayout>
  );
};

export default DailyPage;
