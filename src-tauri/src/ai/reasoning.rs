use crate::types::*;
use crate::error::{AppError, AppResult};
use crate::types::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, warn, debug};

/// 贝叶斯推理网络节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianNode {
    pub player_id: String,
    pub role_probabilities: HashMap<RoleType, f32>,
    pub faction_probability: f32, // 狼人概率
    pub trust_score: f32,
    pub suspicion_score: f32,
    pub evidence: Vec<Evidence>,
}

/// 证据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub confidence: f32,
    pub source: String,
    pub description: String,
    pub weight: f32,
}

/// 证据类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    VotingPattern,      // 投票模式
    SpeechAnalysis,     // 发言分析
    NightResult,        // 夜晚结果
    RoleClaimConsistency, // 身份声明一致性
    DefensiveBehavior,  // 防御行为
    AggressiveBehavior, // 攻击行为
    LogicalInconsistency, // 逻辑矛盾
    TeamworkIndicator,  // 团队协作指标
}

/// 推理引擎
pub struct ReasoningEngine {
    nodes: HashMap<String, BayesianNode>,
    game_state: Option<GameState>,
    reasoning_rules: Vec<ReasoningRule>,
}

/// 推理规则
#[derive(Debug, Clone)]
pub struct ReasoningRule {
    pub name: String,
    pub condition: RuleCondition,
    pub conclusion: RuleConclusion,
    pub confidence: f32,
}

/// 规则条件
#[derive(Debug, Clone)]
pub enum RuleCondition {
    PlayerVotedFor { voter: String, target: String },
    PlayerDefended { defender: String, defended: String },
    ConsistentRoleClaim { player: String, role: RoleType },
    NightKillPattern { pattern: String },
    SpeechContainsKeywords { player: String, keywords: Vec<String> },
}

/// 规则结论
#[derive(Debug, Clone)]
pub enum RuleConclusion {
    IncreaseSuspicion { player: String, amount: f32 },
    DecreaseSuspicion { player: String, amount: f32 },
    IncreaseTrust { player: String, amount: f32 },
    SetRoleProbability { player: String, role: RoleType, probability: f32 },
    MarkAsTeammates { player1: String, player2: String },
}

