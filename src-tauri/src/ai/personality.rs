use crate::types::AIPersonality;
use serde::{Serialize, Deserialize};
use rand::{thread_rng, Rng};

/// 性格管理器
pub struct PersonalityManager;

/// 预定义的AI性格模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_traits: PersonalityTraits,
    pub speech_patterns: SpeechPatterns,
    pub behavioral_tendencies: BehavioralTendencies,
}

/// 性格特征（扩展版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub aggressiveness: f32,    // 攻击性 0.0-1.0
    pub logic: f32,            // 逻辑性 0.0-1.0
    pub deception: f32,        // 欺骗能力 0.0-1.0
    pub trustfulness: f32,     // 信任度 0.0-1.0
    pub patience: f32,         // 耐心 0.0-1.0
    pub confidence: f32,       // 自信 0.0-1.0
    pub empathy: f32,          // 同理心 0.0-1.0
    pub impulsiveness: f32,    // 冲动性 0.0-1.0
}

/// 发言模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechPatterns {
    pub verbosity: SpeechVerbosity,    // 话多话少
    pub formality: SpeechFormality,    // 正式程度
    pub emotional_expression: f32,      // 情感表达强度
    pub humor_usage: f32,              // 幽默使用频率
    pub question_frequency: f32,       // 提问频率
    pub interruption_tendency: f32,    // 打断倾向
}

/// 行为倾向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralTendencies {
    pub risk_taking: f32,              // 冒险倾向
    pub team_cooperation: f32,         // 团队合作
    pub leadership: f32,               // 领导力
    pub adaptability: f32,             // 适应性
    pub memory_retention: f32,         // 记忆保持
    pub pattern_recognition: f32,      // 模式识别
}

/// 发言详细程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechVerbosity {
    Concise,     // 简洁
    Moderate,    // 适中
    Verbose,     // 冗长
}

/// 发言正式程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechFormality {
    Casual,      // 随意
    Neutral,     // 中性
    Formal,      // 正式
}

