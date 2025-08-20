import React from 'react'
import { Link } from 'react-router-dom'

const HomePage: React.FC = () => {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-900 via-blue-800 to-purple-900">
      <header className="bg-white bg-opacity-10 backdrop-blur-sm">
        <nav className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex justify-between items-center">
            <div className="flex items-center">
              <h1 className="text-2xl font-bold text-white">MaxaMem 🧠</h1>
            </div>
            <div className="space-x-4">
              <Link
                to="/auth"
                className="text-white hover:text-blue-200 px-4 py-2 rounded-md transition-colors"
              >
                Sign In
              </Link>
              <Link
                to="/auth"
                className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md transition-colors"
              >
                Get Started
              </Link>
            </div>
          </div>
        </nav>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20">
        <div className="text-center">
          <h1 className="text-6xl font-bold text-white mb-6">
            Transform Ideas into
            <span className="text-blue-300"> Production-Ready</span>
            <br />
            Architectures
          </h1>
          <p className="text-xl text-blue-100 mb-8 max-w-3xl mx-auto">
            MaxaMem is an AI-powered SaaS platform that transforms project descriptions 
            into complete architectures with intelligent communication schemas, comprehensive 
            documentation, and fully scaffolded GitHub repositories.
          </p>
          <div className="space-x-4">
            <Link
              to="/auth"
              className="bg-blue-600 hover:bg-blue-700 text-white px-8 py-4 rounded-lg text-lg font-semibold transition-colors inline-block"
            >
              Start Building
            </Link>
            <button className="border border-blue-300 text-blue-100 hover:bg-blue-800 px-8 py-4 rounded-lg text-lg font-semibold transition-colors">
              View Demo
            </button>
          </div>
        </div>

        <div className="mt-20 grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="bg-white bg-opacity-10 backdrop-blur-sm rounded-lg p-8">
            <h3 className="text-2xl font-bold text-white mb-4">🤖 AI-Powered Generation</h3>
            <p className="text-blue-100">
              Leverage ChatGPT and Claude to generate comprehensive project architectures, 
              documentation, and communication schemas from simple descriptions.
            </p>
          </div>
          <div className="bg-white bg-opacity-10 backdrop-blur-sm rounded-lg p-8">
            <h3 className="text-2xl font-bold text-white mb-4">🔗 GitHub Integration</h3>
            <p className="text-blue-100">
              Automatically create and populate GitHub repositories with generated content, 
              complete directory structures, and detailed READMEs for every folder.
            </p>
          </div>
          <div className="bg-white bg-opacity-10 backdrop-blur-sm rounded-lg p-8">
            <h3 className="text-2xl font-bold text-white mb-4">📊 Communication Schemas</h3>
            <p className="text-blue-100">
              Our proprietary schema technology maps every component interaction, 
              providing complete visibility into system architecture and dependencies.
            </p>
          </div>
        </div>
      </main>
    </div>
  )
}

export default HomePage