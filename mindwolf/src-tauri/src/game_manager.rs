use crate::error::{AppError, AppResult};
use crate::types::*;
use crate::game_engine::GameEngine;
use crate::llm::LLMManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};

/// 游戏管理器
pub struct GameManager {
    engine: Option<GameEngine>,
    llm_manager: Option<Arc<LLMManager>>,
    is_running: bool,
}

impl GameManager {
    /// 创建新的游戏管理器
    pub fn new() -> Self {
        Self {
            engine: None,
            llm_manager: None,
            is_running: false,
        }
    }
    
    /// 设置LLM管理器
    pub fn set_llm_manager(&mut self, llm_manager: Arc<LLMManager>) {
        self.llm_manager = Some(llm_manager);
    }
    
    /// 创建新游戏
    pub async fn create_game(&mut self, config: GameConfig) -> AppResult<GameState> {
        info!(\"创建新游戏\");
        
        let mut engine = GameEngine::new(config)?;
        engine.initialize_game()?;
        
        let state = engine.get_state().clone();
        self.engine = Some(engine);
        self.is_running = false;
        
        Ok(state)
    }
    
    /// 开始游戏
    pub async fn start_game(&mut self) -> AppResult<()> {
        if let Some(engine) = &mut self.engine {
            engine.start_game()?;
            self.is_running = true;
            info!(\"游戏已开始\");
            Ok(())
        } else {
            Err(AppError::GameLogic(\"游戏未创建\".to_string()))
        }
    }
    
    /// 结束游戏
    pub async fn end_game(&mut self) -> AppResult<()> {
        self.engine = None;
        self.is_running = false;
        info!(\"游戏已结束\");
        Ok(())
    }
    
    /// 获取游戏状态
    pub fn get_game_state(&self) -> Option<GameState> {
        self.engine.as_ref().map(|e| e.get_state().clone())
    }
    
    /// 玩家投票
    pub async fn player_vote(&mut self, voter_id: String, target_id: String) -> AppResult<()> {
        if let Some(engine) = &mut self.engine {
            engine.vote(voter_id, target_id)?;
            
            // 检查是否所有存活玩家都已投票
            if self.all_players_voted() {
                self.proceed_to_next_phase().await?;
            }
            
            Ok(())
        } else {
            Err(AppError::GameLogic(\"游戏未开始\".to_string()))
        }
    }
    
    /// 检查所有玩家是否都已投票
    fn all_players_voted(&self) -> bool {
        if let Some(engine) = &self.engine {
            let state = engine.get_state();
            let alive_players = state.players.iter().filter(|p| p.is_alive).count();
            state.votes.len() >= alive_players
        } else {
            false
        }
    }
    
    /// 进入下一阶段
    pub async fn proceed_to_next_phase(&mut self) -> AppResult<()> {
        if let Some(engine) = &mut self.engine {
            engine.next_phase()?;
            
            // 如果进入新的夜晚，执行AI夜晚行动
            if engine.get_state().phase == GamePhase::Night {
                self.execute_night_actions().await?;
            }
            
            Ok(())
        } else {
            Err(AppError::GameLogic(\"游戏未开始\".to_string()))
        }
    }
    
    /// 执行夜晚行动
    async fn execute_night_actions(&mut self) -> AppResult<()> {
        if let Some(engine) = &mut self.engine {
            let state = engine.get_state();
            
            // 获取所有存活的AI玩家
            let ai_players: Vec<_> = state.players.iter()
                .filter(|p| p.is_alive && p.is_ai && p.role.has_night_action)
                .cloned()
                .collect();
            
            // 为每个AI生成夜晚行动
            for player in ai_players {
                if let Some(action) = self.generate_ai_night_action(&player).await? {
                    engine.execute_night_action(action)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 生成AI夜晚行动
    async fn generate_ai_night_action(&self, player: &Player) -> AppResult<Option<NightAction>> {
        if let Some(llm_manager) = &self.llm_manager {
            let prompt = self.build_night_action_prompt(player)?;
            
            match llm_manager.generate_with_fallback(prompt).await {
                Ok(response) => {
                    // TODO: 解析LLM响应生成夜晚行动
                    // 这里简化处理，实际应该解析JSON响应
                    self.parse_night_action_response(player, &response)
                }
                Err(e) => {
                    warn!(\"AI夜晚行动生成失败: {}\", e);
                    Ok(None)
                }
            }
        } else {
            // 如果没有LLM，使用简单的随机逻辑
            Ok(self.generate_simple_night_action(player))
        }
    }
    
    /// 构建夜晚行动提示词
    fn build_night_action_prompt(&self, player: &Player) -> AppResult<String> {
        if let Some(engine) = &self.engine {
            let state = engine.get_state();
            
            let prompt = match player.role.role_type {
                RoleType::Werewolf => {
                    format!(
                        \"你是狼人{}，现在是第{}夜。存活的玩家有：{}。请选择一个目标杀死。返回JSON格式：{{\\\"action\\\":\\\"kill\\\",\\\"target\\\":\\\"player_id\\\"}}\",
                        player.name,
                        state.day,
                        self.format_alive_players(state)
                    )
                }
                RoleType::Seer => {
                    format!(
                        \"你是预言家{}，现在是第{}夜。存活的玩家有：{}。请选择一个目标查验。返回JSON格式：{{\\\"action\\\":\\\"check\\\",\\\"target\\\":\\\"player_id\\\"}}\",
                        player.name,
                        state.day,
                        self.format_alive_players(state)
                    )
                }
                RoleType::Witch => {
                    format!(
                        \"你是女巫{}，现在是第{}夜。你可以选择救人或毒人。返回JSON格式：{{\\\"action\\\":\\\"heal/poison\\\",\\\"target\\\":\\\"player_id\\\"}}\",
                        player.name,
                        state.day
                    )
                }
                RoleType::Guard => {
                    format!(
                        \"你是守卫{}，现在是第{}夜。存活的玩家有：{}。请选择一个目标保护。返回JSON格式：{{\\\"action\\\":\\\"protect\\\",\\\"target\\\":\\\"player_id\\\"}}\",
                        player.name,
                        state.day,
                        self.format_alive_players(state)
                    )
                }
                _ => return Err(AppError::GameLogic(\"无效的夜晚行动角色\".to_string())),
            };
            
            Ok(prompt)
        } else {
            Err(AppError::GameLogic(\"游戏引擎未初始化\".to_string()))
        }
    }
    
    /// 格式化存活玩家列表
    fn format_alive_players(&self, state: &GameState) -> String {
        state.players.iter()
            .filter(|p| p.is_alive)
            .map(|p| format!(\"{}({})\", p.name, p.id))
            .collect::<Vec<_>>()
            .join(\", \")
    }
    
    /// 解析夜晚行动响应
    fn parse_night_action_response(&self, player: &Player, response: &str) -> AppResult<Option<NightAction>> {
        // 简化的JSON解析
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            if let (Some(action_str), target) = (
                json_value.get(\"action\").and_then(|v| v.as_str()),
                json_value.get(\"target\").and_then(|v| v.as_str())
            ) {
                let action_type = match action_str {
                    \"kill\" => NightActionType::Kill,
                    \"check\" => NightActionType::Check,
                    \"heal\" => NightActionType::Heal,
                    \"protect\" => NightActionType::Protect,
                    \"poison\" => NightActionType::Poison,
                    _ => return Ok(None),
                };
                
                return Ok(Some(NightAction {
                    player: player.id.clone(),
                    action: action_type,
                    target: target.map(|s| s.to_string()),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// 生成简单的夜晚行动（备用逻辑）
    fn generate_simple_night_action(&self, player: &Player) -> Option<NightAction> {
        if let Some(engine) = &self.engine {
            let state = engine.get_state();
            let alive_players: Vec<_> = state.players.iter()
                .filter(|p| p.is_alive && p.id != player.id)
                .collect();
            
            if !alive_players.is_empty() {
                use rand::{thread_rng, Rng};
                let mut rng = thread_rng();
                let target = &alive_players[rng.gen_range(0..alive_players.len())];
                
                let action_type = match player.role.role_type {
                    RoleType::Werewolf => NightActionType::Kill,
                    RoleType::Seer => NightActionType::Check,
                    RoleType::Guard => NightActionType::Protect,
                    _ => return None,
                };
                
                return Some(NightAction {
                    player: player.id.clone(),
                    action: action_type,
                    target: Some(target.id.clone()),
                });
            }
        }
        
        None
    }
    
    /// 处理玩家发言
    pub async fn handle_player_speech(&mut self, player_id: String, content: String) -> AppResult<()> {
        if let Some(engine) = &mut self.engine {
            let message = ChatMessage {
                id: crate::utils::generate_id(),
                sender: player_id,
                content,
                timestamp: chrono::Utc::now(),
                message_type: MessageType::Human,
            };
            
            engine.add_chat_message(message)?;
        }
        
        Ok(())
    }
    
    /// 生成AI发言
    pub async fn generate_ai_speech(&mut self, player_id: String) -> AppResult<String> {
        if let Some(llm_manager) = &self.llm_manager {
            if let Some(engine) = &self.engine {
                let state = engine.get_state();
                
                if let Some(player) = state.players.iter().find(|p| p.id == player_id) {
                    let prompt = self.build_speech_prompt(player, state)?;
                    
                    match llm_manager.generate_with_fallback(prompt).await {
                        Ok(response) => {
                            // 记录AI发言
                            let message = ChatMessage {
                                id: crate::utils::generate_id(),
                                sender: player_id,
                                content: response.clone(),
                                timestamp: chrono::Utc::now(),
                                message_type: MessageType::AI,
                            };
                            
                            if let Some(engine) = &mut self.engine {
                                engine.add_chat_message(message)?;
                            }
                            
                            Ok(response)
                        }
                        Err(e) => {
                            warn!(\"AI发言生成失败: {}\", e);
                            Ok(\"我需要思考一下...\".to_string())
                        }
                    }
                } else {
                    Err(AppError::GameLogic(\"玩家不存在\".to_string()))
                }
            } else {
                Err(AppError::GameLogic(\"游戏未开始\".to_string()))
            }
        } else {
            Ok(\"AI系统未配置\".to_string())
        }
    }
    
    /// 构建发言提示词
    fn build_speech_prompt(&self, player: &Player, state: &GameState) -> AppResult<String> {
        let phase_desc = match state.phase {
            GamePhase::DayDiscussion => \"白天讨论\",
            GamePhase::Voting => \"投票阶段\",
            _ => \"其他阶段\",
        };
        
        let prompt = format!(
            \"你是{}，身份是{}，属于{}阵营。现在是第{}天的{}阶段。场上存活玩家：{}。请生成一段符合你身份和性格的发言，长度在50-200字之间。\",
            player.name,
            utils::get_role_description(&player.role.role_type),
            utils::get_faction_description(&player.faction),
            state.day,
            phase_desc,
            self.format_alive_players(state)
        );
        
        Ok(prompt)
    }
    
    /// 更新游戏计时器
    pub async fn update_timer(&mut self) -> AppResult<bool> {
        if let Some(engine) = &mut self.engine {
            engine.update_timer()
        } else {
            Ok(false)
        }
    }
    
    /// 检查游戏是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}