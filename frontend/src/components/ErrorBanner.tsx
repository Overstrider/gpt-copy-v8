"use client";
import { AlertCircle } from "lucide-react";

export interface ErrorBannerProps { message: string; onRetry?: () => void }

export function ErrorBanner({ message, onRetry }: ErrorBannerProps) {
  return (
    <div role="alert" className="flex items-center gap-2 bg-red-900/40 px-4 py-2 text-sm text-red-200">
      <AlertCircle size={16} />
      <span className="flex-1">{message}</span>
      {onRetry && (
        <button type="button" onClick={onRetry} className="underline">
          Retry
        </button>
      )}
    </div>
  );
}
