<template>
  <div class="settings-container">
    <el-container>
      <el-aside width="200px">
        <el-menu
          v-model="activeMenu"
          class="settings-menu"
          @select="handleMenuSelect"
        >
          <el-menu-item index="game">
            <span>游戏设置</span>
          </el-menu-item>
          <el-menu-item index="ai">
            <span>AI设置</span>
          </el-menu-item>
          <el-menu-item index="voice">
            <span>语音设置</span>
          </el-menu-item>
          <el-menu-item index="about">
            <span>关于</span>
          </el-menu-item>
        </el-menu>
      </el-aside>

      <el-main>
        <!-- 游戏设置 -->
        <div v-show="activeMenu === 'game'" class="settings-panel">
          <h3>游戏设置</h3>
          <el-form :model="gameSettings" label-width="120px">
            <el-form-item label="玩家数量">
              <el-select v-model="gameSettings.playerCount">
                <el-option label="6人局" :value="6" />
                <el-option label="8人局" :value="8" />
                <el-option label="10人局" :value="10" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="讨论时间">
              <el-input-number 
                v-model="gameSettings.discussionTime"
                :min="60"
                :max="600"
                :step="30"
              />
              <span style="margin-left: 10px;">秒</span>
            </el-form-item>
            
            <el-form-item label="投票时间">
              <el-input-number 
                v-model="gameSettings.votingTime"
                :min="30"
                :max="180"
                :step="15"
              />
              <span style="margin-left: 10px;">秒</span>
            </el-form-item>
          </el-form>
        </div>

        <!-- AI设置 -->
        <div v-show="activeMenu === 'ai'" class="settings-panel">
          <h3>AI设置</h3>
          <el-form :model="aiSettings" label-width="140px">
            <el-form-item label="API地址">
              <el-input v-model="aiSettings.apiUrl" placeholder="输入OpenAI兼容的API地址" />
            </el-form-item>
            
            <el-form-item label="API密钥">
              <el-input 
                v-model="aiSettings.apiKey" 
                type="password"
                placeholder="输入API密钥"
                show-password
              />
            </el-form-item>
            
            <el-form-item label="模型名称">
              <el-input v-model="aiSettings.modelName" placeholder="例如: gpt-4o-realtime-preview-2024-12-17" />
            </el-form-item>
            
            <el-form-item label="使用实时API">
              <el-switch v-model="aiSettings.useRealtimeApi" />
              <span style="margin-left: 10px; font-size: 12px; color: #909399;">启用WebSocket实时对话</span>
            </el-form-item>
            
            <template v-if="aiSettings.useRealtimeApi">
              <el-divider content-position="left">实时API配置</el-divider>
              
              <el-form-item label="语音类型">
                <el-select v-model="aiSettings.voice" placeholder="请选择语音类型">
                  <el-option label="Alloy" value="alloy" />
                  <el-option label="Echo" value="echo" />
                  <el-option label="Shimmer" value="shimmer" />
                  <el-option label="Fable" value="fable" />
                  <el-option label="Onyx" value="onyx" />
                  <el-option label="Nova" value="nova" />
                </el-select>
              </el-form-item>
              
              <el-form-item label="输入音频格式">
                <el-select v-model="aiSettings.inputAudioFormat">
                  <el-option label="PCM16" value="pcm16" />
                  <el-option label="G711 μ-law" value="g711_ulaw" />
                  <el-option label="G711 A-law" value="g711_alaw" />
                </el-select>
              </el-form-item>
              
              <el-form-item label="输出音频格式">
                <el-select v-model="aiSettings.outputAudioFormat">
                  <el-option label="PCM16" value="pcm16" />
                  <el-option label="G711 μ-law" value="g711_ulaw" />
                  <el-option label="G711 A-law" value="g711_alaw" />
                </el-select>
              </el-form-item>
              
              <el-form-item label="支持模态">
                <el-checkbox-group v-model="aiSettings.modalities">
                  <el-checkbox label="text">文本</el-checkbox>
                  <el-checkbox label="audio">音频</el-checkbox>
                </el-checkbox-group>
              </el-form-item>
              
              <el-form-item label="系统指令">
                <el-input 
                  v-model="aiSettings.instructions" 
                  type="textarea" 
                  :rows="3"
                  placeholder="输入AI的系统指令，例如：你是一个狼人杀游戏AI助手"
                />
              </el-form-item>
              
              <el-form-item label="语音检测">
                <el-select v-model="aiSettings.turnDetectionType">
                  <el-option label="服务端VAD" value="server_vad" />
                  <el-option label="无检测" value="none" />
                </el-select>
              </el-form-item>
            </template>
            
            <el-divider content-position="left">通用配置</el-divider>
            
            <el-form-item label="AI难度">
              <el-select v-model="aiSettings.difficulty">
                <el-option label="简单" value="easy" />
                <el-option label="普通" value="normal" />
                <el-option label="困难" value="hard" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="温度参数">
              <el-slider 
                v-model="aiSettings.temperature"
                :min="0"
                :max="1"
                :step="0.1"
                show-input
              />
            </el-form-item>
            
            <el-form-item label="最大令牌数">
              <el-input-number 
                v-model="aiSettings.maxTokens"
                :min="100"
                :max="4000"
                :step="100"
              />
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" @click="testConnection">测试连接</el-button>
            </el-form-item>
          </el-form>
        </div>

        <!-- 语音设置 -->
        <div v-show="activeMenu === 'voice'" class="settings-panel">
          <h3>语音设置</h3>
          <el-form :model="voiceSettings" label-width="120px">
            <el-form-item label="启用语音">
              <el-switch v-model="voiceSettings.enabled" />
            </el-form-item>
            
            <el-form-item label="语音识别">
              <el-switch v-model="voiceSettings.speechRecognition" />
            </el-form-item>
            
            <el-form-item label="语音合成">
              <el-switch v-model="voiceSettings.speechSynthesis" />
            </el-form-item>
            
            <el-form-item label="输出音量">
              <el-slider 
                v-model="voiceSettings.volume"
                :min="0"
                :max="100"
                show-input
              />
            </el-form-item>
          </el-form>
        </div>

        <!-- 关于 -->
        <div v-show="activeMenu === 'about'" class="settings-panel">
          <h3>关于智狼</h3>
          <div class="about-content">
            <p>智狼 (MindWolf) 是一款基于AI技术的狼人杀游戏。</p>
            <p>版本: 0.1.0</p>
            <p>技术栈: Tauri + Vue3 + TypeScript + Rust</p>
          </div>
        </div>

        <div class="settings-footer">
          <el-button type="primary" @click="saveSettings">保存设置</el-button>
          <el-button @click="resetSettings">重置</el-button>
          <el-button @click="$router.back()">返回</el-button>
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { configAPI } from '../api'

