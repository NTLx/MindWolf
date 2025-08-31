<template>
  <div class="replay-viewer">
    <div class="replay-header">
      <h2>游戏复盘 - {{ replay?.game_id }}</h2>
      <div class="replay-actions">
        <el-button @click="exportReplay" type="primary" :icon="Download">
          导出复盘
        </el-button>
        <el-button @click="shareReplay" :icon="Share">
          分享
        </el-button>
        <el-button @click="$router.back()" :icon="ArrowLeft">
          返回
        </el-button>
      </div>
    </div>

    <div class="replay-content" v-if="replay">
      <!-- 游戏基本信息 -->
      <el-card class="game-info-card" shadow="hover">
        <template #header>
          <span class="card-title">游戏信息</span>
        </template>
        <div class="game-info">
          <div class="info-item">
            <span class="label">游戏时间:</span>
            <span class="value">{{ formatDateTime(replay.start_time) }}</span>
          </div>
          <div class="info-item">
            <span class="label">游戏时长:</span>
            <span class="value">{{ formatDuration(replay.start_time, replay.end_time) }}</span>
          </div>
          <div class="info-item">
            <span class="label">总轮数:</span>
            <span class="value">{{ replay.analysis?.game_statistics.total_rounds || 0 }}</span>
          </div>
          <div class="info-item" v-if="replay.game_result">
            <span class="label">获胜方:</span>
            <span class="value winner" :class="getFactionClass(replay.game_result.winner)">
              {{ getFactionName(replay.game_result.winner) }}
            </span>
          </div>
        </div>
      </el-card>

      <!-- 玩家信息 -->
      <el-card class="players-card" shadow="hover">
        <template #header>
          <span class="card-title">玩家信息</span>
        </template>
        <div class="players-grid">
          <div 
            v-for="player in replay.players" 
            :key="player.id"
            class="player-card"
            :class="getPlayerStatusClass(player)"
          >
            <div class="player-avatar">
              <el-avatar :size="50">
                {{ player.name.charAt(0) }}
              </el-avatar>
            </div>
            <div class="player-info">
              <div class="player-name">{{ player.name }}</div>
              <div class="player-role" :class="getRoleClass(player.role)">
                {{ getRoleName(player.role) }}
              </div>
              <div class="player-performance" v-if="getPlayerPerformance(player.id)">
                <el-rate 
                  v-model="getPlayerPerformance(player.id).overall_rating" 
                  disabled 
                  show-score
                  text-color="#ff9900"
                  score-template="{value}分"
                />
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 标签页内容 -->
      <el-tabs v-model="activeTab" class="replay-tabs">
        <!-- 游戏时间线 -->
        <el-tab-pane label="游戏时间线" name="timeline">
          <div class="timeline-container">
            <el-timeline>
              <el-timeline-item
                v-for="event in sortedEvents"
                :key="event.id"
                :timestamp="formatTime(event.timestamp)"
                :type="getEventType(event.event_type)"
                placement="top"
              >
                <div class="event-content">
                  <div class="event-header">
                    <span class="event-type">{{ getEventTypeName(event.event_type) }}</span>
                    <span class="event-round">第{{ event.round }}轮</span>
                    <span class="event-phase">{{ getPhaseName(event.phase) }}</span>
                  </div>
                  <div class="event-details">
                    <span v-if="event.player_id" class="event-player">
                      {{ getPlayerName(event.player_id) }}
                    </span>
                    <span class="event-content-text">{{ event.content }}</span>
                  </div>
                </div>
              </el-timeline-item>
            </el-timeline>
          </div>
        </el-tab-pane>

        <!-- AI决策分析 -->
        <el-tab-pane label="AI决策分析" name="ai-decisions">
          <div class="ai-decisions-container">
            <div class="ai-metrics" v-if="replay.analysis?.ai_performance_metrics">
              <h3>AI性能指标</h3>
              <div class="metrics-grid">
                <div class="metric-item">
                  <span class="metric-label">平均响应时间</span>
                  <span class="metric-value">
                    {{ replay.analysis.ai_performance_metrics.average_response_time.toFixed(2) }}ms
                  </span>
                </div>
                <div class="metric-item">
                  <span class="metric-label">决策信心度</span>
                  <el-progress 
                    :percentage="replay.analysis.ai_performance_metrics.decision_confidence * 100"
                    :stroke-width="8"
                    status="success"
                  />
                </div>
                <div class="metric-item">
                  <span class="metric-label">策略一致性</span>
                  <el-progress 
                    :percentage="replay.analysis.ai_performance_metrics.strategy_consistency * 100"
                    :stroke-width="8"
                    color="#409EFF"
                  />
                </div>
              </div>
            </div>

            <div class="ai-decisions-list">
              <h3>决策记录</h3>
              <el-table :data="pagedDecisions" stripe>
                <el-table-column prop="timestamp" label="时间" width="150">
                  <template #default="{ row }">
                    {{ formatTime(row.timestamp) }}
                  </template>
                </el-table-column>
                <el-table-column prop="player_id" label="AI玩家" width="120">
                  <template #default="{ row }">
                    {{ getPlayerName(row.player_id) }}
                  </template>
                </el-table-column>
                <el-table-column prop="decision_type" label="决策类型" width="100">
                  <template #default="{ row }">
                    {{ getDecisionTypeName(row.decision_type) }}
                  </template>
                </el-table-column>
                <el-table-column prop="confidence" label="信心度" width="100">
                  <template #default="{ row }">
                    <el-tag 
                      :type="getConfidenceType(row.confidence)"
                      size="small"
                    >
                      {{ (row.confidence * 100).toFixed(1) }}%
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="reasoning" label="推理过程" min-width="200">
                  <template #default="{ row }">
                    <el-popover
                      placement="top"
                      :width="400"
                      trigger="hover"
                      :content="row.reasoning"
                    >
                      <template #reference>
                        <span class="reasoning-preview">
                          {{ row.reasoning.substring(0, 50) }}...
                        </span>
                      </template>
                    </el-popover>
                  </template>
                </el-table-column>
                <el-table-column label="操作" width="80">
                  <template #default="{ row }">
                    <el-button 
                      size="small" 
                      @click="viewDecisionDetails(row)"
                      :icon="View"
                    >
                      详情
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
              
              <el-pagination
                v-model:current-page="currentDecisionPage"
                :page-size="decisionsPerPage"
                :total="replay.ai_decisions.length"
                layout="prev, pager, next"
                @current-change="handleDecisionPageChange"
              />
            </div>
          </div>
        </el-tab-pane>

        <!-- 数据分析 -->
        <el-tab-pane label="数据分析" name="analysis">
          <div class="analysis-container" v-if="replay.analysis">
            <!-- 获胜分析 -->
            <el-card class="analysis-card" shadow="hover">
              <template #header>
                <span class="card-title">获胜分析</span>
              </template>
              <div class="winner-analysis">
                <div class="winner-info">
                  <h4>获胜方: {{ getFactionName(replay.analysis.winner_analysis.winning_faction) }}</h4>
                  <p>{{ replay.analysis.winner_analysis.winning_reason }}</p>
                </div>
                <div class="key-factors">
                  <h5>关键因素:</h5>
                  <ul>
                    <li v-for="factor in replay.analysis.winner_analysis.key_factors" :key="factor">
                      {{ factor }}
                    </li>
                  </ul>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Download, Share, ArrowLeft, View } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import type { 
  GameReplay, 
  AIDecision,
  Faction,
  Role,
  GamePhase,
  GameEventType,
  DecisionType
} from '../types'

