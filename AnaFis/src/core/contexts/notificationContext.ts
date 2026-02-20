// notificationContext.ts - Notification context creation
import { createContext } from 'react';
import type {
  NotificationContextType,
  NotificationData,
} from '@/core/contexts/NotificationContext';

export const NotificationContext =
  createContext<NotificationContextType | null>(null);

export type { NotificationData, NotificationContextType };
