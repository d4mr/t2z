import type { Step, StepId } from '../lib/types';

interface Props {
  steps: Step[];
  currentStep: StepId;
  onStepClick: (stepId: StepId) => void;
}

export function StepIndicator({ steps, currentStep, onStepClick }: Props) {
  return (
    <div className="flex flex-col gap-1">
      {steps.map((step, index) => {
        const isCurrent = step.id === currentStep;
        const isCompleted = step.completed;
        const isClickable = isCompleted || steps.slice(0, index).every(s => s.completed);
        
        return (
          <button
            key={step.id}
            onClick={() => isClickable && onStepClick(step.id)}
            disabled={!isClickable}
            className={`
              text-left px-4 py-3 rounded-lg transition-all
              ${isCurrent 
                ? 'bg-amber-500/20 border border-amber-500/50 text-amber-200' 
                : isCompleted 
                  ? 'bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 hover:bg-emerald-500/20' 
                  : 'bg-white/5 border border-white/10 text-gray-400'
              }
              ${isClickable && !isCurrent ? 'cursor-pointer' : ''}
              ${!isClickable ? 'opacity-50 cursor-not-allowed' : ''}
            `}
          >
            <div className="flex items-center gap-3">
              <div className={`
                w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold
                ${isCompleted 
                  ? 'bg-emerald-500 text-black' 
                  : isCurrent 
                    ? 'bg-amber-500 text-black' 
                    : 'bg-gray-600 text-gray-300'
                }
              `}>
                {isCompleted ? '✓' : index + 1}
              </div>
              <div>
                <div className="font-medium text-sm">{step.title}</div>
                <div className="text-xs opacity-70">{step.description}</div>
              </div>
            </div>
          </button>
        );
      })}
    </div>
  );
}

