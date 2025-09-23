import React from 'react';

export interface Tab {
  id: string;
  title: string;
  content: React.ReactNode;
}

export interface TabFromDetachedPayload {
  id: string;
  title: string;
  content_type: string;
}
