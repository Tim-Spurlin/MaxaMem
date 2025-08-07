import { useState } from 'react'

interface GenerateOptions {
  prompt: string
  projectName: string
  githubRepo: string
}

export const useGeneration = () => {
  const [status, setStatus] = useState<'idle' | 'generating' | 'success' | 'error'>('idle')
  const [progress, setProgress] = useState<{ currentStep: string }>({ currentStep: '' })

  const generate = async (opts: GenerateOptions) => {
    setStatus('generating')
    try {
      // Placeholder for backend API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      setProgress({ currentStep: 'done' })
      setStatus('success')
    } catch (err) {
      console.error(err)
      setStatus('error')
    }
  }

  return { generate, status, progress }
}