impl PersonalityManager {
    /// 获取所有预定义性格模板
    pub fn get_personality_templates() -> Vec<PersonalityTemplate> {
        vec![
            // 逻辑分析型
            PersonalityTemplate {
                id: "analytical".to_string(),
                name: "逻辑分析师".to_string(),
                description: "冷静理性，善于逻辑推理，发言精准有条理".to_string(),
                base_traits: PersonalityTraits {
                    aggressiveness: 0.3,
                    logic: 0.9,
                    deception: 0.2,
                    trustfulness: 0.7,
                    patience: 0.8,
                    confidence: 0.7,
                    empathy: 0.4,
                    impulsiveness: 0.2,
                },
                speech_patterns: SpeechPatterns {
                    verbosity: SpeechVerbosity::Moderate,
                    formality: SpeechFormality::Formal,
                    emotional_expression: 0.3,
                    humor_usage: 0.2,
                    question_frequency: 0.7,
                    interruption_tendency: 0.2,
                },
                behavioral_tendencies: BehavioralTendencies {
                    risk_taking: 0.3,
                    team_cooperation: 0.8,
                    leadership: 0.6,
                    adaptability: 0.7,
                    memory_retention: 0.9,
                    pattern_recognition: 0.9,
                },
            },
            
            // 情绪冲动型
            PersonalityTemplate {
                id: "impulsive".to_string(),
                name: "情绪冲动者".to_string(),
                description: "感情丰富，容易激动，行为较为冲动".to_string(),
                base_traits: PersonalityTraits {
                    aggressiveness: 0.8,
                    logic: 0.4,
                    deception: 0.3,
                    trustfulness: 0.6,
                    patience: 0.2,
                    confidence: 0.6,
                    empathy: 0.8,
                    impulsiveness: 0.9,
                },
                speech_patterns: SpeechPatterns {
                    verbosity: SpeechVerbosity::Verbose,
                    formality: SpeechFormality::Casual,
                    emotional_expression: 0.9,
                    humor_usage: 0.5,
                    question_frequency: 0.8,
                    interruption_tendency: 0.8,
                },
                behavioral_tendencies: BehavioralTendencies {
                    risk_taking: 0.8,
                    team_cooperation: 0.5,
                    leadership: 0.7,
                    adaptability: 0.6,
                    memory_retention: 0.4,
                    pattern_recognition: 0.5,
                },
            },
            
            // 狡猾欺骗型
            PersonalityTemplate {
                id: "deceptive".to_string(),
                name: "狡猾欺骗者".to_string(),
                description: "善于伪装和欺骗，发言具有迷惑性".to_string(),
                base_traits: PersonalityTraits {
                    aggressiveness: 0.5,
                    logic: 0.7,
                    deception: 0.9,
                    trustfulness: 0.3,
                    patience: 0.8,
                    confidence: 0.8,
                    empathy: 0.3,
                    impulsiveness: 0.3,
                },
                speech_patterns: SpeechPatterns {
                    verbosity: SpeechVerbosity::Moderate,
                    formality: SpeechFormality::Neutral,
                    emotional_expression: 0.6,
                    humor_usage: 0.4,
                    question_frequency: 0.5,
                    interruption_tendency: 0.3,
                },
                behavioral_tendencies: BehavioralTendencies {
                    risk_taking: 0.7,
                    team_cooperation: 0.4,
                    leadership: 0.5,
                    adaptability: 0.9,
                    memory_retention: 0.8,
                    pattern_recognition: 0.7,
                },
            },
            
            // 保守谨慎型
            PersonalityTemplate {
                id: "cautious".to_string(),
                name: "保守谨慎者".to_string(),
                description: "行事谨慎，不轻易表态，观察力强".to_string(),
                base_traits: PersonalityTraits {
                    aggressiveness: 0.2,
                    logic: 0.6,
                    deception: 0.4,
                    trustfulness: 0.8,
                    patience: 0.9,
                    confidence: 0.4,
                    empathy: 0.7,
                    impulsiveness: 0.1,
                },
                speech_patterns: SpeechPatterns {
                    verbosity: SpeechVerbosity::Concise,
                    formality: SpeechFormality::Formal,
                    emotional_expression: 0.3,
                    humor_usage: 0.1,
                    question_frequency: 0.3,
                    interruption_tendency: 0.1,
                },
                behavioral_tendencies: BehavioralTendencies {
                    risk_taking: 0.2,
                    team_cooperation: 0.9,
                    leadership: 0.3,
                    adaptability: 0.4,
                    memory_retention: 0.8,
                    pattern_recognition: 0.8,
                },
            },
            
            // 领袖型
            PersonalityTemplate {
                id: "leader".to_string(),
                name: "天生领袖".to_string(),
                description: "具有领导力，善于组织和指挥，自信果断".to_string(),
                base_traits: PersonalityTraits {
                    aggressiveness: 0.7,
                    logic: 0.8,
                    deception: 0.4,
                    trustfulness: 0.5,
                    patience: 0.6,
                    confidence: 0.9,
                    empathy: 0.6,
                    impulsiveness: 0.4,
                },
                speech_patterns: SpeechPatterns {
                    verbosity: SpeechVerbosity::Moderate,
                    formality: SpeechFormality::Formal,
                    emotional_expression: 0.7,
                    humor_usage: 0.3,
                    question_frequency: 0.4,
                    interruption_tendency: 0.5,
                },
                behavioral_tendencies: BehavioralTendencies {
                    risk_taking: 0.6,
                    team_cooperation: 0.7,
                    leadership: 0.9,
                    adaptability: 0.7,
                    memory_retention: 0.7,
                    pattern_recognition: 0.6,
                },
            },
            
            // 随性自由型
            PersonalityTemplate {
                id: "chaotic".to_string(),
                name: "随性自由者".to_string(),
                description: "行为不可预测，思维跳跃，带来变数".to_string(),
                base_traits: PersonalityTraits {
                    aggressiveness: 0.6,
                    logic: 0.3,
                    deception: 0.5,
                    trustfulness: 0.4,
                    patience: 0.3,
                    confidence: 0.7,
                    empathy: 0.5,
                    impulsiveness: 0.8,
                },
                speech_patterns: SpeechPatterns {
                    verbosity: SpeechVerbosity::Verbose,
                    formality: SpeechFormality::Casual,
                    emotional_expression: 0.8,
                    humor_usage: 0.8,
                    question_frequency: 0.9,
                    interruption_tendency: 0.7,
                },
                behavioral_tendencies: BehavioralTendencies {
                    risk_taking: 0.9,
                    team_cooperation: 0.4,
                    leadership: 0.5,
                    adaptability: 0.8,
                    memory_retention: 0.3,
                    pattern_recognition: 0.4,
                },
            },
        ]
    }
    
