import { Component, ReactNode } from "react";
import { Alert } from "./ui";

export class ErrorBoundary extends Component<
  { children: ReactNode },
  { error: Error | null }
> {
  state: { error: Error | null } = { error: null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <Alert kind="danger">
          Something went wrong rendering this view. Please reload.
          <div className="mt-1 text-xs opacity-80">{this.state.error.message}</div>
        </Alert>
      );
    }
    return this.props.children;
  }
}
