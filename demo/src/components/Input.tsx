import { type InputHTMLAttributes } from 'react';

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
}

export function Input({ label, error, hint, className = '', ...props }: Props) {
  return (
    <div className="space-y-1">
      {label && (
        <label className="block text-sm font-medium text-gray-300">
          {label}
        </label>
      )}
      <input
        className={`
          w-full px-3 py-2 bg-black/40 border rounded-lg
          text-white placeholder-gray-500 font-mono text-sm
          focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:border-amber-500
          ${error ? 'border-red-500' : 'border-white/20'}
          ${className}
        `}
        {...props}
      />
      {hint && !error && (
        <p className="text-xs text-gray-500">{hint}</p>
      )}
      {error && (
        <p className="text-xs text-red-400">{error}</p>
      )}
    </div>
  );
}