    /// 根据模板创建个性化AI性格
    pub fn create_personality_from_template(
        template: &PersonalityTemplate,
        variation_factor: f32
    ) -> AIPersonality {
        let mut rng = thread_rng();
        
        // 在模板基础上添加随机变化
        let varied_traits = crate::types::PersonalityTraits {
            aggressiveness: Self::vary_trait(template.base_traits.aggressiveness, variation_factor, &mut rng),
            logic: Self::vary_trait(template.base_traits.logic, variation_factor, &mut rng),
            deception: Self::vary_trait(template.base_traits.deception, variation_factor, &mut rng),
            trustfulness: Self::vary_trait(template.base_traits.trustfulness, variation_factor, &mut rng),
        };
        
        AIPersonality {
            id: format!("{}_variant_{}", template.id, rng.gen::<u32>()),
            name: format!("{}_变体", template.name),
            description: template.description.clone(),
            traits: varied_traits,
        }
    }
    
    /// 创建完全随机的AI性格
    pub fn create_random_personality() -> AIPersonality {
        let mut rng = thread_rng();
        
        let traits = crate::types::PersonalityTraits {
            aggressiveness: rng.gen_range(0.1..0.9),
            logic: rng.gen_range(0.3..0.9),
            deception: rng.gen_range(0.1..0.8),
            trustfulness: rng.gen_range(0.2..0.8),
        };
        
        let personality_type = if traits.logic > 0.7 {
            "理性型"
        } else if traits.aggressiveness > 0.7 {
            "攻击型"
        } else if traits.deception > 0.6 {
            "欺骗型"
        } else {
            "平衡型"
        };
        
        AIPersonality {
            id: format!("random_{}", rng.gen::<u32>()),
            name: format!("{}AI", personality_type),
            description: format!("具有{}特征的AI性格", personality_type),
            traits,
        }
    }
    
    /// 基于游戏角色优化性格
    pub fn optimize_personality_for_role(
        base_personality: &AIPersonality,
        role: &crate::types::Role
    ) -> AIPersonality {
        let mut optimized_traits = base_personality.traits.clone();
        
        match role.role_type {
            crate::types::RoleType::Werewolf => {
                // 狼人需要更强的欺骗能力
                optimized_traits.deception = (optimized_traits.deception + 0.3).min(1.0);
                optimized_traits.trustfulness = (optimized_traits.trustfulness - 0.2).max(0.1);
            }
            crate::types::RoleType::Seer => {
                // 预言家需要更强的逻辑能力
                optimized_traits.logic = (optimized_traits.logic + 0.2).min(1.0);
                optimized_traits.trustfulness = (optimized_traits.trustfulness + 0.1).min(1.0);
            }
            crate::types::RoleType::Witch => {
                // 女巫需要谨慎和逻辑
                optimized_traits.logic = (optimized_traits.logic + 0.15).min(1.0);
                optimized_traits.aggressiveness = (optimized_traits.aggressiveness - 0.1).max(0.1);
            }
            crate::types::RoleType::Hunter => {
                // 猎人可以更加激进
                optimized_traits.aggressiveness = (optimized_traits.aggressiveness + 0.2).min(1.0);
            }
            crate::types::RoleType::Guard => {
                // 守卫需要保护意识
                optimized_traits.trustfulness = (optimized_traits.trustfulness + 0.15).min(1.0);
                optimized_traits.aggressiveness = (optimized_traits.aggressiveness - 0.1).max(0.1);
            }
            crate::types::RoleType::Villager => {
                // 村民保持原有特征，稍微增加逻辑
                optimized_traits.logic = (optimized_traits.logic + 0.1).min(1.0);
            }
        }
        
        AIPersonality {
            id: format!("{}_optimized", base_personality.id),
            name: format!("{}_优化版", base_personality.name),
            description: format!("{} (针对{}角色优化)", 
                base_personality.description, 
                Self::get_role_name(&role.role_type)
            ),
            traits: optimized_traits,
        }
    }
    
