import React from 'react'

interface Step {
  id: string
  name: string
}

interface GenerationProgressProps {
  steps: Step[]
  currentStep: string
}

export const GenerationProgress: React.FC<GenerationProgressProps> = ({ steps, currentStep }) => (
  <ol className="space-y-2">
    {steps.map(step => (
      <li
        key={step.id}
        className={step.id === currentStep ? 'font-semibold text-blue-600' : 'text-gray-600'}
      >
        {step.name}
      </li>
    ))}
  </ol>
)
