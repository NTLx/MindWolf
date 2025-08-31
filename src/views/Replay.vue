<template>
  <div class="replay-container">
    <div class="replay-header">
      <h2>游戏复盘</h2>
      <el-button @click="$router.back()">返回</el-button>
    </div>

    <div class="replay-list">
      <el-table :data="replays" stripe>
        <el-table-column prop="game_id" label="游戏ID" width="200" />
        <el-table-column prop="start_time" label="开始时间" width="200">
          <template #default="{ row }">
            {{ formatDateTime(row.start_time) }}
          </template>
        </el-table-column>
        <el-table-column prop="winner" label="获胜方" width="120" />
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button size="small" @click="viewReplay(row.game_id)">
              查看
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const replays = ref([])

const formatDateTime = (dateTime: string) => {
  return new Date(dateTime).toLocaleString('zh-CN')
}

const viewReplay = (gameId: string) => {
  router.push(`/replay/${gameId}`)
}

onMounted(() => {
  // 加载复盘列表
  replays.value = []
})
</script>

<style scoped>
.replay-container {
  padding: 20px;
  max-width: 1000px;
  margin: 0 auto;
}

.replay-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.replay-header h2 {
  margin: 0;
}

.replay-list {
  background: white;
  border-radius: 8px;
  padding: 20px;
}
</style>