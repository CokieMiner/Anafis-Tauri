// useNotification.ts - Hook for notification system
import { useContext } from 'react';
import { NotificationContext, NotificationContextType } from '@/core/contexts/notificationContext';

export { type NotificationData } from '@/core/contexts/NotificationContext';

export const useNotification = (): NotificationContextType => {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error('useNotification must be used within a NotificationProvider');
  }
  return context;
};