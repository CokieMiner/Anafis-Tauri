// notificationContext.ts - Notification context creation
import { createContext } from 'react';
import type { NotificationData, NotificationContextType } from '@/core/contexts/NotificationContext';

export const NotificationContext = createContext<NotificationContextType | null>(null);

export type { NotificationData, NotificationContextType };