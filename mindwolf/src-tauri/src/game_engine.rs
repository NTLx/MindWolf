use crate::error::{AppError, AppResult};
use crate::types::*;
use crate::utils;
use std::collections::HashMap;
use chrono::Utc;
use log::{info, warn, error};
use rand::{thread_rng, Rng};

/// 游戏引擎
pub struct GameEngine {
    state: GameState,
    players_map: HashMap<String, usize>, // player_id -> players index
    timer: Option<tokio::time::Instant>,
}

impl GameEngine {
    /// 创建新游戏
    pub fn new(config: GameConfig) -> AppResult<Self> {
        let state = GameState {
            phase: GamePhase::Preparation,
            day: 0,
            players: Vec::new(),
            dead_players: Vec::new(),
            votes: Vec::new(),
            game_config: config,
            winner: None,
            current_speaker: None,
            time_remaining: None,
        };
        
        Ok(Self {
            state,
            players_map: HashMap::new(),
            timer: None,
        })
    }
    
    /// 初始化游戏
    pub fn initialize_game(&mut self) -> AppResult<()> {
        info!(\"初始化游戏，玩家数: {}\", self.state.game_config.total_players);
        
        // 生成角色分配
        let role_distribution = utils::generate_role_distribution(self.state.game_config.total_players);
        self.state.game_config.role_distribution = role_distribution.clone();
        
        // 创建角色列表
        let mut roles = Vec::new();
        for (role_type, count) in role_distribution {
            for _ in 0..count {
                roles.push(self.create_role(role_type.clone()));
            }
        }
        
        // 洗牌
        utils::shuffle(&mut roles);
        
        // 创建玩家
        let mut players = Vec::new();
        
        // 添加人类玩家（第一个玩家）
        if let Some(role) = roles.pop() {
            let human_player = Player {
                id: \"human_player\".to_string(),
                name: \"玩家\".to_string(),
                role,
                faction: Faction::Villager, // 将在角色分配后更新
                is_alive: true,
                is_ai: false,
                personality: None,
            };
            players.push(human_player);
        }
        
        // 添加AI玩家
        for (i, role) in roles.into_iter().enumerate() {
            let ai_player = Player {
                id: format!(\"ai_{}\", i + 1),
                name: utils::generate_ai_name(),
                role: role.clone(),
                faction: role.faction.clone(),
                is_alive: true,
                is_ai: true,
                personality: Some(self.generate_ai_personality()),
            };
            players.push(ai_player);
        }
        
        // 更新人类玩家的阵营
        if let Some(human_player) = players.first_mut() {
            human_player.faction = human_player.role.faction.clone();
        }
        
        // 建立玩家映射
        for (index, player) in players.iter().enumerate() {
            self.players_map.insert(player.id.clone(), index);
        }
        
        self.state.players = players;
        
        info!(\"游戏初始化完成，共 {} 名玩家\", self.state.players.len());
        Ok(())
    }
    
    /// 创建角色
    fn create_role(&self, role_type: RoleType) -> Role {
        let faction = match role_type {
            RoleType::Werewolf => Faction::Werewolf,
            _ => Faction::Villager,
        };
        
        let (can_vote, has_night_action) = match role_type {
            RoleType::Werewolf => (true, true),
            RoleType::Villager => (true, false),
            RoleType::Seer => (true, true),
            RoleType::Witch => (true, true),
            RoleType::Hunter => (true, false),
            RoleType::Guard => (true, true),
        };
        
        Role {
            role_type: role_type.clone(),
            faction,
            description: utils::get_role_description(&role_type),
            can_vote,
            has_night_action,
        }
    }
    
    /// 生成AI性格
    fn generate_ai_personality(&self) -> AIPersonality {
        let mut rng = thread_rng();
        
        AIPersonality {
            id: utils::generate_id(),
            name: \"标准AI\".to_string(),
            description: \"平衡型AI，具备适中的各项能力\".to_string(),
            traits: PersonalityTraits {
                aggressiveness: rng.gen_range(0.3..0.8),
                logic: rng.gen_range(0.5..0.9),
                deception: rng.gen_range(0.4..0.7),
                trustfulness: rng.gen_range(0.3..0.7),
            },
        }
    }
    
    /// 开始游戏
    pub fn start_game(&mut self) -> AppResult<()> {
        if self.state.players.is_empty() {
            return Err(AppError::GameLogic(\"没有玩家，无法开始游戏\".to_string()));
        }
        
        self.state.phase = GamePhase::Night;
        self.state.day = 1;
        
        info!(\"游戏开始！第1夜\");
        self.start_phase_timer()
    }
    
    /// 进入下一阶段
    pub fn next_phase(&mut self) -> AppResult<()> {
        match self.state.phase {
            GamePhase::Preparation => {
                self.start_game()?;
            }
            GamePhase::Night => {
                self.state.phase = GamePhase::DayDiscussion;
                info!(\"进入白天讨论阶段\");
                self.start_phase_timer()?;
            }
            GamePhase::DayDiscussion => {
                self.state.phase = GamePhase::Voting;
                info!(\"进入投票阶段\");
                self.start_phase_timer()?;
            }
            GamePhase::Voting => {
                self.process_votes()?;
                if self.check_game_end()? {
                    self.state.phase = GamePhase::GameOver;
                } else {
                    self.state.phase = GamePhase::Night;
                    self.state.day += 1;
                    info!(\"进入第{}夜\", self.state.day);
                }
            }
            GamePhase::LastWords => {
                self.state.phase = GamePhase::Night;
                self.state.day += 1;
                info!(\"进入第{}夜\", self.state.day);
            }
            GamePhase::GameOver => {
                info!(\"游戏已结束\");
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// 开始阶段计时器
    fn start_phase_timer(&mut self) -> AppResult<()> {
        let duration = match self.state.phase {
            GamePhase::DayDiscussion => self.state.game_config.discussion_time,
            GamePhase::Voting => self.state.game_config.voting_time,
            _ => 0,
        };
        
        if duration > 0 {
            self.state.time_remaining = Some(duration);
            self.timer = Some(tokio::time::Instant::now());
        }
        
        Ok(())
    }
    
    /// 处理投票
    fn process_votes(&mut self) -> AppResult<()> {
        let mut vote_counts: HashMap<String, u32> = HashMap::new();
        
        // 统计票数
        for vote in &self.state.votes {
            *vote_counts.entry(vote.target.clone()).or_insert(0) += 1;
        }
        
        // 找出得票最多的玩家
        if let Some((eliminated_player_id, _)) = vote_counts.iter().max_by_key(|(_, &count)| count) {
            self.eliminate_player(eliminated_player_id.clone())?;
        }
        
        // 清空投票记录
        self.state.votes.clear();
        
        Ok(())
    }
    
    /// 淘汰玩家
    fn eliminate_player(&mut self, player_id: String) -> AppResult<()> {
        if let Some(&index) = self.players_map.get(&player_id) {
            if index < self.state.players.len() {
                let mut player = self.state.players.remove(index);
                player.is_alive = false;
                self.state.dead_players.push(player.clone());
                
                // 更新玩家映射
                self.players_map.remove(&player_id);
                for (id, idx) in self.players_map.iter_mut() {
                    if *idx > index {
                        *idx -= 1;
                    }
                }
                
                info!(\"玩家 {} 被淘汰\", player.name);
                
                // 检查猎人技能
                if player.role.role_type == RoleType::Hunter {
                    // TODO: 实现猎人技能
                    info!(\"猎人 {} 可以开枪带走一名玩家\", player.name);
                }
            }
        }
        
        Ok(())
    }
    
    /// 检查游戏是否结束
    fn check_game_end(&mut self) -> AppResult<bool> {
        let alive_werewolves = self.state.players.iter()
            .filter(|p| p.is_alive && p.role.faction == Faction::Werewolf)
            .count();
            
        let alive_villagers = self.state.players.iter()
            .filter(|p| p.is_alive && p.role.faction == Faction::Villager)
            .count();
        
        if let Some(winner) = utils::check_win_condition(alive_werewolves, alive_villagers) {
            self.state.winner = Some(winner.clone());
            self.state.phase = GamePhase::GameOver;
            
            info!(\"游戏结束！获胜方: {:?}\", winner);
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// 投票
    pub fn vote(&mut self, voter_id: String, target_id: String) -> AppResult<()> {
        if self.state.phase != GamePhase::Voting {
            return Err(AppError::GameLogic(\"当前不是投票阶段\".to_string()));
        }
        
        // 检查投票者是否存在且存活
        if !self.is_player_alive(&voter_id) {
            return Err(AppError::GameLogic(\"投票者不存在或已死亡\".to_string()));
        }
        
        // 检查目标是否存在且存活
        if !self.is_player_alive(&target_id) {
            return Err(AppError::GameLogic(\"投票目标不存在或已死亡\".to_string()));
        }
        
        // 移除之前的投票（如果有）
        self.state.votes.retain(|v| v.voter != voter_id);
        
        // 添加新投票
        let vote = VoteRecord {
            voter: voter_id,
            target: target_id,
            timestamp: Utc::now(),
        };
        
        self.state.votes.push(vote);
        
        Ok(())
    }
    
    /// 检查玩家是否存活
    fn is_player_alive(&self, player_id: &str) -> bool {
        self.state.players.iter().any(|p| p.id == player_id && p.is_alive)
    }
    
    /// 获取游戏状态
    pub fn get_state(&self) -> &GameState {
        &self.state
    }
    
    /// 获取可变游戏状态
    pub fn get_state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }
    
    /// 更新计时器
    pub fn update_timer(&mut self) -> AppResult<bool> {
        if let (Some(timer), Some(time_remaining)) = (self.timer, self.state.time_remaining) {
            let elapsed = timer.elapsed().as_secs() as u32;
            
            if elapsed >= time_remaining {
                self.state.time_remaining = None;
                self.timer = None;
                info!(\"阶段时间已到\");
                return Ok(true); // 时间到了
            } else {
                self.state.time_remaining = Some(time_remaining - elapsed);
            }
        }
        
        Ok(false)
    }
    
    /// 添加聊天消息
    pub fn add_chat_message(&mut self, message: ChatMessage) -> AppResult<()> {
        // TODO: 存储聊天消息到某个地方
        info!(\"聊天消息: {} - {}\", message.sender, message.content);
        Ok(())
    }
    
    /// 执行夜晚行动
    pub fn execute_night_action(&mut self, action: NightAction) -> AppResult<()> {
        match action.action {
            NightActionType::Kill => {
                if let Some(target_id) = action.target {
                    self.eliminate_player(target_id)?;
                }
            }
            NightActionType::Check => {
                // TODO: 实现预言家查验
                info!(\"预言家查验: {:?}\", action.target);
            }
            NightActionType::Heal => {
                // TODO: 实现女巫救人
                info!(\"女巫救人: {:?}\", action.target);
            }
            NightActionType::Protect => {
                // TODO: 实现守卫保护
                info!(\"守卫保护: {:?}\", action.target);
            }
            NightActionType::Poison => {
                // TODO: 实现女巫毒人
                info!(\"女巫毒人: {:?}\", action.target);
            }
        }
        
        Ok(())
    }
}