use crate::error::{AppError, AppResult};
use crate::ai::reasoning::ReasoningEngine;
use crate::types::*;
use serde::{Serialize, Deserialize};
use rand::{thread_rng, Rng};
use log::{info, debug};

/// 策略决策器
#[derive(Debug)]
pub struct StrategyEngine {
    personality: AIPersonality,
    game_knowledge: GameKnowledge,
    current_strategy: Strategy,
}

/// 游戏知识库
#[derive(Debug, Clone)]
pub struct GameKnowledge {
    pub known_roles: std::collections::HashMap<String, RoleType>,
    pub suspected_wolves: Vec<String>,
    pub trusted_players: Vec<String>,
    pub night_actions_history: Vec<NightActionRecord>,
    pub voting_patterns: std::collections::HashMap<String, Vec<String>>,
}

impl GameKnowledge {
    fn new() -> Self {
        Self {
            known_roles: std::collections::HashMap::new(),
            suspected_wolves: Vec::new(),
            trusted_players: Vec::new(),
            night_actions_history: Vec::new(),
            voting_patterns: std::collections::HashMap::new(),
        }
    }
}

/// 策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub strategy_type: StrategyType,
    pub priority_targets: Vec<String>,
    pub avoid_targets: Vec<String>,
    pub speech_style: SpeechStyle,
    pub voting_strategy: VotingStrategy,
    pub deception_level: f32, // 0.0-1.0
}

/// 发言风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechStyle {
    Concise,         // 简洁
    Detailed,        // 详细
    Emotional,       // 情绪化
    Analytical,      // 分析型
    Casual,          // 随意
}

/// 投票策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingStrategy {
    FollowMajority,  // 跟随大多数
    Independent,     // 独立思考
    Protective,      // 保护队友
    Aggressive,      // 积极投票
    Random,          // 随机投票
}

/// 发言策略
#[derive(Debug, Clone)]
pub struct SpeechStrategy {
    pub strategy_type: StrategyType,
    pub target_players: Vec<String>,
    pub key_points: Vec<String>,
}

impl StrategyEngine {
    /// 创建新的策略引擎
    pub fn new(personality: AIPersonality, role: &Role) -> Self {
        let strategy = Self::generate_initial_strategy(&personality, role);
        
        Self {
            personality,
            game_knowledge: GameKnowledge::new(),
            current_strategy: strategy,
        }
    }
    
    /// 生成初始策略
    fn generate_initial_strategy(personality: &AIPersonality, role: &Role) -> Strategy {
        let strategy_type = match role.faction {
            Faction::Werewolf => {
                if personality.traits.deception > 0.7 {
                    StrategyType::Deceptive
                } else if personality.traits.aggressiveness > 0.6 {
                    StrategyType::Aggressive
                } else {
                    StrategyType::Defensive
                }
            }
            Faction::Villager => {
                if personality.traits.logic > 0.7 {
                    StrategyType::Logical
                } else if personality.traits.aggressiveness > 0.6 {
                    StrategyType::Aggressive
                } else {
                    StrategyType::Neutral
                }
            }
        };
        
        let speech_style = if personality.traits.logic > 0.7 {
            SpeechStyle::Analytical
        } else if personality.traits.aggressiveness > 0.6 {
            SpeechStyle::Emotional
        } else {
            SpeechStyle::Concise
        };
        
        let voting_strategy = match strategy_type {
            StrategyType::Aggressive => VotingStrategy::Aggressive,
            StrategyType::Defensive => VotingStrategy::Protective,
            StrategyType::Logical => VotingStrategy::Independent,
            StrategyType::Deceptive => VotingStrategy::FollowMajority,
            _ => VotingStrategy::Independent,
        };
        
        Strategy {
            strategy_type,
            priority_targets: Vec::new(),
            avoid_targets: Vec::new(),
            speech_style,
            voting_strategy,
            deception_level: personality.traits.deception,
        }
    }
    
    /// 决定夜晚行动
    pub fn decide_night_action(
        &mut self,
        my_role: &Role,
        game_state: &GameState,
        _reasoning: &ReasoningEngine
    ) -> AppResult<Option<NightAction>> {
        match my_role.role_type {
            RoleType::Werewolf => self.decide_werewolf_kill(game_state),
            RoleType::Seer => self.decide_seer_check(game_state),
            RoleType::Witch => self.decide_witch_action(game_state),
            RoleType::Guard => self.decide_guard_protect(game_state),
            _ => Ok(None),
        }
    }
    
    /// 决定狼人击杀目标
    fn decide_werewolf_kill(&mut self, game_state: &GameState) -> AppResult<Option<NightAction>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive && p.faction == Faction::Villager)
            .collect();
        
        if alive_players.is_empty() {
            return Ok(None);
        }
        
        let mut rng = thread_rng();
        let target = &alive_players[rng.gen_range(0..alive_players.len())];
        
        Ok(Some(NightAction {
            player: "werewolf".to_string(),
            action: NightActionType::Kill,
            target: Some(target.id.clone()),
        }))
    }
    
    /// 决定预言家查验目标
    fn decide_seer_check(&mut self, game_state: &GameState) -> AppResult<Option<NightAction>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive)
            .collect();
        
        if alive_players.is_empty() {
            return Ok(None);
        }
        
        let mut rng = thread_rng();
        let target = &alive_players[rng.gen_range(0..alive_players.len())];
        
        Ok(Some(NightAction {
            player: "seer".to_string(),
            action: NightActionType::Check,
            target: Some(target.id.clone()),
        }))
    }
    
    /// 决定女巫行动
    fn decide_witch_action(&mut self, _game_state: &GameState) -> AppResult<Option<NightAction>> {
        // 简化处理，不做任何行动
        Ok(None)
    }
    
    /// 决定守卫保护目标
    fn decide_guard_protect(&mut self, game_state: &GameState) -> AppResult<Option<NightAction>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive)
            .collect();
        
        if alive_players.is_empty() {
            return Ok(None);
        }
        
        let mut rng = thread_rng();
        let target = &alive_players[rng.gen_range(0..alive_players.len())];
        
        Ok(Some(NightAction {
            player: "guard".to_string(),
            action: NightActionType::Protect,
            target: Some(target.id.clone()),
        }))
    }
    
    /// 更新策略
    pub fn update_strategy(&mut self, _game_state: &GameState, _reasoning: &ReasoningEngine) {
        debug!("更新AI策略");
    }
    
    /// 生成发言策略
    pub fn generate_speech_strategy(
        &self,
        _game_state: &GameState,
        _reasoning: &ReasoningEngine,
        speech_type: SpeechType
    ) -> SpeechStrategy {
        let key_points = match speech_type {
            SpeechType::Accusation => vec!["指控某人".to_string()],
            SpeechType::Defense => vec!["为自己辩护".to_string()],
            SpeechType::Information => vec!["分享信息".to_string()],
            _ => vec!["一般发言".to_string()],
        };
        
        SpeechStrategy {
            strategy_type: self.current_strategy.strategy_type.clone(),
            target_players: vec![],
            key_points,
        }
    }
    
    /// 决定投票目标
    pub async fn decide_vote_target(
        &self,
        game_state: &GameState,
        _reasoning: &ReasoningEngine
    ) -> AppResult<Option<String>> {
        let alive_others: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive && !p.is_ai)
            .collect();
            
        if !alive_others.is_empty() {
            let mut rng = thread_rng();
            let target = &alive_others[rng.gen_range(0..alive_others.len())];
            Ok(Some(target.id.clone()))
        } else {
            Ok(None)
        }
    }
}