// 路由相关
const route = useRoute()
const router = useRouter()

// 响应式数据
const replay = ref<GameReplay | null>(null)
const loading = ref(true)
const activeTab = ref('timeline')

// AI决策分页
const currentDecisionPage = ref(1)
const decisionsPerPage = 10

// 计算属性
const sortedEvents = computed(() => {
  if (!replay.value) return []
  return [...replay.value.game_events].sort((a, b) => 
    new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  )
})

const pagedDecisions = computed(() => {
  if (!replay.value) return []
  const start = (currentDecisionPage.value - 1) * decisionsPerPage
  const end = start + decisionsPerPage
  return replay.value.ai_decisions.slice(start, end)
})

// 生命周期
onMounted(async () => {
  await loadReplay()
})

// 方法
const loadReplay = async () => {
  try {
    loading.value = true
    const gameId = route.params.id as string
    replay.value = await invoke<GameReplay>('get_game_replay', { gameId })
  } catch (error) {
    console.error('加载复盘数据失败:', error)
    ElMessage.error('加载复盘数据失败')
    router.back()
  } finally {
    loading.value = false
  }
}

// 格式化和获取名称的方法
const formatDateTime = (dateTime: string) => new Date(dateTime).toLocaleString('zh-CN')
const formatTime = (dateTime: string) => new Date(dateTime).toLocaleTimeString('zh-CN')
const formatDuration = (start: string, end: string | null | undefined) => {
  if (!end) return '进行中'
  const duration = new Date(end).getTime() - new Date(start).getTime()
  const minutes = Math.floor(duration / 60000)
  const seconds = Math.floor((duration % 60000) / 1000)
  return `${minutes}分${seconds}秒`
}

