use uuid::Uuid;
use rand::{thread_rng, Rng};
use crate::types::{RoleType, Faction};

/// 生成唯一ID
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// 生成随机昵称
pub fn generate_ai_name() -> String {
    let adjectives = [
        \"聪明的\", \"机智的\", \"冷静的\", \"狡猾的\", \"勇敢的\",
        \"沉稳的\", \"敏锐的\", \"谨慎的\", \"果断的\", \"睿智的\"
    ];
    
    let nouns = [
        \"狼\", \"鹰\", \"狐\", \"豹\", \"虎\", \"狮\", \"熊\", \"鹿\", \"鸟\", \"蛇\"
    ];
    
    let mut rng = thread_rng();
    let adj = adjectives[rng.gen_range(0..adjectives.len())];
    let noun = nouns[rng.gen_range(0..nouns.len())];
    
    format!(\"{}{}\", adj, noun)
}

/// 获取角色描述
pub fn get_role_description(role_type: &RoleType) -> String {
    match role_type {
        RoleType::Werewolf => \"狼人：夜晚可以杀死一名玩家，目标是消灭所有好人\".to_string(),
        RoleType::Villager => \"村民：普通村民，没有特殊技能，依靠投票和推理找出狼人\".to_string(),
        RoleType::Seer => \"预言家：每晚可以查验一名玩家的身份\".to_string(),
        RoleType::Witch => \"女巫：拥有一瓶解药和一瓶毒药，可以救人或杀人\".to_string(),
        RoleType::Hunter => \"猎人：被投票出局或被狼人杀死时，可以带走一名玩家\".to_string(),
        RoleType::Guard => \"守卫：每晚可以保护一名玩家，使其免受狼人攻击\".to_string(),
    }
}

/// 获取阵营描述
pub fn get_faction_description(faction: &Faction) -> String {
    match faction {
        Faction::Werewolf => \"狼人阵营：消灭所有好人\".to_string(),
        Faction::Villager => \"好人阵营：找出并消灭所有狼人\".to_string(),
    }
}

/// 计算游戏胜利条件
pub fn check_win_condition(alive_werewolves: usize, alive_villagers: usize) -> Option<Faction> {
    if alive_werewolves == 0 {
        Some(Faction::Villager)
    } else if alive_werewolves >= alive_villagers {
        Some(Faction::Werewolf)
    } else {
        None
    }
}

/// 时间格式化
pub fn format_duration(seconds: u32) -> String {
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    
    if minutes > 0 {
        format!(\"{}分{}秒\", minutes, remaining_seconds)
    } else {
        format!(\"{}秒\", remaining_seconds)
    }
}

/// 洗牌算法
pub fn shuffle<T>(vec: &mut Vec<T>) {
    let mut rng = thread_rng();
    for i in (1..vec.len()).rev() {
        let j = rng.gen_range(0..=i);
        vec.swap(i, j);
    }
}

/// 生成角色分配
pub fn generate_role_distribution(total_players: u8) -> std::collections::HashMap<RoleType, u8> {
    let mut distribution = std::collections::HashMap::new();
    
    match total_players {
        6 => {
            distribution.insert(RoleType::Werewolf, 2);
            distribution.insert(RoleType::Villager, 2);
            distribution.insert(RoleType::Seer, 1);
            distribution.insert(RoleType::Witch, 1);
        }
        8 => {
            distribution.insert(RoleType::Werewolf, 3);
            distribution.insert(RoleType::Villager, 3);
            distribution.insert(RoleType::Seer, 1);
            distribution.insert(RoleType::Witch, 1);
        }
        10 => {
            distribution.insert(RoleType::Werewolf, 3);
            distribution.insert(RoleType::Villager, 4);
            distribution.insert(RoleType::Seer, 1);
            distribution.insert(RoleType::Witch, 1);
            distribution.insert(RoleType::Hunter, 1);
        }
        12 => {
            distribution.insert(RoleType::Werewolf, 4);
            distribution.insert(RoleType::Villager, 4);
            distribution.insert(RoleType::Seer, 1);
            distribution.insert(RoleType::Witch, 1);
            distribution.insert(RoleType::Hunter, 1);
            distribution.insert(RoleType::Guard, 1);
        }
        _ => {
            // 默认配置
            distribution.insert(RoleType::Werewolf, 2);
            distribution.insert(RoleType::Villager, total_players - 2);
        }
    }
    
    distribution
}