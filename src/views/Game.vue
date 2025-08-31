<template>
  <div class="game-container">
    <div class="game-status">
      <el-card class="status-card">
        <div class="status-info">
          <span class="game-day">第{{ gameState.day }}天</span>
          <span class="game-phase">{{ phaseText }}</span>
        </div>
      </el-card>
    </div>
    
    <div class="players-container">
      <div class="players-grid">
        <div 
          v-for="player in gameState.players" 
          :key="player.id"
          class="player-seat"
        >
          <el-card class="player-card">
            <div class="player-info">
              <div class="player-name">{{ player.name }}</div>
            </div>
          </el-card>
        </div>
      </div>
    </div>

    <div class="chat-container">
      <el-card class="chat-card">
        <template #header>
          <span>游戏发言</span>
        </template>
        <div class="chat-messages">
          <div>游戏聊天区域</div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const gameState = ref({
  phase: 'DAY_DISCUSSION',
  day: 1,
  players: [] as Array<{id: string, name: string}>
})

const phaseText = computed(() => {
  return '白天讨论'
})
</script>

<style scoped>
.game-container {
  padding: 20px;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.game-status {
  margin-bottom: 20px;
}

.status-info {
  display: flex;
  gap: 20px;
  align-items: center;
}

.game-day, .game-phase {
  font-weight: bold;
  font-size: 16px;
}

.players-container {
  flex: 1;
  margin-bottom: 20px;
}

.players-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.player-card {
  min-height: 100px;
}

.player-name {
  font-weight: bold;
  text-align: center;
}

.chat-container {
  height: 300px;
}

.chat-messages {
  height: 250px;
  overflow-y: auto;
  padding: 10px;
}
</style>