const activeMenu = ref('game')

const gameSettings = ref({
  playerCount: 8,
  discussionTime: 300,
  votingTime: 60
})

const aiSettings = ref({
  apiUrl: '',
  apiKey: '',
  modelName: 'gpt-4o-realtime-preview-2024-12-17',
  difficulty: 'normal',
  useRealtimeApi: false,
  voice: 'alloy',
  inputAudioFormat: 'pcm16',
  outputAudioFormat: 'pcm16',
  modalities: ['text', 'audio'],
  instructions: '你是一个狼人杀游戏AI助手',
  turnDetectionType: 'server_vad',
  temperature: 0.7,
  maxTokens: 2000
})

const voiceSettings = ref({
  enabled: false,
  speechRecognition: false,
  speechSynthesis: true,
  volume: 80
})

const handleMenuSelect = (index: string) => {
  activeMenu.value = index
}

const testConnection = async () => {
  try {
    ElMessage.info('正在测试连接...')
    const result = await configAPI.testLLMConnection()
    if (result) {
      ElMessage.success('LLM连接测试成功')
    } else {
      ElMessage.error('LLM连接测试失败')
    }
  } catch (error) {
    ElMessage.error(`连接测试失败: ${error}`)
  }
}

const saveSettings = () => {
  ElMessage.success('设置已保存')
}

const resetSettings = () => {
  ElMessage.info('设置已重置')
}
</script>

<style scoped>
.settings-container {
  height: 100vh;
  background: #f5f5f5;
}

.settings-menu {
  height: 100%;
  border-right: 1px solid #e6e6e6;
}

.settings-panel {
  padding: 20px;
  background: white;
  border-radius: 8px;
  margin-bottom: 20px;
}

.settings-panel h3 {
  margin: 0 0 20px 0;
  color: #303133;
}

.about-content {
  padding: 20px 0;
}

.about-content p {
  margin: 10px 0;
  color: #606266;
}

.settings-footer {
  padding: 20px;
  text-align: right;
  background: white;
  border-radius: 8px;
}
</style>