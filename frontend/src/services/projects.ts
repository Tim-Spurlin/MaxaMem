import api from './api'
import { Project, CreateProjectRequest } from '../types'

export const projectService = {
  async createProject(projectData: CreateProjectRequest): Promise<Project> {
    const response = await api.post('/projects', projectData)
    return response.data
  },

  async getProjects(): Promise<Project[]> {
    const response = await api.get('/projects')
    return response.data
  },

  async getProject(id: string): Promise<Project> {
    const response = await api.get(`/projects/${id}`)
    return response.data
  },

  async deleteProject(id: string): Promise<void> {
    await api.delete(`/projects/${id}`)
  },
}