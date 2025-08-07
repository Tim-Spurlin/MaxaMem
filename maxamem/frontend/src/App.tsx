import React, { useEffect, useState } from 'react'

export const App: React.FC = () => {
  const [health, setHealth] = useState<string>('checking...')
  
  useEffect(() => {
    fetch('http://localhost:8080/api/health')
      .then(res => res.text())
      .then(data => setHealth(data))
      .catch(() => setHealth('Backend not connected'))
  }, [])
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="container mx-auto p-8">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-5xl font-bold text-center text-gray-800 mb-4">
            MaxaMem SaaS
          </h1>
          <p className="text-center text-gray-600 text-lg mb-8">
            AI-powered project documentation generator
          </p>
          
          <div className="bg-white rounded-lg shadow-lg p-6">
            <h2 className="text-2xl font-semibold mb-4">System Status</h2>
            <div className="space-y-2">
              <div className="flex items-center justify-between p-3 bg-gray-50 rounded">
                <span className="font-medium">Frontend</span>
                <span className="text-green-600">✓ Running</span>
              </div>
              <div className="flex items-center justify-between p-3 bg-gray-50 rounded">
                <span className="font-medium">Backend API</span>
                <span className={health === 'OK' ? 'text-green-600' : 'text-red-600'}>
                  {health === 'OK' ? '✓ Connected' : '✗ ' + health}
                </span>
              </div>
            </div>
          </div>
          
          <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-white rounded-lg shadow p-6">
              <h3 className="font-semibold text-lg mb-2">Step 1</h3>
              <p className="text-gray-600">Describe your project</p>
            </div>
            <div className="bg-white rounded-lg shadow p-6">
              <h3 className="font-semibold text-lg mb-2">Step 2</h3>
              <p className="text-gray-600">AI generates documentation</p>
            </div>
            <div className="bg-white rounded-lg shadow p-6">
              <h3 className="font-semibold text-lg mb-2">Step 3</h3>
              <p className="text-gray-600">Deploy to GitHub</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
