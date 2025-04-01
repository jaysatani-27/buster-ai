import { Alert } from 'antd';
import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}

export class BusterChartErrorWrapper extends Component<Props, State> {
  state = {
    hasError: false
  };

  static getDerivedStateFromError(_: Error): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Chart Error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          className="flex h-full w-full items-center justify-center rounded border p-5"
          role="alert">
          <Alert
            message="Something went wrong rendering the chart. This is likely an error on our end. Please contact Buster support."
            type="error"
            showIcon
          />
        </div>
      );
    }

    return this.props.children;
  }
}
