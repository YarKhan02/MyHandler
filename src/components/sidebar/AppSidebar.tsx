import { Calendar, Focus, LayoutGrid, Settings } from 'lucide-react';
import { SidebarNavItem } from './SidebarNavItem';

export const AppSidebar = () => {
  return (
    <aside className="fixed left-0 top-0 h-screen w-60 bg-sidebar border-r border-sidebar-border flex flex-col">
      {/* App Logo */}
      <div className="h-16 flex items-center px-5 border-b border-sidebar-border">
        <h1 className="text-xl font-semibold text-foreground tracking-tight">
          MyHandler
        </h1>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-3 space-y-1">
        <SidebarNavItem to="/" icon={LayoutGrid} label="Daily" />
        <SidebarNavItem to="/calendar" icon={Calendar} label="Calendar" />
        <SidebarNavItem to="/focus" icon={Focus} label="Focus" />
        <SidebarNavItem to="/settings" icon={Settings} label="Settings" />
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-sidebar-border">
        <p className="text-xs text-muted-foreground">
          Stay consistent, stay focused.
        </p>
      </div>
    </aside>
  );
};
