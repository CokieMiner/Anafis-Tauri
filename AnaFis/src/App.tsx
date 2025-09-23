import React, { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { MainWindow } from './components/MainWindow';
import { DetachedTabWindow } from './components/DetachedTabWindow';

function App() {
  const [isDetached, setIsDetached] = useState(false);

  useEffect(() => {
    const checkDetached = async () => {
      try {
        const currentWindow = getCurrentWindow();
        if (currentWindow.label !== 'main') {
          setIsDetached(true);
        }
      } catch (error) {
        console.error("Failed to check if window is detached", error);
      }
    };
    checkDetached();
  }, []);

  if (isDetached) {
    return <DetachedTabWindow />;
  }

  return <MainWindow />;
}

export default App;