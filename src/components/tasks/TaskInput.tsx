import { useState, KeyboardEvent } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Plus } from 'lucide-react';
import { motion } from 'framer-motion';

interface TaskInputProps {
  onAddTask: (title: string) => void;
  placeholder?: string;
}

export const TaskInput = ({ 
  onAddTask, 
  placeholder = 'Write a task…' 
}: TaskInputProps) => {
  const [value, setValue] = useState('');

  const handleSubmit = () => {
    const trimmed = value.trim();
    if (trimmed) {
      onAddTask(trimmed);
      setValue('');
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSubmit();
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex items-center gap-2 px-4 py-2"
    >
      <Input
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        className="flex-1 h-9 bg-background border-border/50 focus-visible:ring-primary/30"
      />
      <Button
        size="sm"
        onClick={handleSubmit}
        disabled={!value.trim()}
        className="h-9 px-3 gap-1.5"
      >
        <Plus className="h-4 w-4" />
        Add
      </Button>
    </motion.div>
  );
};
