use crate::types::*;
use crate::error::{AppError, AppResult};
use crate::llm::LLMManager;
use crate::types::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use log::{info, warn};

/// 自然语言处理模块
pub struct NLPProcessor {
    llm_manager: Option<Arc<LLMManager>>,
    context_memory: Vec<SpeechRecord>,
}

/// 发言记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechRecord {
    pub speaker: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub phase: GamePhase,
    pub day: u32,
}

/// 发言分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAnalysis {
    pub intent: SpeechIntent,
    pub emotion: String,
    pub credibility: f32,
    pub key_information: Vec<String>,
    pub targets_mentioned: Vec<String>,
}

impl NLPProcessor {
    pub fn new(llm_manager: Option<Arc<LLMManager>>) -> Self {
        Self {
            llm_manager,
            context_memory: Vec::new(),
        }
    }
    
    /// 生成玩家发言
    pub async fn generate_speech(
        &mut self,
        player: &Player,
        game_state: &GameState,
        context: &str
    ) -> AppResult<String> {
        if let Some(llm_manager) = &self.llm_manager {
            let prompt = self.build_speech_prompt(player, game_state, context);
            
            match llm_manager.generate_with_fallback(prompt).await {
                Ok(response) => {
                    let speech = self.post_process_speech(response.as_str());
                    self.record_speech(player.id.clone(), speech.clone(), game_state.phase.clone(), game_state.day);
                    Ok(speech)
                }
                Err(_e) => {
                    Ok(self.generate_fallback_speech(player, game_state))
                }
            }
        } else {
            Ok(self.generate_fallback_speech(player, game_state))
        }
    }
    
    /// 分析玩家发言
    pub async fn analyze_speech(
        &mut self,
        speaker_id: String,
        content: String,
        game_state: &GameState
    ) -> AppResult<SpeechAnalysis> {
        self.record_speech(speaker_id.clone(), content.clone(), game_state.phase.clone(), game_state.day);
        
        let intent = self.analyze_intent(&content);
        let emotion = self.analyze_emotion(&content);
        let credibility = self.calculate_credibility(&content);
        let key_info = self.extract_key_info(&content);
        let targets = self.extract_targets(&content, game_state);
        
        Ok(SpeechAnalysis {
            intent,
            emotion,
            credibility,
            key_information: key_info,
            targets_mentioned: targets,
        })
    }
    
    fn build_speech_prompt(&self, player: &Player, game_state: &GameState, context: &str) -> String {
        let role_desc = match player.role.role_type {
            RoleType::Werewolf => "你是狼人，需要隐藏身份，误导好人。",
            RoleType::Seer => "你是预言家，需要分享验人信息。",
            RoleType::Villager => "你是村民，需要找出狼人。",
            _ => "你需要根据身份合理发言。",
        };
        
        format!(
            "你是{}，{}当前是第{}天。存活玩家：{}。{}请生成50-150字的发言：",
            player.name,
            role_desc,
            game_state.day,
            self.format_alive_players(game_state),
            context
        )
    }
    
    fn generate_fallback_speech(&self, player: &Player, game_state: &GameState) -> String {
        let templates = match player.role.role_type {
            RoleType::Werewolf => vec![
                "我觉得某位玩家的发言有些可疑。",
                "我们需要仔细分析投票情况。",
                "我倾向于相信好人的判断。",
            ],
            RoleType::Seer => vec![
                "我有一些信息要分享。",
                "根据我的观察，有人可能有问题。",
                "大家要相信我的判断。",
            ],
            _ => vec![
                "我需要再观察一下。",
                "大家的分析都很有道理。",
                "我暂时保留意见。",
            ],
        };
        
        templates[game_state.day as usize % templates.len()].to_string()
    }
    
    fn analyze_intent(&self, content: &str) -> SpeechIntent {
        let intent_type = if content.contains("投票") {
            SpeechType::Vote
        } else if content.contains("怀疑") {
            SpeechType::Accusation
        } else if content.contains("不是我") {
            SpeechType::Defense
        } else if content.contains("验了") {
            SpeechType::Information
        } else {
            SpeechType::Strategy
        };
        
        SpeechIntent {
            intent_type,
            target: None,
            content: content.to_string(),
            confidence: 0.7,
        }
    }
    
    fn analyze_emotion(&self, content: &str) -> String {
        if content.contains("气死") || content.contains("愤怒") {
            "愤怒".to_string()
        } else if content.contains("紧张") || content.contains("不是我") {
            "紧张".to_string()
        } else if content.contains("一定") || content.contains("肯定") {
            "自信".to_string()
        } else {
            "冷静".to_string()
        }
    }
    
    fn calculate_credibility(&self, content: &str) -> f32 {
        let mut score: f32 = 0.7;
        
        if content.contains("绝对") || content.contains("一定") {
            score -= 0.1;
        }
        if content.contains("为什么怀疑我") {
            score -= 0.2;
        }
        if content.len() > 200 {
            score -= 0.1;
        }
        
        score.clamp(0.0, 1.0)
    }
    
    fn extract_key_info(&self, content: &str) -> Vec<String> {
        let mut info = Vec::new();
        
        if content.contains("我是") {
            info.push("角色声明".to_string());
        }
        if content.contains("验了") {
            info.push("验人结果".to_string());
        }
        if content.contains("投票") {
            info.push("投票意向".to_string());
        }
        
        info
    }
    
    fn extract_targets(&self, content: &str, game_state: &GameState) -> Vec<String> {
        let mut targets = Vec::new();
        
        for player in &game_state.players {
            if content.contains(&player.name) {
                targets.push(player.id.clone());
            }
        }
        
        targets
    }
    
    fn record_speech(&mut self, speaker: String, content: String, phase: GamePhase, day: u32) {
        let record = SpeechRecord {
            speaker,
            content,
            timestamp: chrono::Utc::now(),
            phase,
            day,
        };
        
        self.context_memory.push(record);
        
        if self.context_memory.len() > 50 {
            self.context_memory.remove(0);
        }
    }
    
    fn post_process_speech(&self, speech: &str) -> String {
        let mut processed = speech.trim().to_string();
        
        if processed.len() > 200 {
            processed = processed.chars().take(197).collect::<String>() + "...";
        }
        
        if processed.len() < 10 {
            processed = "我需要再思考一下。".to_string();
        }
        
        processed
    }
    
    fn format_alive_players(&self, game_state: &GameState) -> String {
        game_state.players.iter()
            .filter(|p| p.is_alive)
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
