import React, { useCallback, useMemo } from 'react';
import { Box, Typography, Paper, List, ListItemButton, ListItemIcon, ListItemText, Card, CardContent, Avatar } from '@mui/material';
import TableChartIcon from '@mui/icons-material/TableChart';
import ShowChartIcon from '@mui/icons-material/ShowChart';
import FunctionsIcon from '@mui/icons-material/Functions';
import CasinoIcon from '@mui/icons-material/Casino';
import DescriptionIcon from '@mui/icons-material/Description';

// Components are now created through the proper tab system, no need for direct imports

// Import centralized tab colors
import { getTabColors } from '@/core/utils/tabColors';

interface HomeTabProps {
  openNewTab: (id: string, title: string, content: React.ReactNode) => void;
}

interface QuickAction {
  id: string;
  title: string;
  description: string;
  iconComponent: React.ComponentType;
  colors?: ReturnType<typeof getTabColors>;
}

interface RecentFile {
  name: string;
  type: string;
  iconComponent: React.ComponentType;
  color: string;
  key?: string;
}

// Memoized constants to prevent recreation on every render
const QUICK_ACTIONS_CONFIG = [
  {
    id: 'spreadsheet',
    title: 'Spreadsheet',
    description: 'Data analysis and manipulation',
    iconComponent: TableChartIcon,
  },
  {
    id: 'fitting',
    title: 'Fitting',
    description: 'Curve fitting and regression',
    iconComponent: ShowChartIcon,
  },
  {
    id: 'solver',
    title: 'Solver',
    description: 'Mathematical equation solving',
    iconComponent: FunctionsIcon,
  },
  {
    id: 'montecarlo',
    title: 'Monte Carlo',
    description: 'Statistical simulations',
    iconComponent: CasinoIcon,
  }
] as const;

// Memoized recent files data
const RECENT_FILES_DATA = [
  { name: 'pendulum_data.csv', type: 'Data File', iconComponent: TableChartIcon, color: '#64b5f6' },
  { name: 'transistor_curves.xlsx', type: 'Spreadsheet', iconComponent: TableChartIcon, color: '#64b5f6' },
  { name: 'g_measurement.anafis', type: 'Project', iconComponent: DescriptionIcon, color: '#ba68c8' },
  { name: 'circuit_analysis.fit', type: 'Fitting Result', iconComponent: ShowChartIcon, color: '#ffb74d' },
] as const;

// Memoized styles to prevent recreation
const HERO_LOGO_STYLES = {
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
  },
} as const;

const HomeTab: React.FC<HomeTabProps> = ({ openNewTab }) => {
  // Optimized click handler using proper tab creation flow
  const handleNewTabClick = useCallback((tabType: string, title: string) => {
    const uniqueId = `${tabType}-${Date.now()}`;
    // Use openNewTab without pre-created content - let the tab system create the content
    openNewTab(uniqueId, title, null);
  }, [openNewTab]);

  // Memoize quick actions with colors to prevent recalculation
  const quickActions = useMemo(() =>
    QUICK_ACTIONS_CONFIG.map(action => ({
      ...action,
      colors: getTabColors(action.id),
    })), []
  );

  // Memoize recent files with icons to prevent recreation
  const recentFiles = useMemo(() =>
    RECENT_FILES_DATA.map((file, index) => ({
      ...file,
      key: `${file.name}-${index}`, // Stable key for React
    })), []
  );

  return (
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
        <Box sx={HERO_LOGO_STYLES}>
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
          {quickActions.map((action) => {
            const IconComponent = action.iconComponent;
            return (
              <QuickActionCard
                key={action.id}
                action={action}
                colors={action.colors}
                onNewTabClick={handleNewTabClick}
                IconComponent={IconComponent}
              />
            );
          })}
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
              {recentFiles.map((file, index) => {
                const IconComponent = file.iconComponent;
                return (
                  <RecentFileItem
                    key={file.key}
                    file={file}
                    index={index}
                    totalFiles={recentFiles.length}
                    IconComponent={IconComponent}
                  />
                );
              })}
            </List>
          </Box>
        </Paper>
      </Box>
    </Box>
  );
};

// Memoized QuickActionCard component to prevent unnecessary re-renders
const QuickActionCard = React.memo<{
  action: QuickAction;
  colors: ReturnType<typeof getTabColors>;
  onNewTabClick: (tabType: string, title: string) => void;
  IconComponent: React.ComponentType<Record<string, unknown>>;
}>(({ action, colors, onNewTabClick, IconComponent }) => (
  <Box sx={{ maxWidth: '280px', flex: '0 0 auto' }}>
    <Card
      sx={{
        height: 200,
        width: 200,
        cursor: 'pointer',
        transition: 'all 0.3s ease-in-out',
        background: 'linear-gradient(135deg, rgba(26, 26, 26, 0.8) 0%, rgba(42, 42, 42, 0.4) 100%)',
        backdropFilter: 'blur(10px)',
        border: '1px solid rgba(255, 255, 255, 0.1)',
        '&:hover': {
          transform: 'translateY(-8px)',
          boxShadow: `0 20px 40px ${colors.primary}30`,
          border: '1px solid',
          borderColor: colors.primary,
        },
      }}
      onClick={() => onNewTabClick(action.id, action.title)}
    >
      <CardContent sx={{ p: 3, textAlign: 'center' }}>
        <Avatar
          sx={{
            width: 60,
            height: 60,
            mx: 'auto',
            mb: 2,
            bgcolor: `${colors.primary}15`,
            color: colors.primary,
            border: '2px solid',
            borderColor: colors.primary,
            opacity: 1,
            transition: 'all 0.3s ease-in-out',
            boxShadow: `0 4px 12px ${colors.primary}30`,
            '&:hover': {
              opacity: 1,
              transform: 'scale(1.1)',
              boxShadow: `0 8px 20px ${colors.primary}50`,
            },
          }}
        >
          <IconComponent sx={{ fontSize: 28 }} />
        </Avatar>
        <Typography variant="h6" sx={{ mb: 1, fontWeight: 600 }}>
          {action.title}
        </Typography>
        <Typography variant="body2" color="text.secondary">
          {action.description}
        </Typography>
      </CardContent>
    </Card>
  </Box>
));

// Memoized RecentFileItem component
const RecentFileItem = React.memo<{
  file: RecentFile;
  index: number;
  totalFiles: number;
  IconComponent: React.ComponentType<Record<string, unknown>>;
}>(({ file, index, totalFiles, IconComponent }) => (
  <ListItemButton
    sx={{
      borderBottom: index < totalFiles - 1 ? '1px solid rgba(255, 255, 255, 0.1)' : 'none',
      '&:hover': {
        bgcolor: 'rgba(0, 212, 255, 0.1)',
      },
    }}
  >
    <ListItemIcon>
      <Avatar sx={{ width: 32, height: 32, bgcolor: `${file.color}20`, color: file.color }}>
        <IconComponent />
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
));

export default React.memo(HomeTab);
