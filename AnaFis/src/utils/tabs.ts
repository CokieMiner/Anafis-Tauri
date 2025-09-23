import React from 'react';
import SpreadsheetTab from '../pages/SpreadsheetTab';
import FittingTab from '../pages/FittingTab';
import SolverTab from '../pages/SolverTab';
import MonteCarloTab from '../pages/MonteCarloTab';
import HomeTab from '../pages/HomeTab';

export const getTabContent = (tabType: string, openNewTab: (id: string, title: string, content: React.ReactNode) => void): React.ReactNode => {
  switch (tabType) {
    case 'spreadsheet':
      return <SpreadsheetTab />;
    case 'fitting':
      return <FittingTab />;
    case 'solver':
      return <SolverTab />;
    case 'montecarlo':
      return <MonteCarloTab />;
    default:
      return <HomeTab openNewTab={openNewTab} />;
  }
};
