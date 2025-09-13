import React from 'react';
import { Box, Typography, Paper, List, ListItemButton, ListItemIcon, ListItemText, Card, CardContent, Avatar } from '@mui/material'; // Updated import
import TableChartIcon from '@mui/icons-material/TableChart'; // Spreadsheet
import ShowChartIcon from '@mui/icons-material/ShowChart'; // Fitting
import FunctionsIcon from '@mui/icons-material/Functions'; // Solver
import CasinoIcon from '@mui/icons-material/Casino'; // Monte Carlo (die icon)
import DescriptionIcon from '@mui/icons-material/Description'; // Generic file icon

// Import actual tab components
import SpreadsheetTab from './SpreadsheetTab';
import FittingTab from './FittingTab';
import SolverTab from './SolverTab';
import MonteCarloTab from './MonteCarloTab';

interface HomeTabProps {
  openNewTab: (id: string, title: string, content: React.ReactNode) => void;
}

const HomeTab: React.FC<HomeTabProps> = ({ openNewTab }) => {
  const handleNewTabClick = (tabType: string, title: string, content: React.ReactNode) => {
    openNewTab(`${tabType}-${Date.now()}`, title, content);
  };

  const quickActions = [
    {
      id: 'spreadsheet',
      title: 'New Spreadsheet',
      description: 'Data analysis and manipulation',
      icon: <TableChartIcon sx={{ fontSize: 28 }} />,
      color: '#2196f3', // Light blue
      content: <SpreadsheetTab />,
      emoji: '📊'
    },
    {
      id: 'fitting',
      title: 'New Fitting',
      description: 'Curve fitting and regression',
      icon: <ShowChartIcon sx={{ fontSize: 28 }} />,
      color: '#ff9800', // Orange
      content: <FittingTab />,
      emoji: '📈'
    },
    {
      id: 'solver',
      title: 'New Solver',
      description: 'Mathematical equation solving',
      icon: <FunctionsIcon sx={{ fontSize: 28 }} />,
      color: '#4caf50', // Green
      content: <SolverTab />,
      emoji: '🧮'
    },
    {
      id: 'montecarlo',
      title: 'New Monte Carlo',
      description: 'Statistical simulations',
      icon: <CasinoIcon sx={{ fontSize: 28 }} />,
      color: '#e91e63', // Pink
      content: <MonteCarloTab />,
      emoji: '🎲'
    }
  ];  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        bgcolor: 'background.default',
        color: 'text.primary',
        p: 4,
        gap: 4,
        boxSizing: 'border-box',
        overflow: 'auto',
        background: 'radial-gradient(circle at 20% 50%, rgba(30, 27, 75, 0.05) 0%, transparent 50%), radial-gradient(circle at 80% 20%, rgba(127, 29, 29, 0.05) 0%, transparent 50%), radial-gradient(circle at 40% 80%, rgba(88, 28, 135, 0.05) 0%, transparent 50%)',
        width: '100%', // Ensure full width
      }}
    >
      {/* Hero Section */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 4, mb: 6 }}>
        <Box
          sx={{
            position: 'relative',
            width: 140,
            height: 140,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: '20px',
            overflow: 'hidden',
            background: 'linear-gradient(135deg, rgba(30, 27, 75, 0.2) 0%, rgba(127, 29, 29, 0.2) 100%)',
            boxShadow: '0 8px 32px rgba(30, 27, 75, 0.2)',
            '&::before': {
              content: '""',
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              background: 'linear-gradient(45deg, transparent 30%, rgba(255, 255, 255, 0.1) 50%, transparent 70%)',
              animation: 'shine 3s ease-in-out infinite',
            },
            '@keyframes shine': {
              '0%': { transform: 'translateX(-100%)' },
              '100%': { transform: 'translateX(100%)' },
            },
          }}
        >
          <img
            src="/icon.png"
            alt="AnaFis Logo"
            style={{
              width: '110%',
              height: '110%',
              objectFit: 'contain',
              filter: 'drop-shadow(0 0 10px rgba(30, 27, 75, 0.3))'
            }}
          />
        </Box>
        <Box>
          <Typography
            variant="h1"
            component="h1"
            sx={{
              fontWeight: 'bold',
              color: 'text.primary',
              background: 'linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)',
              backgroundClip: 'text',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              textShadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
              mb: 1
            }}
          >
            AnaFis
            <Typography
              component="span"
              variant="h4"
              sx={{
                fontWeight: 300,
                color: 'primary.main',
                ml: 2,
                opacity: 1,
                fontSize: '0.6em',
                verticalAlign: 'super'
              }}
            >
              v0.1.0
            </Typography>
          </Typography>
          <Typography variant="h6" color="text.secondary" sx={{ mb: 2 }}>
            Advanced Numerical Analysis and Fitting Interface System
          </Typography>
        </Box>
      </Box>

      {/* Quick Actions Grid */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h4" sx={{ mb: 3, fontWeight: 600, color: 'text.primary' }}>
          Quick Start
        </Typography>
        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 3, justifyContent: 'flex-start' }}>
          {quickActions.map((action) => (
            <Box key={action.id} sx={{ maxWidth: '280px', flex: '0 0 auto' }}>
              <Card
                sx={{
                  height: 200, // Fixed height for squared appearance
                  width: 200, // Fixed width for squared appearance
                  cursor: 'pointer',
                  transition: 'all 0.3s ease-in-out',
                  background: 'linear-gradient(135deg, rgba(26, 26, 26, 0.8) 0%, rgba(42, 42, 42, 0.4) 100%)',
                  backdropFilter: 'blur(10px)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  '&:hover': {
                    transform: 'translateY(-8px)',
                    boxShadow: `0 20px 40px ${action.color}30`,
                    border: '1px solid',
                    borderColor: action.color,
                  },
                }}
                onClick={() => handleNewTabClick(action.id, action.title.replace('New ', ''), action.content)}
              >
                <CardContent sx={{ p: 3, textAlign: 'center' }}>
                  <Avatar
                    sx={{
                      width: 60,
                      height: 60,
                      mx: 'auto',
                      mb: 2,
                      bgcolor: `${action.color}15`,
                      color: action.color,
                      border: '2px solid',
                      borderColor: action.color,
                      opacity: 1,
                      transition: 'all 0.3s ease-in-out',
                      boxShadow: `0 4px 12px ${action.color}30`,
                      '&:hover': {
                        opacity: 1,
                        transform: 'scale(1.1)',
                        boxShadow: `0 8px 20px ${action.color}50`,
                      },
                    }}
                  >
                    {action.icon}
                  </Avatar>
                  <Typography variant="h6" sx={{ mb: 1, fontWeight: 600 }}>
                    {action.emoji} {action.title.replace('New ', '')}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    {action.description}
                  </Typography>
                </CardContent>
              </Card>
            </Box>
          ))}
        </Box>
      </Box>

      {/* Recent Files Section */}
      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
        <Paper
          elevation={0}
          sx={{
            p: 3,
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            height: '100%',
            flexGrow: 1
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
            <DescriptionIcon sx={{ color: '#ffffff' }} />
            <Typography variant="h5" component="h2" sx={{ fontWeight: 600 }}>
              Recent Files
            </Typography>
          </Box>
          <Box sx={{
            flexGrow: 1,
            border: 1,
            borderColor: 'divider',
            borderRadius: '12px',
            overflow: 'hidden',
            background: 'rgba(255, 255, 255, 0.02)',
            minHeight: '400px' // Ensure it expands
          }}>
              <List dense sx={{ p: 0 }}>
                {[
// Spreadsheet files - light blue
                  { name: 'pendulum_data.csv', type: 'Data File', icon: <TableChartIcon />, color: '#64b5f6' },
                  { name: 'transistor_curves.xlsx', type: 'Spreadsheet', icon: <TableChartIcon />, color: '#64b5f6' },
// Project files - purple
                  { name: 'g_measurement.anafis', type: 'Project', icon: <DescriptionIcon />, color: '#ba68c8' },
// Fitting files - orange
                  { name: 'circuit_analysis.fit', type: 'Fitting Result', icon: <ShowChartIcon />, color: '#ffb74d' },
                ].map((file, index) => (
                  <ListItemButton
                    key={index}
                    sx={{
                      borderBottom: index < 3 ? '1px solid rgba(255, 255, 255, 0.1)' : 'none',
                      '&:hover': {
                        bgcolor: 'rgba(0, 212, 255, 0.1)',
                      },
                    }}
                  >
                    <ListItemIcon>
                      <Avatar sx={{ width: 32, height: 32, bgcolor: `${file.color}20`, color: file.color }}>
                        {file.icon}
                      </Avatar>
                    </ListItemIcon>
                    <ListItemText
                      primary={file.name}
                      secondary={file.type}
                      slotProps={{
                        primary: { sx: { color: 'text.primary', fontWeight: 500 } },
                        secondary: { sx: { color: 'text.secondary' } }
                      }}
                    />
                  </ListItemButton>
                ))}
              </List>
            </Box>
          </Paper>
      </Box>
    </Box>
  );
};

export default HomeTab;
