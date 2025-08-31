<template>
  <div class="home-container">
    <div class="hero-section">
      <h1 class="game-title">智狼 (MindWolf)</h1>
      <p class="game-subtitle">AI狼人杀游戏</p>
      
      <div class="main-actions">
        <el-button 
          type="primary" 
          size="large" 
          @click="startQuickGame"
          class="action-button"
        >
          快速开始
        </el-button>
        
        <el-button 
          size="large" 
          @click="$router.push('/settings')"
          class="action-button"
        >
          游戏设置
        </el-button>
        
        <el-button 
          size="large" 
          @click="$router.push('/replay')"
          class="action-button"
        >
          查看复盘
        </el-button>
      </div>
    </div>

    <div class="features-section">
      <el-row :gutter="20">
        <el-col :span="8">
          <el-card class="feature-card">
            <h3>AI智能对手</h3>
            <p>与高智能AI进行策略博弈</p>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card class="feature-card">
            <h3>语音交互</h3>
            <p>支持语音识别和语音合成</p>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card class="feature-card">
            <h3>游戏复盘</h3>
            <p>详细的游戏分析和复盘功能</p>
          </el-card>
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'

const router = useRouter()

const startQuickGame = () => {
  router.push('/game')
  ElMessage.success('开始游戏!')
}
</script>

<style scoped>
.home-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.hero-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  padding: 40px;
}

.game-title {
  font-size: 4rem;
  margin-bottom: 20px;
  text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
}

.game-subtitle {
  font-size: 1.5rem;
  margin-bottom: 40px;
  opacity: 0.9;
}

.main-actions {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
  justify-content: center;
}

.action-button {
  min-width: 150px;
  height: 50px;
  font-size: 16px;
}

.features-section {
  padding: 40px;
  background: rgba(255,255,255,0.1);
  backdrop-filter: blur(10px);
}

.feature-card {
  text-align: center;
  height: 150px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.feature-card h3 {
  color: #333;
  margin-bottom: 10px;
}

.feature-card p {
  color: #666;
}
</style>