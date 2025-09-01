// 核心类型定义
export interface Player {
  id: string;
  name: string;
  role: Role;
  faction: Faction;
  isAlive: boolean;
  isAI: boolean;
  personality?: AIPersonality;
}

export interface Role {
  type: RoleType;
  faction: Faction;
  description: string;
  canVote: boolean;
  hasNightAction: boolean;
}

export enum RoleType {
  WEREWOLF = 'werewolf',
  VILLAGER = 'villager',
  SEER = 'seer',
  WITCH = 'witch',
  HUNTER = 'hunter',
  GUARD = 'guard'
}

export enum Faction {
  WEREWOLF = 'werewolf',
  VILLAGER = 'villager'
}

export enum GamePhase {
  PREPARATION = 'preparation',
  NIGHT = 'night',
  DAY_DISCUSSION = 'day_discussion',
  VOTING = 'voting',
  LAST_WORDS = 'last_words',
  GAME_OVER = 'game_over'
}

export interface GameState {
  phase: GamePhase;
  day: number;
  players: Player[];
  deadPlayers: Player[];
  votes: VoteRecord[];
  gameConfig: GameConfig;
  winner: Faction | null;
  currentSpeaker?: string;
  timeRemaining?: number;
}

export interface VoteRecord {
  voter: string;
  target: string;
  timestamp: number;
}

export interface GameConfig {
  totalPlayers: number;
  roleDistribution: Record<RoleType, number>;
  discussionTime: number;
  votingTime: number;
  enableVoice: boolean;
}

export interface AIPersonality {
  id: string;
  name: string;
  description: string;
  traits: {
    aggressiveness: number; // 0-1
    logic: number; // 0-1
    deception: number; // 0-1
    trustfulness: number; // 0-1
  };
}

export interface SpeechIntent {
  type: 'accusation' | 'defense' | 'information' | 'strategy' | 'vote';
  target?: string;
  content: string;
  confidence: number;
}

export interface LLMConfig {
  provider: 'openai' | 'anthropic' | 'azure' | 'custom';
  apiKey: string;
  baseUrl: string;
  model: string;
  maxTokens: number;
  temperature: number;
  timeout: number;
  // 实时API相关配置
  useRealtimeApi?: boolean;
  voice?: string;
  inputAudioFormat?: string;
  outputAudioFormat?: string;
  modalities?: string[];
  instructions?: string;
  turnDetection?: TurnDetectionConfig;
}

export interface TurnDetectionConfig {
  detectionType: string; // "server_vad" 或 "none"
  threshold?: number;
  prefixPaddingMs?: number;
  silenceDurationMs?: number;
}

// 实时事件类型
export interface RealtimeEvent {
  eventId?: string;
  eventType: string;
  content: any;
}

// 实时会话配置
export interface RealtimeSessionConfig {
  modalities: string[];
  instructions?: string;
  voice?: string;
  inputAudioFormat?: string;
  outputAudioFormat?: string;
  inputAudioTranscription?: TranscriptionConfig;
  turnDetection?: TurnDetectionConfig;
  tools?: any[];
  toolChoice?: string;
  temperature?: number;
  maxResponseOutputTokens?: number;
}

export interface TranscriptionConfig {
  model: string;
}

export interface GameAction {
  type: string;
  player: string;
  target?: string;
  data?: any;
  timestamp: number;
}

export interface NightAction {
  player: string;
  action: 'kill' | 'check' | 'heal' | 'protect' | 'poison';
  target?: string;
}

// 复盘相关类型
export interface GameReplay {
  game_id: string;
  start_time: string;
  end_time?: string;
  players: Player[];
  game_events: GameEvent[];
  ai_decisions: AIDecision[];
  game_result?: GameResult;
  analysis?: any;
}

export interface GameEvent {
  id: string;
  event_type: GameEventType;
  timestamp: string;
  round: number;
  phase: GamePhase;
  player_id?: string;
  content: string;
}

export interface AIDecision {
  id: string;
  timestamp: string;
  player_id: string;
  decision_type: DecisionType;
  reasoning: string;
  confidence: number;
  alternatives: any[];
}

export type GameEventType = 'GameStart' | 'GameEnd' | 'Speech' | 'Vote' | 'PlayerDeath';
export type DecisionType = 'Speech' | 'Vote' | 'SkillTarget';

export interface GameResult {
  winner: Faction;
  reason: string;
  survivors: string[];
}