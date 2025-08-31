use crate::types::*;
use crate::error::{AppError, AppResult};
use crate::ai::{reasoning::ReasoningEngine, strategy::StrategyEngine, nlp::NLPProcessor};
use crate::llm::LLMManager;
use std::sync::Arc;
use log::{info, warn, debug};

/// AI代理 - 智能AI玩家的核心
pub struct AIAgent {
    pub player_id: String,
    pub personality: AIPersonality,
    pub role: Role,
    reasoning_engine: ReasoningEngine,
    strategy_engine: StrategyEngine,
    nlp_processor: NLPProcessor,
    memory: AIMemory,
}

/// AI记忆系统
#[derive(Debug, Clone)]
pub struct AIMemory {
    pub known_roles: std::collections::HashMap<String, RoleType>,
    pub trust_scores: std::collections::HashMap<String, f32>,
    pub suspicion_scores: std::collections::HashMap<String, f32>,
    pub voting_history: Vec<VoteRecord>,
    pub speech_history: Vec<SpeechMemory>,
    pub night_action_history: Vec<NightActionMemory>,
}

/// 发言记忆
#[derive(Debug, Clone)]
pub struct SpeechMemory {
    pub speaker: String,
    pub content: String,
    pub day: u32,
    pub phase: GamePhase,
    pub my_reaction: String,
}

/// 夜晚行动记忆
#[derive(Debug, Clone)]
pub struct NightActionMemory {
    pub night: u32,
    pub my_action: Option<NightAction>,
    pub observed_results: Vec<String>,
}

/// AI决策结果
#[derive(Debug, Clone)]
pub struct AIDecision {
    pub decision_type: DecisionType,
    pub target: Option<String>,
    pub reasoning: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum DecisionType {
    Vote,
    NightAction,
    Speech,
    ClaimRole,
}

impl AIAgent {
    /// 创建新的AI代理
    pub fn new(
        player_id: String,
        personality: AIPersonality,
        role: Role,
        llm_manager: Option<Arc<LLMManager>>
    ) -> Self {
        let reasoning_engine = ReasoningEngine::new();
        let strategy_engine = StrategyEngine::new(personality.clone(), &role);
        let nlp_processor = NLPProcessor::new(llm_manager);
        
        Self {
            player_id,
            personality,
            role,
            reasoning_engine,
            strategy_engine,
            nlp_processor,
            memory: AIMemory::new(),
        }
    }
    
