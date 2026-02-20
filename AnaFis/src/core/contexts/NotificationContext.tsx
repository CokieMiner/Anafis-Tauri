// NotificationContext.tsx - Simple notification system for user feedback

import { Alert, type AlertColor, Snackbar } from '@mui/material';
import type React from 'react';
import { type ReactNode, useCallback, useState } from 'react';
import { NotificationContext } from '@/core/contexts/notificationContext';

export interface NotificationData {
  type: AlertColor;
  message: string;
  duration?: number;
}

export interface NotificationContextType {
  showNotification: (notification: NotificationData) => void;
  hideNotification: () => void;
}

interface NotificationProviderProps {
  children: ReactNode;
}

export const NotificationProvider: React.FC<NotificationProviderProps> = ({
  children,
}) => {
  const [notification, setNotification] = useState<NotificationData | null>(
    null
  );
  const [open, setOpen] = useState(false);

  const showNotification = useCallback((notificationData: NotificationData) => {
    setNotification(notificationData);
    setOpen(true);
  }, []);

  const hideNotification = useCallback(() => {
    setOpen(false);
    setNotification(null);
  }, []);

  const contextValue: NotificationContextType = {
    showNotification,
    hideNotification,
  };

  return (
    <NotificationContext.Provider value={contextValue}>
      {children}
      {notification && (
        <Snackbar
          open={open}
          autoHideDuration={notification.duration ?? 6000}
          onClose={hideNotification}
          anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
        >
          <Alert
            onClose={hideNotification}
            severity={notification.type}
            variant="filled"
            sx={{ width: '100%' }}
          >
            {notification.message}
          </Alert>
        </Snackbar>
      )}
    </NotificationContext.Provider>
  );
};
