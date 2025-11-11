import { Component, ReactNode, ErrorInfo } from 'react';
import { Box, Typography, Button, Paper, Alert, AlertTitle } from '@mui/material';
import { ErrorOutline, Refresh, BugReport } from '@mui/icons-material';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';

interface Props {
  children: ReactNode;
  title?: string;
  componentName?: string;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorId: string;
}

export class SpreadsheetErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    // Generate unique error ID for tracking
    const errorId = `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      hasError: true,
      error,
      errorId,
    };
  }

  override componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details
    console.error('Spreadsheet Error Boundary caught an error:', {
      error: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      errorId: this.state.errorId,
    });

    // Update state with error info
    this.setState({ errorInfo });

    // Call optional error handler
    this.props.onError?.(error, errorInfo);

    // Error reporting is handled through user-initiated bug reports only
    // No automatic error collection or telemetry - users must manually report bugs
  }

  handleRetry = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
    });
  };

  handleReportBug = () => {
    const { error, errorId } = this.state;
    const reportData = {
      errorId,
      message: error?.message,
      stack: error?.stack,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
    };

    // Copy error details to clipboard
    navigator.clipboard.writeText(JSON.stringify(reportData, null, 2))
      .then(() => {
        alert('Error details copied to clipboard. Please paste them in a bug report.');
      })
      .catch(() => {
        alert('Failed to copy error details. Please check the console for details.');
      });
  };

  override render() {
    if (this.state.hasError) {
      const { title = 'Spreadsheet Error', componentName } = this.props;
      const { error, errorId } = this.state;

      return (
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: 400,
            p: 3,
            bgcolor: '#0a0a0a',
          }}
        >
          <Paper
            elevation={3}
            sx={{
              p: 4,
              maxWidth: 600,
              width: '100%',
              bgcolor: '#1a1a1a',
              border: `1px solid ${anafisColors.buttons.close}`,
            }}
          >
            <Box sx={{ textAlign: 'center', mb: 3 }}>
              <ErrorOutline
                sx={{
                  fontSize: 64,
                  color: anafisColors.buttons.close,
                  mb: 2
                }}
              />
              <Typography variant="h5" sx={{ color: 'white', mb: 1 }}>
                {title}
              </Typography>
              {componentName && (
                <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.7)', mb: 2 }}>
                  Error in: {componentName}
                </Typography>
              )}
              <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.5)', fontFamily: 'monospace' }}>
                Error ID: {errorId}
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
              <AlertTitle sx={{ fontWeight: 'bold' }}>Something went wrong</AlertTitle>
              <Typography variant="body2">
                {error?.message ?? 'An unexpected error occurred in the spreadsheet component.'}
              </Typography>
            </Alert>

            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
              <Button
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
                variant="outlined"
                startIcon={<BugReport />}
                onClick={this.handleReportBug}
                sx={{
                  borderColor: 'rgba(255, 255, 255, 0.3)',
                  color: 'white',
                  '&:hover': {
                    borderColor: 'rgba(255, 255, 255, 0.5)',
                    bgcolor: 'rgba(255, 255, 255, 0.05)'
                  }
                }}
              >
                Report Bug
              </Button>
            </Box>

            {process.env.NODE_ENV === 'development' && error?.stack && (
              <Box sx={{ mt: 3 }}>
                <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.5)', display: 'block', mb: 1 }}>
                  Stack Trace (Development Only):
                </Typography>
                <Paper
                  sx={{
                    p: 2,
                    bgcolor: '#0a0a0a',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    maxHeight: 200,
                    overflow: 'auto'
                  }}
                >
                  <Typography
                    variant="caption"
                    component="pre"
                    sx={{
                      color: '#ff6b6b',
                      fontFamily: 'monospace',
                      fontSize: '11px',
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-word'
                    }}
                  >
                    {error.stack}
                  </Typography>
                </Paper>
              </Box>
            )}
          </Paper>
        </Box>
      );
    }

    return this.props.children;
  }
}