    /// 获取性格对应的发言风格
    pub fn get_speech_style_from_personality(personality: &AIPersonality) -> String {
        if personality.traits.logic > 0.7 {
            "逻辑分析型发言".to_string()
        } else if personality.traits.aggressiveness > 0.7 {
            "激进攻击型发言".to_string()
        } else if personality.traits.deception > 0.6 {
            "巧妙欺骗型发言".to_string()
        } else if personality.traits.trustfulness > 0.7 {
            "诚实信任型发言".to_string()
        } else {
            "平衡中性型发言".to_string()
        }
    }
    
    /// 分析性格兼容性
    pub fn analyze_personality_compatibility(
        personality1: &AIPersonality,
        personality2: &AIPersonality
    ) -> CompatibilityAnalysis {
        let trait_differences = (
            (personality1.traits.aggressiveness - personality2.traits.aggressiveness).abs(),
            (personality1.traits.logic - personality2.traits.logic).abs(),
            (personality1.traits.deception - personality2.traits.deception).abs(),
            (personality1.traits.trustfulness - personality2.traits.trustfulness).abs(),
        );
        
        let avg_difference = (trait_differences.0 + trait_differences.1 + 
                              trait_differences.2 + trait_differences.3) / 4.0;
        
        let compatibility_score = 1.0 - avg_difference;
        
        let relationship_type = if compatibility_score > 0.8 {
            "高度兼容"
        } else if compatibility_score > 0.6 {
            "适度兼容"
        } else if compatibility_score > 0.4 {
            "低度兼容"
        } else {
            "不兼容"
        };
        
        CompatibilityAnalysis {
            compatibility_score,
            relationship_type: relationship_type.to_string(),
            main_differences: vec![
                format!("攻击性差异: {:.2}", trait_differences.0),
                format!("逻辑性差异: {:.2}", trait_differences.1),
                format!("欺骗性差异: {:.2}", trait_differences.2),
                format!("信任度差异: {:.2}", trait_differences.3),
            ],
        }
    }
    
    /// 在特征值上添加变化
    fn vary_trait(base_value: f32, variation: f32, rng: &mut impl Rng) -> f32 {
        let change = rng.gen_range(-variation..variation);
        (base_value + change).clamp(0.0, 1.0)
    }
    
    /// 获取角色名称
    fn get_role_name(role_type: &crate::types::RoleType) -> &'static str {
        match role_type {
            crate::types::RoleType::Werewolf => "狼人",
            crate::types::RoleType::Villager => "村民",
            crate::types::RoleType::Seer => "预言家",
            crate::types::RoleType::Witch => "女巫",
            crate::types::RoleType::Hunter => "猎人",
            crate::types::RoleType::Guard => "守卫",
        }
    }
}

/// 兼容性分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityAnalysis {
    pub compatibility_score: f32,
    pub relationship_type: String,
    pub main_differences: Vec<String>,
}

/// 根据难度创建AI性格
pub fn create_personality_by_difficulty(difficulty: &str) -> AIPersonality {
    let templates = PersonalityManager::get_personality_templates();
    
    match difficulty {
        "easy" => {
            // 简单难度：逻辑较弱，容易被识破
            let mut personality = PersonalityManager::create_personality_from_template(
                &templates[3], // 保守谨慎型
                0.3
            );
            personality.traits.logic = 0.3;
            personality.traits.deception = 0.2;
            personality
        }
        "normal" => {
            // 普通难度：平衡的AI
            PersonalityManager::create_personality_from_template(
                &templates[0], // 逻辑分析型
                0.2
            )
        }
        "hard" => {
            // 困难难度：高逻辑，善于欺骗
            let mut personality = PersonalityManager::create_personality_from_template(
                &templates[2], // 狡猾欺骗型
                0.1
            );
            personality.traits.logic = 0.8;
            personality.traits.deception = 0.8;
            personality
        }
        "expert" => {
            // 专家难度：完美AI
            let mut personality = PersonalityManager::create_personality_from_template(
                &templates[4], // 天生领袖
                0.05
            );
            personality.traits.logic = 0.9;
            personality.traits.deception = 0.7;
            personality.traits.aggressiveness = 0.8;
            personality
        }
        _ => PersonalityManager::create_random_personality(),
    }
}
