import { ReactNode } from 'react';
import { AppSidebar } from '@/components/sidebar';

interface MainLayoutProps {
  children: ReactNode;
}

export const MainLayout = ({ children }: MainLayoutProps) => {
  return (
    <div className="min-h-screen bg-background">
      <AppSidebar />
      <main className="ml-60 min-h-screen">
        {children}
      </main>
    </div>
  );
};
