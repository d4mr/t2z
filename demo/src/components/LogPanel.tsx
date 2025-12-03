import { useEffect, useRef } from 'react';
import type { LogEntry } from '../lib/types';

interface Props {
  logs: LogEntry[];
}

export function LogPanel({ logs }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div 
      ref={containerRef}
      className="h-48 overflow-y-auto bg-black/40 rounded-lg p-3 font-mono text-xs"
    >
      {logs.length === 0 ? (
        <div className="text-gray-500 italic">No logs yet...</div>
      ) : (
        logs.map(log => (
          <div key={log.id} className="mb-2">
            <div className="flex items-start gap-2">
              <span className="text-gray-500 shrink-0">
                {log.timestamp.toLocaleTimeString()}
              </span>
              <span className={`shrink-0 px-1.5 py-0.5 rounded text-[10px] font-bold uppercase ${
                log.type === 'error' 
                  ? 'bg-red-500/20 text-red-400' 
                  : log.type === 'success'
                    ? 'bg-emerald-500/20 text-emerald-400'
                    : log.type === 'code'
                      ? 'bg-purple-500/20 text-purple-400'
                      : 'bg-blue-500/20 text-blue-400'
              }`}>
                {log.step}
              </span>
              <span className={`${
                log.type === 'error' ? 'text-red-400' : 'text-gray-300'
              }`}>
                {log.message}
              </span>
            </div>
            {log.data && (
              <pre className="mt-1 ml-20 text-gray-500 overflow-x-auto whitespace-pre-wrap break-all">
                {log.data.length > 200 ? `${log.data.slice(0, 200)}...` : log.data}
              </pre>
            )}
          </div>
        ))
      )}
    </div>
  );
}

