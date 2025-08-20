export interface User {
  id: string
  email: string
  full_name: string
  subscription_tier: 'free' | 'starter' | 'professional' | 'enterprise'
}

export interface Project {
  id: string
  name: string
  description: string
  status: 'pending' | 'generating' | 'complete' | 'failed'
  progress: number
  repository_url?: string
  technologies: string[]
  created_at: string
  updated_at: string
}

export interface CreateProjectRequest {
  name: string
  description: string
  technologies: string[]
}

export interface LoginRequest {
  email: string
  password: string
}

export interface CreateUserRequest {
  email: string
  password: string
  full_name: string
}

export interface LoginResponse {
  token: string
  user: User
}