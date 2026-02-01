// UniverErrorBoundary.tsx - Modern error boundary with recovery mechanisms
import React, { Component, ReactNode } from 'react';

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | undefined;
  errorInfo: React.ErrorInfo | undefined;
  retryCount: number;
  isAutoRetryScheduled: boolean;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  maxRetries?: number;
}

export class UniverErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
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

          {import.meta.env.DEV && this.state.error && (
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
                {this.state.errorInfo && (
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