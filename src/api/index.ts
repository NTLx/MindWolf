import { invoke } from '@tauri-apps/api/core'
import type { LLMConfig, GameConfig, GameState } from '../types'

// 应用配置API
export const configAPI = {
  async getAppConfig() {
    return await invoke('get_app_config')
  },

  async updateLLMConfig(config: LLMConfig) {
    return await invoke('update_llm_config', { config })
  },

  async testLLMConnection(): Promise<boolean> {
    return await invoke('test_llm_connection')
  },

  async updateGameConfig(config: GameConfig) {
    return await invoke('update_game_config', { config })
  },

  async exportConfig(): Promise<string> {
    return await invoke('export_config')
  },

  async importConfig(configJson: string) {
    return await invoke('import_config', { configJson })
  }
}

// 游戏API
export const gameAPI = {
  async startNewGame(config: GameConfig): Promise<GameState> {
    return await invoke('start_new_game', { config })
  },

  async launchGame() {
    return await invoke('launch_game')
  },

  async getGameState(): Promise<GameState | null> {
    return await invoke('get_game_state')
  },

  async playerVote(voterId: string, targetId: string) {
    return await invoke('player_vote', { voterId, targetId })
  },

  async playerSpeech(playerId: string, content: string) {
    return await invoke('player_speech', { playerId, content })
  },

  async generateAISpeech(playerId: string): Promise<string> {
    return await invoke('generate_ai_speech', { playerId })
  },

  async endGame() {
    return await invoke('end_game')
  }
}

// AI相关API
export const aiAPI = {
  async generateAIResponse(prompt: string): Promise<string> {
    return await invoke('generate_ai_response', { prompt })
  }
}

// 系统API
export const systemAPI = {
  async getAppVersion(): Promise<string> {
    return await invoke('get_app_version')
  }
}