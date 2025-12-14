import axios, { type AxiosInstance } from 'axios'

// INT3RCEPTOR Backend API Client
const API_BASE_URL = 'http://localhost:3000/api'

class INT3RCEPTORClient {
  private client: AxiosInstance

  constructor(baseURL: string = API_BASE_URL) {
    this.client = axios.create({
      baseURL,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        console.error('API Error:', error.message)
        return Promise.reject(error)
      }
    )
  }

  // Traffic API
  async getTraffic(limit = 100) {
    try {
      const response = await this.client.get('/traffic', { params: { limit } })
      return response.data
    } catch (error) {
      console.error('Failed to fetch traffic:', error)
      return []
    }
  }

  async getRequest(id: string) {
    const response = await this.client.get(`/traffic/${id}`)
    return response.data
  }

  async clearTraffic() {
    const response = await this.client.delete('/traffic')
    return response.data
  }

  // Repeater API
  async sendRepeaterRequest(request: {
    method: string
    url: string
    headers?: Record<string, string>
    body?: string
  }) {
    const response = await this.client.post('/repeater/send', request)
    return response.data
  }

  // Settings API
  async getSettings() {
    const response = await this.client.get('/settings')
    return response.data
  }

  async updateSettings(settings: any) {
    const response = await this.client.put('/settings', settings)
    return response.data
  }

  // Plugins API
  async getPlugins() {
    try {
      const response = await this.client.get('/plugins')
      return response.data
    } catch (error) {
      console.error('Failed to fetch plugins:', error)
      return []
    }
  }

  async reloadPlugin(name: string) {
    const response = await this.client.post(`/plugins/${name}/reload`)
    return response.data
  }

  async togglePlugin(name: string, enabled: boolean) {
    const response = await this.client.post(`/plugins/${name}/toggle`, { enabled })
    return response.data
  }

  // Stats API
  async getStats() {
    try {
      const response = await this.client.get('/stats')
      return response.data
    } catch (error) {
      console.error('Failed to fetch stats:', error)
      return {
        requests: 0,
        memory: 0,
        connections: 0,
        uptime: 0,
      }
    }
  }

  // WebSocket API (for real-time updates)
  async getWebSocketStats() {
    try {
      const response = await this.client.get('/websocket/stats')
      return response.data
    } catch (error) {
      console.error('Failed to fetch WebSocket stats:', error)
      return {
        connections: 0,
        frames: 0,
      }
    }
  }

  // Rules API
  async getRules() {
    const response = await this.client.get('/rules')
    return response.data
  }

  async createRule(rule: any) {
    const response = await this.client.post('/rules', rule)
    return response.data
  }

  async deleteRule(id: string) {
    const response = await this.client.delete(`/rules/${id}`)
    return response.data
  }

  // Scope API
  async getScope() {
    const response = await this.client.get('/scope')
    return response.data
  }

  async updateScope(scope: any) {
    const response = await this.client.put('/scope', scope)
    return response.data
  }

  // Intruder API
  async intruderGenerate(request: any) {
    const response = await this.client.post('/intruder/generate', request)
    return response.data
  }

  async intruderStart(request: any) {
    const response = await this.client.post('/intruder/start', request)
    return response.data
  }

  async intruderStop() {
    const response = await this.client.post('/intruder/stop')
    return response.data
  }

  async intruderResults() {
    const response = await this.client.get('/intruder/results')
    return response.data
  }

  async intruderClear() {
    const response = await this.client.delete('/intruder/results')
    return response.data
  }

  // Health check
  async healthCheck() {
    try {
      const response = await this.client.get('/health')
      return response.status === 200
    } catch {
      return false
    }
  }
}

// Export singleton instance
export const apiClient = new INT3RCEPTORClient()

// Export types
export interface TrafficItem {
  id: string
  method: string
  url: string
  status: number
  size: number
  time: number
  timestamp: string
}

export interface PluginInfo {
  name: string
  version: string
  enabled: boolean
  description: string
}

export interface Stats {
  requests: number
  memory: number
  connections: number
  uptime: number
}

export type AttackType = 'Sniper' | 'Battering' | 'Pitchfork' | 'ClusterBomb'

export interface IntruderPosition {
  start: number
  end: number
  name: string
}

export interface IntruderOptions {
  concurrency: number
  delay_ms: number
}

export interface IntruderConfig {
  positions: IntruderPosition[]
  payloads: string[]
  attack_type: AttackType
  options: IntruderOptions
}

export interface IntruderResult {
  request_id: number
  payload: string
  status_code: number
  response_length: number
  duration_ms: number
}
