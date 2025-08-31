import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { GameState, GamePhase, LLMConfig, GameConfig, RoleType } from '../types'
import { gameAPI, configAPI } from '../api'
import { ElMessage, ElNotification } from 'element-plus'

export const useGameStore = defineStore('game', () => {
  // 状态
  const gameState = ref<GameState | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const llmConfig = ref<LLMConfig>({
    provider: 'openai',
    apiKey: '',
    baseUrl: 'https://api.openai.com',
    model: 'gpt-4',
    maxTokens: 2000,
    temperature: 0.7,
    timeout: 60
  })
  const gameConfig = ref<GameConfig>({
    totalPlayers: 8,
    roleDistribution: {
      werewolf: 2,
      villager: 4,
      seer: 1,
      witch: 1,
      hunter: 0,
      guard: 0,
      idiot: 0
    } as Record<RoleType, number>,
    discussionTime: 300,
    votingTime: 60,
    enableVoice: false
  })

  // 计算属性
  const isGameRunning = computed(() => {
    return gameState.value && gameState.value.phase !== GamePhase.PREPARATION && gameState.value.phase !== GamePhase.GAME_OVER
  })

  const alivePlayers = computed(() => {
    return gameState.value?.players.filter(p => p.isAlive) || []
  })

  const deadPlayers = computed(() => {
    return gameState.value?.deadPlayers || []
  })

  const currentPhaseText = computed(() => {
    if (!gameState.value) return ''
    
    const phaseMap = {
      [GamePhase.PREPARATION]: '准备阶段',
      [GamePhase.NIGHT]: '夜晚',
      [GamePhase.DAY_DISCUSSION]: '白天讨论',
      [GamePhase.VOTING]: '投票阶段',
      [GamePhase.LAST_WORDS]: '遗言',
      [GamePhase.GAME_OVER]: '游戏结束'
    }
    
    return phaseMap[gameState.value.phase] || '未知阶段'
  })

  const humanPlayer = computed(() => {
    return gameState.value?.players.find(p => !p.isAI)
  })

  // 方法
  const setLoading = (loading: boolean) => {
    isLoading.value = loading
  }

  const setError = (errorMessage: string | null) => {
    error.value = errorMessage
    if (errorMessage) {
      ElMessage.error(errorMessage)
    }
  }

  // LLM配置管理
  const updateLLMConfig = async (config: Partial<LLMConfig>) => {
    try {
      setLoading(true)
      Object.assign(llmConfig.value, config)
      await configAPI.updateLLMConfig(llmConfig.value)
      ElMessage.success('LLM配置已更新')
    } catch (err) {
      setError(`更新LLM配置失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const testLLMConnection = async () => {
    try {
      setLoading(true)
      const result = await configAPI.testLLMConnection()
      if (result) {
        ElMessage.success('LLM连接测试成功')
      } else {
        ElMessage.warning('LLM连接测试失败')
      }
      return result
    } catch (err) {
      setError(`LLM连接测试失败: ${err}`)
      return false
    } finally {
      setLoading(false)
    }
  }

  // 游戏控制
  const createGame = async (config: GameConfig) => {
    try {
      setLoading(true)
      setError(null)
      
      const newGameState = await gameAPI.startNewGame(config)
      gameState.value = newGameState
      gameConfig.value = config
      
      ElNotification({
        title: '游戏创建成功',
        message: `已创建${config.totalPlayers}人局游戏`,
        type: 'success'
      })
      
      return newGameState
    } catch (err) {
      setError(`创建游戏失败: ${err}`)
      throw err
    } finally {
      setLoading(false)
    }
  }

  const startGame = async () => {
    try {
      setLoading(true)
      await gameAPI.launchGame()
      await refreshGameState()
      
      ElNotification({
        title: '游戏开始',
        message: '狼人杀游戏正式开始！',
        type: 'success'
      })
    } catch (err) {
      setError(`启动游戏失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const endGame = async () => {
    try {
      setLoading(true)
      await gameAPI.endGame()
      gameState.value = null
      
      ElMessage.success('游戏已结束')
    } catch (err) {
      setError(`结束游戏失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const refreshGameState = async () => {
    try {
      const state = await gameAPI.getGameState()
      gameState.value = state
    } catch (err) {
      setError(`获取游戏状态失败: ${err}`)
    }
  }

  // 玩家行动
  const playerVote = async (targetId: string) => {
    try {
      setLoading(true)
      await gameAPI.playerVote('human_player', targetId)
      await refreshGameState()
      
      ElMessage.success(`已投票给 ${targetId}`)
    } catch (err) {
      setError(`投票失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const playerSpeech = async (content: string) => {
    try {
      setLoading(true)
      await gameAPI.playerSpeech('human_player', content)
      await refreshGameState()
      
      ElMessage.success('发言已提交')
    } catch (err) {
      setError(`发言失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const generateAISpeech = async (playerId: string) => {
    try {
      const speech = await gameAPI.generateAISpeech(playerId)
      await refreshGameState()
      return speech
    } catch (err) {
      setError(`生成AI发言失败: ${err}`)
      return ''
    }
  }

  // 配置管理
  const updateGameConfig = async (config: Partial<GameConfig>) => {
    try {
      Object.assign(gameConfig.value, config)
      await configAPI.updateGameConfig(gameConfig.value)
      ElMessage.success('游戏配置已更新')
    } catch (err) {
      setError(`更新游戏配置失败: ${err}`)
    }
  }

  const loadAppConfig = async () => {
    try {
      const config = await configAPI.getAppConfig()
      if ((config as any).llm) {
        llmConfig.value = (config as any).llm
      }
      if ((config as any).game) {
        gameConfig.value = (config as any).game
      }
    } catch (err) {
      setError(`加载配置失败: ${err}`)
    }
  }

  const exportConfig = async () => {
    try {
      const configJson = await configAPI.exportConfig()
      // 创建下载链接
      const blob = new Blob([configJson], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'mindwolf-config.json'
      a.click()
      URL.revokeObjectURL(url)
      
      ElMessage.success('配置已导出')
    } catch (err) {
      setError(`导出配置失败: ${err}`)
    }
  }

  const importConfig = async (file: File) => {
    try {
      const text = await file.text()
      await configAPI.importConfig(text)
      await loadAppConfig()
      
      ElMessage.success('配置已导入')
    } catch (err) {
      setError(`导入配置失败: ${err}`)
    }
  }

  return {
    // 状态
    gameState,
    isLoading,
    error,
    llmConfig,
    gameConfig,
    
    // 计算属性
    isGameRunning,
    alivePlayers,
    deadPlayers,
    currentPhaseText,
    humanPlayer,
    
    // 方法
    setLoading,
    setError,
    updateLLMConfig,
    testLLMConnection,
    createGame,
    startGame,
    endGame,
    refreshGameState,
    playerVote,
    playerSpeech,
    generateAISpeech,
    updateGameConfig,
    loadAppConfig,
    exportConfig,
    importConfig
  }
})