    /// 初始化AI代理
    pub fn initialize(&mut self, game_state: &GameState) -> AppResult<()> {
        self.reasoning_engine.initialize(game_state);
        
        // 初始化对其他玩家的印象
        for player in &game_state.players {
            if player.id != self.player_id {
                self.memory.trust_scores.insert(player.id.clone(), 0.5);
                self.memory.suspicion_scores.insert(player.id.clone(), 0.5);
            }
        }
        
        info!(\"AI代理 {} 已初始化\", self.player_id);
        Ok(())
    }
    
    /// 决定夜晚行动
    pub async fn decide_night_action(&mut self, game_state: &GameState) -> AppResult<Option<NightAction>> {
        debug!(\"AI {} 正在决定夜晚行动\", self.player_id);
        
        // 更新推理状态
        self.update_reasoning(game_state)?;
        
        // 生成策略决策
        let action = self.strategy_engine.decide_night_action(
            &self.role,
            game_state,
            &self.reasoning_engine
        ).await?;
        
        // 记录行动决策
        if let Some(ref action) = action {
            let memory = NightActionMemory {
                night: game_state.day,
                my_action: Some(action.clone()),
                observed_results: Vec::new(),
            };
            self.memory.night_action_history.push(memory);
            
            info!(\"AI {} 决定夜晚行动: {:?}\", self.player_id, action.action);
        }
        
        Ok(action)
    }
    
    /// 决定投票目标
    pub async fn decide_vote(&mut self, game_state: &GameState) -> AppResult<Option<String>> {
        debug!(\"AI {} 正在决定投票目标\", self.player_id);
        
        // 更新推理状态
        self.update_reasoning(game_state)?;
        
        // 策略决策
        let target = self.strategy_engine.decide_vote_target(
            game_state,
            &self.reasoning_engine
        ).await?;
        
        if let Some(ref target_id) = target {
            info!(\"AI {} 决定投票给: {}\", self.player_id, target_id);
            
            // 记录投票决策的推理过程
            let reasoning = self.get_vote_reasoning(target_id);
            debug!(\"投票推理: {}\", reasoning);
        }
        
        Ok(target)
    }
    
    /// 生成发言
    pub async fn generate_speech(
        &mut self,
        game_state: &GameState,
        speech_type: SpeechType
    ) -> AppResult<String> {
        debug!(\"AI {} 正在生成发言，类型: {:?}\", self.player_id, speech_type);
        
        // 更新推理状态
        self.update_reasoning(game_state)?;
        
        // 生成发言策略
        let strategy = self.strategy_engine.generate_speech_strategy(
            game_state,
            &self.reasoning_engine,
            speech_type
        );
        
        // 使用NLP生成发言
        let player = self.create_player_snapshot();
        let context = self.build_speech_context(game_state);
        
        let speech = self.nlp_processor.generate_speech(
            &player,
            game_state,
            &context
        ).await?;
        
        // 记录发言
        self.memory.speech_history.push(SpeechMemory {
            speaker: self.player_id.clone(),
            content: speech.clone(),
            day: game_state.day,
            phase: game_state.phase.clone(),
            my_reaction: \"我说的话\".to_string(),
        });
        
        info!(\"AI {} 生成发言: {}\", self.player_id, speech);
        Ok(speech)
    }
    
    /// 处理其他玩家的发言
    pub async fn process_player_speech(
        &mut self,
        speaker_id: String,
        content: String,
        game_state: &GameState
    ) -> AppResult<()> {
        debug!(\"AI {} 正在处理 {} 的发言\", self.player_id, speaker_id);
        
        // 使用NLP分析发言
        let analysis = self.nlp_processor.analyze_speech(
            speaker_id.clone(),
            content.clone(),
            game_state
        ).await?;
        
        // 更新推理引擎
        self.reasoning_engine.analyze_speech(
            speaker_id.clone(),
            &content
        )?;
        
        // 更新对该玩家的印象
        self.update_player_impression(&speaker_id, &analysis);
        
        // 记录发言
        self.memory.speech_history.push(SpeechMemory {
            speaker: speaker_id.clone(),
            content,
            day: game_state.day,
            phase: game_state.phase.clone(),
            my_reaction: format!(\"可信度: {:.2}\", analysis.credibility),
        });
        
        Ok(())
    }
    
    /// 处理投票信息
    pub fn process_vote(&mut self, vote: VoteRecord) -> AppResult<()> {
        debug!(\"AI {} 处理投票: {} -> {}\", self.player_id, vote.voter, vote.target);
        
        // 分析投票行为
        self.reasoning_engine.analyze_vote(
            vote.voter.clone(),
            vote.target.clone()
        )?;
        
        // 更新投票历史
        self.memory.voting_history.push(vote);
        
        Ok(())
    }
    
    /// 处理夜晚结果
    pub fn process_night_result(&mut self, result: NightResult) -> AppResult<()> {
        debug!(\"AI {} 处理夜晚结果\", self.player_id);
        
        // 更新最近的夜晚行动记忆
        if let Some(last_memory) = self.memory.night_action_history.last_mut() {
            last_memory.observed_results.push(format!(\"{:?}\", result));
        }
        
        // 根据结果更新推理
        match result {
            NightResult::PlayerKilled(player_id) => {
                info!(\"AI {} 得知 {} 被杀\", self.player_id, player_id);
                // 分析谁可能是凶手
                self.analyze_kill_target(&player_id);
            }
            NightResult::PlayerSaved => {
                info!(\"AI {} 得知有人被救\", self.player_id);
                // 分析女巫行为
            }
            NightResult::NoKill => {
                info!(\"AI {} 得知平安夜\", self.player_id);
                // 分析可能的原因
            }
        }
        
        Ok(())
    }
    
    /// 获取AI的分析报告
    pub fn get_analysis_report(&self) -> AIAnalysisReport {
        let reasoning_report = self.reasoning_engine.get_analysis_report();
        
        AIAnalysisReport {
            agent_id: self.player_id.clone(),
            current_strategy: format!(\"{:?}\", self.strategy_engine),
            trust_rankings: self.get_trust_rankings(),
            suspicion_rankings: self.get_suspicion_rankings(),
            reasoning_summary: reasoning_report,
            memory_highlights: self.get_memory_highlights(),
        }
    }
    
    // 私有辅助方法
    
    fn update_reasoning(&mut self, game_state: &GameState) -> AppResult<()> {
        // 更新策略引擎
        self.strategy_engine.update_strategy(game_state, &self.reasoning_engine);
        Ok(())
    }
    
    fn create_player_snapshot(&self) -> Player {
        Player {
            id: self.player_id.clone(),
            name: format!(\"AI_{}\", self.player_id),
            role: self.role.clone(),
            faction: self.role.faction.clone(),
            is_alive: true,
            is_ai: true,
            personality: Some(self.personality.clone()),
        }
    }
    
    fn build_speech_context(&self, game_state: &GameState) -> String {
        let recent_speeches = self.memory.speech_history.iter()
            .rev()
            .take(3)
            .map(|s| format!(\"{}: {}\", s.speaker, s.content))
            .collect::<Vec<_>>()
            .join(\"\n\");
        
        format!(
            \"当前阶段: {:?}\n最近发言:\n{}\",
            game_state.phase,
            recent_speeches
        )
    }
    
    fn update_player_impression(&mut self, player_id: &str, analysis: &crate::ai::nlp::SpeechAnalysis) {
        // 更新信任度
        if let Some(trust) = self.memory.trust_scores.get_mut(player_id) {
            *trust = (*trust + analysis.credibility) / 2.0;
        }
        
        // 更新怀疑度
        if let Some(suspicion) = self.memory.suspicion_scores.get_mut(player_id) {
            *suspicion = (*suspicion + (1.0 - analysis.credibility)) / 2.0;
        }
    }
    
    fn get_vote_reasoning(&self, target_id: &str) -> String {
        let suspicion = self.memory.suspicion_scores.get(target_id).unwrap_or(&0.5);
        let trust = self.memory.trust_scores.get(target_id).unwrap_or(&0.5);
        
        format!(
            \"投票给{}：怀疑度{:.2}，信任度{:.2}\",
            target_id, suspicion, trust
        )
    }
    
    fn analyze_kill_target(&mut self, target_id: &str) {
        // 分析为什么这个玩家被杀
        if let Some(trust) = self.memory.trust_scores.get(target_id) {
            if *trust > 0.7 {
                info!(\"AI {} 认为 {} 被杀是因为太可信\", self.player_id, target_id);
            }
        }
    }
    
    fn get_trust_rankings(&self) -> Vec<(String, f32)> {
        let mut rankings: Vec<_> = self.memory.trust_scores.iter()
            .map(|(id, &score)| (id.clone(), score))
            .collect();
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }
    
    fn get_suspicion_rankings(&self) -> Vec<(String, f32)> {
        let mut rankings: Vec<_> = self.memory.suspicion_scores.iter()
            .map(|(id, &score)| (id.clone(), score))
            .collect();
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }
    
    fn get_memory_highlights(&self) -> Vec<String> {
        let mut highlights = Vec::new();
        
        // 最近的重要发言
        for speech in self.memory.speech_history.iter().rev().take(3) {
            highlights.push(format!(
                \"第{}天{:?}: {} - {}\",
                speech.day, speech.phase, speech.speaker, speech.content
            ));
        }
        
        highlights
    }
}

impl AIMemory {
    fn new() -> Self {
        Self {
            known_roles: std::collections::HashMap::new(),
            trust_scores: std::collections::HashMap::new(),
            suspicion_scores: std::collections::HashMap::new(),
            voting_history: Vec::new(),
            speech_history: Vec::new(),
            night_action_history: Vec::new(),
        }
    }
}

/// 夜晚结果枚举
#[derive(Debug, Clone)]
pub enum NightResult {
    PlayerKilled(String),
    PlayerSaved,
    NoKill,
}

/// AI分析报告
#[derive(Debug, Clone)]
pub struct AIAnalysisReport {
    pub agent_id: String,
    pub current_strategy: String,
    pub trust_rankings: Vec<(String, f32)>,
    pub suspicion_rankings: Vec<(String, f32)>,
    pub reasoning_summary: crate::ai::reasoning::ReasoningReport,
    pub memory_highlights: Vec<String>,
}