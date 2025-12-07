import { StrictMode, Suspense, lazy } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';

const App = lazy(() => import('./App').then((m) => ({ default: m.App })));

function LoadingScreen() {
  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center space-y-6">
        <div className="w-16 h-16 mx-auto rounded-xl bg-gradient-to-br from-amber-400 to-amber-600 flex items-center justify-center font-bold text-black text-2xl animate-pulse">
          t2z
        </div>
        <div className="space-y-2">
          <h2 className="text-xl font-semibold text-white">Loading...</h2>
          <p className="text-sm text-gray-400">Preparing cryptographic primitives</p>
        </div>
        <div className="w-48 h-1 mx-auto bg-white/10 rounded-full overflow-hidden">
          <div
            className="h-full bg-gradient-to-r from-amber-400 to-amber-600 rounded-full"
            style={{ width: '60%', animation: 'loading 1.5s ease-in-out infinite' }}
          />
        </div>
      </div>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Suspense fallback={<LoadingScreen />}>
      <App />
    </Suspense>
  </StrictMode>,
);
