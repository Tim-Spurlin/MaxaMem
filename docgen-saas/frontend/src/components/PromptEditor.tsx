import React from 'react'

interface PromptEditorProps {
  value: string
  onChange: (val: string) => void
  placeholder?: string
}

export const PromptEditor: React.FC<PromptEditorProps> = ({ value, onChange, placeholder }) => (
  <textarea
    className="w-full h-40 p-3 border rounded-lg"
    value={value}
    onChange={e => onChange(e.target.value)}
    placeholder={placeholder}
  />
)
