import { Component, ReactNode, ErrorInfo } from 'react';
import { Box, Typography, Button, Paper, Alert, AlertTitle, IconButton } from '@mui/material';
import { ErrorOutline, Refresh, Close } from '@mui/icons-material';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';

interface SidebarErrorBoundaryProps {
  children: ReactNode;
  sidebarName: string;
  onClose: () => void;
  onRetry?: () => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  retryCount: number;
}

export class SidebarErrorBoundary extends Component<SidebarErrorBoundaryProps, State> {
  constructor(props: SidebarErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      retryCount: 0,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
    };
  }

  override componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error(`Sidebar Error Boundary (${this.props.sidebarName}) caught an error:`, {
      error: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
    });

    this.setState({ errorInfo });
  }

  handleRetry = () => {
    const { onRetry } = this.props;
    const newRetryCount = this.state.retryCount + 1;

    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      retryCount: newRetryCount,
    });

    // Call optional retry handler
    onRetry?.();
  };

  handleClose = () => {
    this.props.onClose();
  };

  override render() {
    if (this.state.hasError) {
      const { sidebarName } = this.props;
      const { error, retryCount } = this.state;

      return (
        <Box
          sx={{
            ...sidebarStyles.container,
            px: 2,
            pt: 2,
            display: 'flex',
            flexDirection: 'column',
            minHeight: 300,
          }}
        >
          {/* Header */}
          <Box sx={{ ...sidebarStyles.header, mb: 2 }}>
            <Typography sx={{ ...sidebarStyles.text.header, color: anafisColors.buttons.close }}>
              ⚠️ {sidebarName} Error
            </Typography>
            <IconButton
              onClick={this.handleClose}
              sx={sidebarStyles.iconButton}
            >
              <Close />
            </IconButton>
          </Box>

          {/* Error Content */}
          <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <Box sx={{ textAlign: 'center', mb: 3 }}>
              <ErrorOutline
                sx={{
                  fontSize: 48,
                  color: anafisColors.buttons.close,
                  mb: 2
                }}
              />
              <Typography variant="h6" sx={{ color: 'white', mb: 1 }}>
                Something went wrong
              </Typography>
              <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.7)' }}>
                The {sidebarName.toLowerCase()} encountered an error
              </Typography>
            </Box>

            <Alert
              severity="error"
              sx={{
                mb: 3,
                bgcolor: 'rgba(244, 67, 54, 0.1)',
                color: 'white',
                '& .MuiAlert-icon': { color: anafisColors.buttons.close }
              }}
            >
              <AlertTitle sx={{ fontWeight: 'bold' }}>Error Details</AlertTitle>
              <Typography variant="body2">
                {error?.message ?? 'An unexpected error occurred.'}
              </Typography>
              {retryCount > 0 && (
                <Typography variant="caption" sx={{ display: 'block', mt: 1, color: 'rgba(255, 255, 255, 0.6)' }}>
                  Retry attempts: {retryCount}
                </Typography>
              )}
            </Alert>

            <Box sx={{ display: 'flex', gap: 1, flexDirection: 'column' }}>
              <Button
                fullWidth
                variant="contained"
                startIcon={<Refresh />}
                onClick={this.handleRetry}
                sx={{
                  bgcolor: anafisColors.spreadsheet,
                  '&:hover': { bgcolor: '#1565c0' }
                }}
              >
                Try Again
              </Button>

              <Button
                fullWidth
                variant="outlined"
                onClick={this.handleClose}
                sx={{
                  borderColor: 'rgba(255, 255, 255, 0.3)',
                  color: 'white',
                  '&:hover': {
                    borderColor: 'rgba(255, 255, 255, 0.5)',
                    bgcolor: 'rgba(255, 255, 255, 0.05)'
                  }
                }}
              >
                Close Sidebar
              </Button>
            </Box>

            {import.meta.env.DEV && error?.stack && (
              <Box sx={{ mt: 2 }}>
                <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.5)', display: 'block', mb: 1 }}>
                  Stack Trace (Development):
                </Typography>
                <Paper
                  sx={{
                    p: 1,
                    bgcolor: '#0a0a0a',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    maxHeight: 120,
                    overflow: 'auto'
                  }}
                >
                  <Typography
                    variant="caption"
                    component="pre"
                    sx={{
                      color: '#ff6b6b',
                      fontFamily: 'monospace',
                      fontSize: '10px',
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-word'
                    }}
                  >
                    {error.stack}
                  </Typography>
                </Paper>
              </Box>
            )}
          </Box>
        </Box>
      );
    }

    return this.props.children;
  }
}

// Import sidebar styles (assuming it exists)
import { sidebarStyles } from "@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles"