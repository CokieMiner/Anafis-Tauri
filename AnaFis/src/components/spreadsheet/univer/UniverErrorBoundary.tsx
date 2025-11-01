// UniverErrorBoundary.tsx - Enhanced error boundary with recovery mechanisms
import React, { Component, ReactNode, useCallback } from 'react';
import { Box, Typography, Button, Alert } from '@mui/material';
import { RefreshOutlined, BugReportOutlined } from '@mui/icons-material';

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | undefined;
  errorInfo: React.ErrorInfo | undefined;
}

interface LightweightErrorBoundaryState extends ErrorBoundaryState {
  retryCount: number;
  isAutoRetryScheduled: boolean;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  maxRetries?: number;
}

export class LightweightErrorBoundary extends Component<ErrorBoundaryProps, LightweightErrorBoundaryState> {
  private retryTimeoutId: ReturnType<typeof setTimeout> | null = null;

  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: undefined,
      errorInfo: undefined,
      retryCount: 0,
      isAutoRetryScheduled: false
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  override componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Univer Error Boundary caught an error:', error, errorInfo);
    
    this.setState({ errorInfo });
    
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    // Auto-retry mechanism for transient errors
    const maxRetries = this.props.maxRetries ?? 2;
    if (this.state.retryCount < maxRetries) {
      // Clear any pending timeout before scheduling a new one
      if (this.retryTimeoutId) {
        clearTimeout(this.retryTimeoutId);
        this.setState({ isAutoRetryScheduled: false });
      }
      
      this.retryTimeoutId = setTimeout(() => {
        this.setState(prevState => ({
          hasError: false,
          error: undefined,
          errorInfo: undefined,
          retryCount: prevState.retryCount + 1,
          isAutoRetryScheduled: false
        }));
        // Clear the timeout ID after execution
        this.retryTimeoutId = null;
      }, 1000 * (this.state.retryCount + 1)); // Linear backoff
      
      this.setState({ isAutoRetryScheduled: true });
    }
  }

  override componentWillUnmount() {
    if (this.retryTimeoutId) {
      clearTimeout(this.retryTimeoutId);
      this.retryTimeoutId = null;
      this.setState({ isAutoRetryScheduled: false });
    }
  }

  private handleManualRetry = () => {
    if (this.retryTimeoutId) {
      clearTimeout(this.retryTimeoutId);
      this.retryTimeoutId = null;
    }
    this.setState({
      hasError: false,
      error: undefined,
      errorInfo: undefined,
      retryCount: 0,
      isAutoRetryScheduled: false
    });
  };

  override render() {
    if (this.state.hasError) {
      return this.props.fallback ?? (
        <div style={{
          padding: '20px',
          border: '1px solid #ff6b6b',
          borderRadius: '4px',
          backgroundColor: '#ffeaea',
          color: '#d63031',
          textAlign: 'center'
        }}>
          <h3>Spreadsheet Error</h3>
          <p>An error occurred while rendering the spreadsheet.</p>
          
          <button
            onClick={this.handleManualRetry}
            style={{
              padding: '8px 16px',
              backgroundColor: '#d63031',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              marginTop: '10px'
            }}
          >
            Retry
          </button>

          {this.state.retryCount < (this.props.maxRetries ?? 2) && this.state.isAutoRetryScheduled && (
            <p style={{ fontSize: '12px', marginTop: '10px' }}>
              Auto-retry in progress... (Attempt {this.state.retryCount + 1})
            </p>
          )}

          {process.env.NODE_ENV === 'development' && this.state.error && (
            <details style={{ marginTop: '15px', textAlign: 'left' }}>
              <summary>Error Details (Development)</summary>
              <pre style={{ 
                fontSize: '11px', 
                marginTop: '5px',
                backgroundColor: '#f5f5f5',
                padding: '10px',
                borderRadius: '4px',
                overflow: 'auto',
                maxHeight: '200px'
              }}>
                {this.state.error.message}
                {this.state.errorInfo?.componentStack && (
                  `\n\nComponent Stack:${  this.state.errorInfo.componentStack}`
                )}
              </pre>
            </details>
          )}
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Hook for handling errors in functional components
 */
export function useErrorHandler() {
  return useCallback((error: Error) => {
    console.error('Univer operation error:', error);
    // In a real application, you might want to send this to an error reporting service
  }, []);
}

// Full-featured error boundary with detailed UI and retry functionality (for backward compatibility)
export class UniverErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: undefined,
      errorInfo: undefined
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error
    };
  }

  override componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    this.setState({
      error,
      errorInfo
    });

    // Log error for debugging
    console.error('Univer Error Boundary caught an error:', error, errorInfo);

    // Call custom error handler if provided
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  handleRetry = () => {
    this.setState({
      hasError: false,
      error: undefined,
      errorInfo: undefined
    });
  };

  override render() {
    if (this.state.hasError) {
      // Custom fallback UI if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: 400,
            p: 3,
            textAlign: 'center',
            backgroundColor: 'background.default',
            color: 'text.primary',
            width: '100%',
            height: '100%'
          }}
        >
          <BugReportOutlined sx={{ fontSize: 48, color: 'error.main', mb: 2 }} />

          <Typography variant="h6" gutterBottom color="text.primary">
            Spreadsheet Error
          </Typography>

          <Typography variant="body2" color="text.secondary" sx={{ mb: 2, maxWidth: 400 }}>
            Something went wrong with the spreadsheet. Click retry to continue.
          </Typography>

          <Alert severity="error" sx={{ mb: 2, maxWidth: 500 }}>
            <Typography variant="body2">
              {this.state.error?.message ?? 'Unknown error occurred'}
            </Typography>
          </Alert>

          <Button
            variant="contained"
            size="small"
            startIcon={<RefreshOutlined />}
            onClick={this.handleRetry}
            sx={{ mb: 1 }}
          >
            Retry
          </Button>

          {process.env.NODE_ENV === 'development' && this.state.errorInfo && (
            <Box sx={{ mt: 2, p: 1, bgcolor: 'background.paper', borderRadius: 1, maxWidth: 600, border: 1, borderColor: 'divider' }}>
              <Typography variant="caption" component="pre" sx={{ fontSize: '0.7rem', overflow: 'auto', color: 'text.secondary' }}>
                {this.state.error?.message}
              </Typography>
            </Box>
          )}
        </Box>
      );
    }

    return this.props.children;
  }
}