const getFactionName = (faction: Faction) => {
  const names: Record<string, string> = { 'Village': '好人阵营', 'Werewolf': '狼人阵营', 'ThirdParty': '第三方' }
  return names[faction as string] || faction
}

const getRoleName = (role: Role) => {
  const names: Record<string, string> = {
    'Villager': '村民', 'Werewolf': '狼人', 'Seer': '预言家',
    'Witch': '女巫', 'Hunter': '猎人', 'Guard': '守卫', 'Idiot': '白痴'
  }
  return names[String(role)] || String(role)
}

const getPhaseName = (phase: GamePhase) => {
  const names: Record<string, string> = {
    'Day': '白天', 'Night': '夜晚', 'Discussion': '讨论',
    'Voting': '投票', 'LastWords': '遗言'
  }
  return names[phase as string] || phase
}

const getEventTypeName = (eventType: GameEventType) => {
  const names: Record<string, string> = {
    'GameStart': '游戏开始', 'GameEnd': '游戏结束', 'Speech': '发言',
    'Vote': '投票', 'PlayerDeath': '玩家死亡'
  }
  return names[eventType as string] || eventType
}

const getDecisionTypeName = (decisionType: DecisionType) => {
  const names: Record<string, string> = {
    'Speech': '发言决策', 'Vote': '投票决策', 'SkillTarget': '技能目标'
  }
  return names[decisionType as string] || decisionType
}

// 样式类名方法
const getFactionClass = (faction: Faction) => {
  return ({ 'Village': 'faction-village', 'Werewolf': 'faction-werewolf' } as Record<string, string>)[faction as string] || ''
}

const getRoleClass = (role: Role) => {
  const isVillage = ['Villager', 'Seer', 'Witch', 'Hunter', 'Guard', 'Idiot'].includes(String(role))
  return isVillage ? 'role-village' : 'role-werewolf'
}

const getPlayerStatusClass = (player: any) => player.is_alive ? 'player-alive' : 'player-dead'
const getEventType = (eventType: GameEventType) => {
  const typeMap: Record<string, string> = { 'GameStart': 'success', 'GameEnd': 'danger', 'PlayerDeath': 'danger' }
  return typeMap[eventType as string] || 'info'
}

const getConfidenceType = (confidence: number) => {
  if (confidence >= 0.8) return 'success'
  if (confidence >= 0.6) return 'warning'
  return 'danger'
}

const getPlayerName = (playerId: string) => {
  if (!replay.value) return playerId
  const player = replay.value.players.find((p: any) => p.id === playerId)
  return player?.name || playerId
}

const getPlayerPerformance = (playerId: string) => {
  if (!replay.value?.analysis?.player_performance) return null
  return replay.value.analysis.player_performance[playerId]
}

// 事件处理
const handleDecisionPageChange = (page: number) => {
  currentDecisionPage.value = page
}

const viewDecisionDetails = (decision: AIDecision) => {
  console.log('查看决策详情:', decision)
}

const exportReplay = async () => {
  try {
    if (replay.value) {
      ElMessage.success('导出功能开发中')
    }
  } catch (error) {
    ElMessage.error('导出失败')
  }
}

const shareReplay = () => {
  ElMessage.success('分享功能开发中')
}
</script>

<style scoped>
.replay-viewer { padding: 20px; max-width: 1200px; margin: 0 auto; }
.replay-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.replay-header h2 { margin: 0; color: #303133; }
.replay-actions { display: flex; gap: 10px; }
.game-info-card, .players-card { margin-bottom: 20px; }
.card-title { font-weight: bold; color: #303133; }
.game-info { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; }
.info-item { display: flex; justify-content: space-between; }
.label { color: #606266; }
.value { font-weight: bold; }
.winner { padding: 2px 8px; border-radius: 4px; color: white; }
.faction-village { background-color: #67C23A; }
.faction-werewolf { background-color: #F56C6C; }
.players-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 15px; }
.player-card { display: flex; padding: 15px; border: 1px solid #EBEEF5; border-radius: 8px; background: #FAFAFA; }
.player-card.player-dead { opacity: 0.6; background: #F5F5F5; }
.player-avatar { margin-right: 15px; }
.player-info { flex: 1; }
.player-name { font-weight: bold; margin-bottom: 5px; }
.player-role { font-size: 12px; padding: 2px 6px; border-radius: 3px; color: white; margin-bottom: 8px; display: inline-block; }
.role-village { background-color: #67C23A; }
.role-werewolf { background-color: #F56C6C; }
</style>