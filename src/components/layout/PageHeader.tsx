import { useState } from 'react';
import { Search, Plus } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

interface PageHeaderProps {
  title: string;
  showSearch?: boolean;
  showQuickAdd?: boolean;
  onSearch?: (query: string) => void;
  onQuickAdd?: () => void;
}

export const PageHeader = ({
  title,
  showSearch = false,
  showQuickAdd = false,
  onSearch,
  onQuickAdd,
}: PageHeaderProps) => {
  const [searchQuery, setSearchQuery] = useState('');

  const handleSearchChange = (value: string) => {
    setSearchQuery(value);
    onSearch?.(value);
  };

  return (
    <header className="h-16 border-b border-border bg-background/80 backdrop-blur-sm sticky top-0 z-10">
      <div className="h-full px-6 flex items-center justify-between gap-4">
        {/* Page Title */}
        <h1 className="text-xl font-semibold text-foreground">{title}</h1>

        {/* Actions */}
        <div className="flex items-center gap-3">
          {showSearch && (
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                value={searchQuery}
                onChange={(e) => handleSearchChange(e.target.value)}
                placeholder="Search tasks..."
                className="w-64 pl-9 h-9 bg-secondary/50 border-border/50"
              />
            </div>
          )}

          {showQuickAdd && (
            <Button size="sm" onClick={onQuickAdd} className="gap-1.5">
              <Plus className="h-4 w-4" />
              Quick Add
            </Button>
          )}
        </div>
      </div>
    </header>
  );
};
