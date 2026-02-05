import { NavLink as RouterNavLink, useLocation } from 'react-router-dom';
import { LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

interface SidebarNavItemProps {
  to: string;
  icon: LucideIcon;
  label: string;
}

export const SidebarNavItem = ({ to, icon: Icon, label }: SidebarNavItemProps) => {
  const location = useLocation();
  const isActive = location.pathname === to;

  return (
    <RouterNavLink
      to={to}
      className={cn(
        'nav-item group',
        isActive && 'active'
      )}
    >
      <Icon 
        className={cn(
          'h-5 w-5 transition-colors',
          isActive ? 'text-sidebar-primary' : 'text-sidebar-foreground group-hover:text-sidebar-accent-foreground'
        )} 
      />
      <span className="text-sm">{label}</span>
    </RouterNavLink>
  );
};
