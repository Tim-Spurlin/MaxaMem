import React, { useState } from 'react'
import { useGeneration } from '../hooks/useGeneration'
import { PromptEditor } from '../components/PromptEditor'
import { GenerationProgress } from '../components/GenerationProgress'

export const ProjectWizard: React.FC = () => {
  const [prompt, setPrompt] = useState('')
  const [projectName, setProjectName] = useState('')
  const { generate, status, progress } = useGeneration()

  const steps = [
    { id: 'dev_plan', name: 'Development Plan' },
    { id: 'architecture', name: 'Technical Architecture' },
    { id: 'blueprint', name: 'Blueprint JSON' },
    { id: 'readme', name: 'Main README' },
    { id: 'tree', name: 'Directory Tree' },
    { id: 'schema', name: 'Communication Schema' },
    { id: 'agents', name: 'Directory Documentation' },
    { id: 'github', name: 'GitHub Repository' }
  ]

  const handleGenerate = async () => {
    await generate({
      prompt,
      projectName,
      githubRepo: projectName.toLowerCase().replace(/\s+/g, '-')
    })
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Create New Project</h1>

      <div className="space-y-6">
        <input
          type="text"
          placeholder="Project Name"
          value={projectName}
          onChange={e => setProjectName(e.target.value)}
          className="w-full p-3 border rounded-lg"
        />

        <PromptEditor
          value={prompt}
          onChange={setPrompt}
          placeholder="Describe your project in detail..."
        />

        <button
          onClick={handleGenerate}
          disabled={!prompt || !projectName || status === 'generating'}
          className="w-full bg-blue-600 text-white py-3 rounded-lg hover:bg-blue-700 disabled:opacity-50"
        >
          Generate Project
        </button>

        {status === 'generating' && (
          <GenerationProgress steps={steps} currentStep={progress.currentStep} />
        )}
      </div>
    </div>
  )
}
