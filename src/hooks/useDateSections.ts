import { useState, useMemo } from 'react';
import { DateSection } from '@/interfaces/date-section';

export const useDateSections = () => {
  const sections = useMemo(() => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    
    const twoDaysAgo = new Date(today);
    twoDaysAgo.setDate(twoDaysAgo.getDate() - 2);
    
    return [
      { date: today, label: 'Today', isToday: true, isYesterday: false },
      { date: yesterday, label: 'Yesterday', isToday: false, isYesterday: true },
      { 
        date: twoDaysAgo, 
        label: twoDaysAgo.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' }), 
        isToday: false, 
        isYesterday: false 
      },
    ];
  }, []);

  return {
    sections,
    isLoading: false,
    error: null,
    refetch: () => {},
  };
};