impl ReasoningEngine {
    /// 创建新的推理引擎
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            game_state: None,
            reasoning_rules: Self::create_default_rules(),
        }
    }
    
    /// 初始化贝叶斯网络
    pub fn initialize(&mut self, game_state: &GameState) {
        self.game_state = Some(game_state.clone());
        
        // 为每个玩家创建节点
        for player in &game_state.players {
            let mut role_probabilities = HashMap::new();
            
            // 基于角色分配设置初始概率
            let total_players = game_state.players.len() as f32;
            for (role, count) in &game_state.game_config.role_distribution {
                let probability = *count as f32 / total_players;
                role_probabilities.insert(role.clone(), probability);
            }
            
            let faction_probability = if role_probabilities.contains_key(&RoleType::Werewolf) {
                role_probabilities[&RoleType::Werewolf]
            } else {
                0.0
            };
            
            let node = BayesianNode {
                player_id: player.id.clone(),
                role_probabilities,
                faction_probability,
                trust_score: 0.5,
                suspicion_score: 0.5,
                evidence: Vec::new(),
            };
            
            self.nodes.insert(player.id.clone(), node);
        }
        
        info!("推理引擎已初始化，共{}个节点", self.nodes.len());
    }
    
    /// 添加证据并更新推理
    pub fn add_evidence(&mut self, player_id: String, evidence: Evidence) -> AppResult<()> {
        if let Some(node) = self.nodes.get_mut(&player_id) {
            node.evidence.push(evidence.clone());
            self.update_probabilities(&player_id, &evidence)?;
            debug!("为玩家{}添加证据: {:?}", player_id, evidence.evidence_type);
        }
        Ok(())
    }
    
    /// 更新概率
    fn update_probabilities(&mut self, player_id: &str, evidence: &Evidence) -> AppResult<()> {
        if let Some(node) = self.nodes.get_mut(player_id) {
            match evidence.evidence_type {
                EvidenceType::SpeechAnalysis => {
                    // 基于发言分析更新概率
                    if evidence.confidence > 0.7 {
                        node.suspicion_score += evidence.weight * 0.2;
                        node.trust_score -= evidence.weight * 0.1;
                    }
                }
                EvidenceType::VotingPattern => {
                    // 基于投票模式更新概率
                    node.faction_probability += evidence.weight * 0.15;
                }
                EvidenceType::DefensiveBehavior => {
                    // 防御行为可能表明身份暴露
                    node.suspicion_score += evidence.weight * 0.25;
                }
                EvidenceType::LogicalInconsistency => {
                    // 逻辑矛盾强烈指向狼人
                    node.faction_probability += evidence.weight * 0.3;
                    node.suspicion_score += evidence.weight * 0.4;
                }
                _ => {
                    // 其他证据类型的处理
                    node.suspicion_score += evidence.weight * 0.1;
                }
            }
            
            // 确保概率在有效范围内
            node.suspicion_score = node.suspicion_score.clamp(0.0, 1.0);
            node.trust_score = node.trust_score.clamp(0.0, 1.0);
            node.faction_probability = node.faction_probability.clamp(0.0, 1.0);
        }
        
        Ok(())
    }
    
    /// 分析投票行为
    pub fn analyze_vote(&mut self, voter_id: String, target_id: String) -> AppResult<()> {
        // 分析投票模式
        let evidence = Evidence {
            evidence_type: EvidenceType::VotingPattern,
            confidence: 0.8,
            source: "voting_analysis".to_string(),
            description: format!("{}投票给{}", voter_id, target_id),
            weight: 0.3,
        };
        
        self.add_evidence(voter_id.clone(), evidence)?;
        
        // 应用推理规则
        self.apply_reasoning_rules()?;
        
        Ok(())
    }
    
    /// 分析发言内容
    pub fn analyze_speech(&mut self, player_id: String, content: &str) -> AppResult<()> {
        let analysis = self.perform_speech_analysis(content);
        
        let evidence = Evidence {
            evidence_type: EvidenceType::SpeechAnalysis,
            confidence: analysis.confidence,
            source: "speech_analysis".to_string(),
            description: format!("发言分析: {}", analysis.summary),
            weight: analysis.suspicion_weight,
        };
        
        self.add_evidence(player_id, evidence)?;
        Ok(())
    }
    
    /// 执行发言分析
    fn perform_speech_analysis(&self, content: &str) -> SpeechAnalysisResult {
        let content_lower = content.to_lowercase();
        let mut suspicion_weight: f32 = 0.0;
        let mut confidence: f32 = 0.5;
        let mut summary = String::new();
        
        // 检查可疑关键词
        let suspicious_keywords = [
            "一定是", "肯定是", "我觉得不是", "太明显了",
            "这么简单", "显而易见", "不可能", "绝对"
        ];
        
        let defensive_keywords = [
            "我不是", "相信我", "为什么怀疑我", "我是好人",
            "你们错了", "冤枉", "诬陷"
        ];
        
        let aggressive_keywords = [
            "一定是狼", "明显的狼", "狼人", "出他",
            "投他", "他有问题"
        ];
        
        // 检查可疑关键词
        for keyword in &suspicious_keywords {
            if content_lower.contains(keyword) {
                suspicion_weight += 0.1;
                summary.push_str(&format!("包含可疑词汇: {}; ", keyword));
            }
        }
        
        // 检查防御性关键词
        for keyword in &defensive_keywords {
            if content_lower.contains(keyword) {
                suspicion_weight += 0.2;
                confidence += 0.1;
                summary.push_str(&format!("防御性发言: {}; ", keyword));
            }
        }
        
        // 检查攻击性关键词
        for keyword in &aggressive_keywords {
            if content_lower.contains(keyword) {
                suspicion_weight += 0.05;
                summary.push_str(&format!("攻击性发言: {}; ", keyword));
            }
        }
        
        // 分析发言长度
        if content.len() > 200 {
            suspicion_weight += 0.05;
            summary.push_str("发言较长，可能过度解释; ");
        } else if content.len() < 20 {
            suspicion_weight += 0.1;
            summary.push_str("发言过短，可能隐藏信息; ");
        }
        
        SpeechAnalysisResult {
            suspicion_weight: suspicion_weight.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            summary: if summary.is_empty() {
                "正常发言".to_string()
            } else {
                summary
            },
        }
    }
    
    /// 应用推理规则
    fn apply_reasoning_rules(&mut self) -> AppResult<()> {
        for rule in self.reasoning_rules.clone() {
            if self.evaluate_rule_condition(&rule.condition) {
                self.apply_rule_conclusion(&rule.conclusion, rule.confidence)?;
            }
        }
        Ok(())
    }
    
    /// 评估规则条件
    fn evaluate_rule_condition(&self, condition: &RuleCondition) -> bool {
        match condition {
            RuleCondition::PlayerVotedFor { voter, target } => {
                // 检查投票记录
                if let Some(game_state) = &self.game_state {
                    game_state.votes.iter().any(|vote| 
                        vote.voter == *voter && vote.target == *target
                    )
                } else {
                    false
                }
            }
            RuleCondition::ConsistentRoleClaim { player, role } => {
                // 检查角色声明一致性
                if let Some(node) = self.nodes.get(player) {
                    node.role_probabilities.get(role).unwrap_or(&0.0) > &0.8
                } else {
                    false
                }
            }
            _ => false, // 其他条件的实现
        }
    }
    
    /// 应用规则结论
    fn apply_rule_conclusion(&mut self, conclusion: &RuleConclusion, confidence: f32) -> AppResult<()> {
        match conclusion {
            RuleConclusion::IncreaseSuspicion { player, amount } => {
                if let Some(node) = self.nodes.get_mut(player) {
                    node.suspicion_score += amount * confidence;
                    node.suspicion_score = node.suspicion_score.clamp(0.0, 1.0);
                }
            }
            RuleConclusion::DecreaseSuspicion { player, amount } => {
                if let Some(node) = self.nodes.get_mut(player) {
                    node.suspicion_score -= amount * confidence;
                    node.suspicion_score = node.suspicion_score.clamp(0.0, 1.0);
                }
            }
            RuleConclusion::IncreaseTrust { player, amount } => {
                if let Some(node) = self.nodes.get_mut(player) {
                    node.trust_score += amount * confidence;
                    node.trust_score = node.trust_score.clamp(0.0, 1.0);
                }
            }
            RuleConclusion::SetRoleProbability { player, role, probability } => {
                if let Some(node) = self.nodes.get_mut(player) {
                    node.role_probabilities.insert(role.clone(), *probability * confidence);
                }
            }
            _ => {} // 其他结论的实现
        }
        Ok(())
    }
    
    /// 获取最可疑的玩家
    pub fn get_most_suspicious_player(&self) -> Option<String> {
        self.nodes.iter()
            .max_by(|(_, a), (_, b)| a.suspicion_score.partial_cmp(&b.suspicion_score).unwrap())
            .map(|(id, _)| id.clone())
    }
    
    /// 获取最信任的玩家
    pub fn get_most_trusted_player(&self) -> Option<String> {
        self.nodes.iter()
            .max_by(|(_, a), (_, b)| a.trust_score.partial_cmp(&b.trust_score).unwrap())
            .map(|(id, _)| id.clone())
    }
    
    /// 获取玩家的狼人概率
    pub fn get_werewolf_probability(&self, player_id: &str) -> f32 {
        self.nodes.get(player_id)
            .map(|node| node.faction_probability)
            .unwrap_or(0.5)
    }
    
    /// 获取推理分析报告
    pub fn get_analysis_report(&self) -> ReasoningReport {
        let mut player_analysis = Vec::new();
        
        for (player_id, node) in &self.nodes {
            let analysis = PlayerAnalysis {
                player_id: player_id.clone(),
                werewolf_probability: node.faction_probability,
                suspicion_score: node.suspicion_score,
                trust_score: node.trust_score,
                main_evidence: node.evidence.iter()
                    .take(3)
                    .map(|e| e.description.clone())
                    .collect(),
            };
            player_analysis.push(analysis);
        }
        
        // 按怀疑度排序
        player_analysis.sort_by(|a, b| 
            b.suspicion_score.partial_cmp(&a.suspicion_score).unwrap()
        );
        
        ReasoningReport {
            player_analysis,
            most_suspicious: self.get_most_suspicious_player(),
            most_trusted: self.get_most_trusted_player(),
        }
    }
    
    /// 创建默认推理规则
    fn create_default_rules() -> Vec<ReasoningRule> {
        vec![
            ReasoningRule {
                name: "连续防御规则".to_string(),
                condition: RuleCondition::SpeechContainsKeywords {
                    player: "any".to_string(),
                    keywords: vec!["我不是".to_string(), "相信我".to_string()],
                },
                conclusion: RuleConclusion::IncreaseSuspicion {
                    player: "self".to_string(),
                    amount: 0.2,
                },
                confidence: 0.8,
            },
            // 可以添加更多规则
        ]
    }
    

}

/// 发言分析结果
#[derive(Debug)]
struct SpeechAnalysisResult {
    suspicion_weight: f32,
    confidence: f32,
    summary: String,
}

/// 玩家分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnalysis {
    pub player_id: String,
    pub werewolf_probability: f32,
    pub suspicion_score: f32,
    pub trust_score: f32,
    pub main_evidence: Vec<String>,
}

/// 推理报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningReport {
    pub player_analysis: Vec<PlayerAnalysis>,
    pub most_suspicious: Option<String>,
    pub most_trusted: Option<String>,
}
