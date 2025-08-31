use crate::types::*;
use crate::error::{AppError, AppResult};
use crate::ai::reasoning::ReasoningEngine;
use serde::{Serialize, Deserialize};
use rand::{thread_rng, Rng};
use log::{info, debug};

/// 策略决策器
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

/// 夜晚行动记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightActionRecord {
    pub night: u32,
    pub action_type: NightActionType,
    pub actor: Option<String>,
    pub target: Option<String>,
    pub result: ActionResult,
}

/// 行动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success,
    Failed,
    Blocked,
    Unknown,
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

/// 策略类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Aggressive,      // 攻击型
    Defensive,       // 防守型
    Neutral,         // 中立型
    Deceptive,       // 欺骗型
    Logical,         // 逻辑型
    Chaotic,         // 混乱型
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
        let mut rng = thread_rng();
        
        let strategy_type = match role.faction {
            Faction::Werewolf => {
                // 狼人策略偏向欺骗和攻击
                if personality.traits.deception > 0.7 {
                    StrategyType::Deceptive
                } else if personality.traits.aggressiveness > 0.6 {
                    StrategyType::Aggressive
                } else {
                    StrategyType::Defensive
                }
            }
            Faction::Villager => {
                // 好人策略偏向逻辑和中立
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
        reasoning: &ReasoningEngine
    ) -> AppResult<Option<NightAction>> {
        match my_role.role_type {
            RoleType::Werewolf => self.decide_werewolf_kill(game_state, reasoning),
            RoleType::Seer => self.decide_seer_check(game_state, reasoning),
            RoleType::Witch => self.decide_witch_action(game_state, reasoning),
            RoleType::Guard => self.decide_guard_protect(game_state, reasoning),
            _ => Ok(None),
        }
    }
    
    /// 决定狼人击杀目标
    fn decide_werewolf_kill(
        &mut self,
        game_state: &GameState,
        reasoning: &ReasoningEngine
    ) -> AppResult<Option<NightAction>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive && p.faction == Faction::Villager)
            .collect();
        
        if alive_players.is_empty() {
            return Ok(None);
        }
        
        // 根据策略选择目标
        let target = match self.current_strategy.strategy_type {
            StrategyType::Aggressive => {
                // 攻击型：优先击杀最信任的玩家（可能是神职）
                reasoning.get_most_trusted_player()
                    .or_else(|| self.select_random_target(&alive_players))
            }
            StrategyType::Deceptive => {
                // 欺骗型：击杀中等可疑度的玩家，制造混乱
                self.select_medium_suspicion_target(&alive_players, reasoning)
            }
            _ => {
                // 默认：随机选择或基于简单逻辑
                self.select_strategic_target(&alive_players, reasoning)
            }
        };
        
        if let Some(target_id) = target {
            debug!(\"狼人决定击杀: {}\", target_id);
            Ok(Some(NightAction {
                player: \"self\".to_string(),
                action: NightActionType::Kill,
                target: Some(target_id),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 决定预言家查验目标
    fn decide_seer_check(
        &mut self,
        game_state: &GameState,
        reasoning: &ReasoningEngine
    ) -> AppResult<Option<NightAction>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive && p.id != \"self\")
            .collect();
        
        if alive_players.is_empty() {
            return Ok(None);
        }
        
        // 优先查验最可疑的玩家
        let target = reasoning.get_most_suspicious_player()
            .or_else(|| self.select_random_target(&alive_players));
        
        if let Some(target_id) = target {
            debug!(\"预言家决定查验: {}\", target_id);
            Ok(Some(NightAction {
                player: \"self\".to_string(),
                action: NightActionType::Check,
                target: Some(target_id),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 决定女巫行动
    fn decide_witch_action(
        &mut self,
        game_state: &GameState,
        reasoning: &ReasoningEngine
    ) -> AppResult<Option<NightAction>> {
        // 简化的女巫逻辑：优先救人，其次毒人
        let mut rng = thread_rng();
        
        if rng.gen_bool(0.7) {
            // 70%概率选择救人
            Ok(Some(NightAction {
                player: \"self\".to_string(),
                action: NightActionType::Heal,
                target: None, // 具体目标需要根据当晚的击杀情况决定
            }))
        } else {
            // 30%概率选择毒人
            let target = reasoning.get_most_suspicious_player();
            Ok(Some(NightAction {
                player: \"self\".to_string(),
                action: NightActionType::Poison,
                target,
            }))
        }
    }
    
    /// 决定守卫保护目标
    fn decide_guard_protect(
        &mut self,
        game_state: &GameState,
        reasoning: &ReasoningEngine
    ) -> AppResult<Option<NightAction>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive && p.id != \"self\")
            .collect();
        
        // 优先保护最信任的玩家
        let target = reasoning.get_most_trusted_player()
            .or_else(|| self.select_random_target(&alive_players));
        
        if let Some(target_id) = target {
            debug!(\"守卫决定保护: {}\", target_id);
            Ok(Some(NightAction {
                player: \"self\".to_string(),
                action: NightActionType::Protect,
                target: Some(target_id),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 决定投票目标
    pub fn decide_vote_target(
        &mut self,
        game_state: &GameState,
        reasoning: &ReasoningEngine
    ) -> AppResult<Option<String>> {
        let alive_players: Vec<_> = game_state.players.iter()
            .filter(|p| p.is_alive && p.id != \"self\")
            .collect();
        
        if alive_players.is_empty() {
            return Ok(None);
        }
        
        match self.current_strategy.voting_strategy {
            VotingStrategy::Aggressive => {
                // 积极投票给最可疑的玩家
                Ok(reasoning.get_most_suspicious_player())
            }
            VotingStrategy::Independent => {
                // 独立思考，基于推理结果
                Ok(reasoning.get_most_suspicious_player())
            }
            VotingStrategy::FollowMajority => {
                // 跟随大多数（简化实现）
                Ok(self.select_random_target(&alive_players))
            }
            VotingStrategy::Protective => {
                // 保护策略：避免投票给信任的玩家
                let trusted = reasoning.get_most_trusted_player();
                let suspicious = reasoning.get_most_suspicious_player();
                
                if let Some(suspicious_id) = suspicious {
                    if Some(&suspicious_id) != trusted.as_ref() {
                        Ok(Some(suspicious_id))
                    } else {
                        Ok(self.select_random_target(&alive_players))
                    }
                } else {
                    Ok(self.select_random_target(&alive_players))
                }
            }
            VotingStrategy::Random => {
                Ok(self.select_random_target(&alive_players))
            }
        }
    }
    
    /// 生成发言策略
    pub fn generate_speech_strategy(
        &self,
        game_state: &GameState,
        reasoning: &ReasoningEngine,
        speech_type: SpeechType
    ) -> SpeechStrategy {
        let most_suspicious = reasoning.get_most_suspicious_player();
        let most_trusted = reasoning.get_most_trusted_player();
        
        match speech_type {
            SpeechType::Accusation => {
                SpeechStrategy {
                    target: most_suspicious.clone(),
                    tone: self.get_speech_tone(),
                    key_points: vec![
                        \"指出可疑行为\".to_string(),
                        \"分析投票模式\".to_string(),
                        \"提供逻辑推理\".to_string(),
                    ],
                    confidence_level: self.personality.traits.aggressiveness,
                    deception_elements: if self.current_strategy.deception_level > 0.5 {
                        vec![\"混淆视听\".to_string(), \"转移注意力\".to_string()]
                    } else {
                        vec![]
                    },
                }
            }
            SpeechType::Defense => {
                SpeechStrategy {
                    target: None,
                    tone: SpeechTone::Defensive,
                    key_points: vec![
                        \"澄清误解\".to_string(),
                        \"提供证据\".to_string(),
                        \"反驳指控\".to_string(),
                    ],
                    confidence_level: 0.8,
                    deception_elements: vec![],
                }
            }
            SpeechType::Information => {
                SpeechStrategy {
                    target: most_trusted.clone(),
                    tone: SpeechTone::Analytical,
                    key_points: vec![
                        \"分享观察\".to_string(),
                        \"提供信息\".to_string(),
                        \"建议策略\".to_string(),
                    ],
                    confidence_level: self.personality.traits.logic,
                    deception_elements: vec![],
                }
            }
            _ => {
                SpeechStrategy {
                    target: None,
                    tone: SpeechTone::Neutral,
                    key_points: vec![\"一般发言\".to_string()],
                    confidence_level: 0.5,
                    deception_elements: vec![],
                }
            }
        }
    }
    
    /// 获取发言语调
    fn get_speech_tone(&self) -> SpeechTone {
        match self.current_strategy.speech_style {
            SpeechStyle::Analytical => SpeechTone::Analytical,
            SpeechStyle::Emotional => SpeechTone::Aggressive,
            SpeechStyle::Casual => SpeechTone::Neutral,
            _ => SpeechTone::Confident,
        }
    }
    
    /// 选择随机目标
    fn select_random_target(&self, players: &[&Player]) -> Option<String> {
        if players.is_empty() {
            return None;
        }
        
        let mut rng = thread_rng();
        let index = rng.gen_range(0..players.len());
        Some(players[index].id.clone())
    }
    
    /// 选择中等可疑度目标
    fn select_medium_suspicion_target(
        &self,
        players: &[&Player],
        reasoning: &ReasoningEngine
    ) -> Option<String> {
        // 选择可疑度在0.3-0.7之间的玩家
        let candidates: Vec<_> = players.iter()
            .filter(|p| {
                let suspicion = reasoning.get_werewolf_probability(&p.id);
                suspicion >= 0.3 && suspicion <= 0.7
            })
            .collect();
        
        if !candidates.is_empty() {
            self.select_random_target(&candidates.iter().map(|&p| *p).collect::<Vec<_>>())
        } else {
            self.select_random_target(players)
        }
    }
    
    /// 选择策略性目标
    fn select_strategic_target(
        &self,
        players: &[&Player],
        reasoning: &ReasoningEngine
    ) -> Option<String> {
        // 综合考虑可疑度和威胁程度
        let mut best_target = None;
        let mut best_score = 0.0;
        
        for player in players {
            let suspicion = reasoning.get_werewolf_probability(&player.id);
            let threat_level = self.calculate_threat_level(player);
            let score = (1.0 - suspicion) * threat_level; // 低可疑度但高威胁的玩家
            
            if score > best_score {
                best_score = score;
                best_target = Some(player.id.clone());
            }
        }
        
        best_target.or_else(|| self.select_random_target(players))
    }
    
    /// 计算威胁程度
    fn calculate_threat_level(&self, player: &Player) -> f32 {
        match player.role.role_type {
            RoleType::Seer => 0.9,      // 预言家威胁最大
            RoleType::Witch => 0.8,     // 女巫次之
            RoleType::Hunter => 0.7,    // 猎人有反击能力
            RoleType::Guard => 0.6,     // 守卫有保护能力
            RoleType::Villager => 0.3,  // 普通村民威胁较小
            RoleType::Werewolf => 0.0,  // 同伴不攻击
        }
    }
    
    /// 更新策略
    pub fn update_strategy(&mut self, game_state: &GameState, reasoning: &ReasoningEngine) {
        // 根据游戏进展和推理结果调整策略
        if game_state.day > 3 {
            // 后期更加激进
            self.current_strategy.deception_level += 0.1;
            if matches!(self.current_strategy.strategy_type, StrategyType::Defensive) {
                self.current_strategy.strategy_type = StrategyType::Aggressive;
            }
        }
        
        // 更新目标列表
        self.current_strategy.priority_targets = vec![
            reasoning.get_most_suspicious_player().unwrap_or_default()
        ];
        
        self.current_strategy.avoid_targets = vec![
            reasoning.get_most_trusted_player().unwrap_or_default()
        ];
    }
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

/// 发言策略
#[derive(Debug, Clone)]
pub struct SpeechStrategy {
    pub target: Option<String>,
    pub tone: SpeechTone,
    pub key_points: Vec<String>,
    pub confidence_level: f32,
    pub deception_elements: Vec<String>,
}

/// 发言语调
#[derive(Debug, Clone)]
pub enum SpeechTone {
    Aggressive,
    Defensive,
    Analytical,
    Confident,
    Neutral,
}