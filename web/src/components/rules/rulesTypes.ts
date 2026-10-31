import type { LocalRuleRecord, Language } from '../../types';

export type FilterSubTab = 'categories' | 'rules' | 'keywords';

export interface RulesTabProps {
  rules: LocalRuleRecord[];
  lang: Language;
  onRefresh: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export interface ParsedImportRule {
  rule_id: string;
  rule_type: 'domain' | 'application';
  reason: string;
  quota?: number;
  enabled: boolean;
}

export interface MergedDomainItem {
  id: string;
  domain: string;
  source: 'local' | 'cloud';
  enabled: boolean;
}

export const STANDARD_CATEGORIES = [
  {
    id: 'adult',
    name: 'Nội Dung Người Lớn (18+)',
    nameEn: 'Adult & Explicit (18+)',
    icon: '🔞',
    desc: 'Khiêu dâm, đồi trụy, hình ảnh và nội dung không phù hợp với lứa tuổi.',
    descEn: 'Pornography, explicit imagery, and age-inappropriate content.',
    seeds: ['pornhub.com', 'xvideos.com', 'xnxx.com', 'redtube.com', 'onlyfans.com'],
  },
  {
    id: 'gambling',
    name: 'Cờ Bạc & Cá Cược Trực Tuyến',
    nameEn: 'Gambling & Betting',
    icon: '🎲',
    desc: 'Casino trực tuyến, cá độ thể thao, máy đánh bạc và cờ bạc ăn tiền.',
    descEn: 'Online casinos, sports betting, slot machines, and wagering.',
    seeds: ['bet365.com', '1xbet.com', 'w88.com', 'fun88.com', 'fb88.com'],
  },
  {
    id: 'violence',
    name: 'Bạo Lực & Kinh Dị Ghê Rợn',
    nameEn: 'Violence & Gore',
    icon: '⚔️',
    desc: 'Cảnh máu me, bạo lực tàn bạo, kích động hành vi nguy hiểm.',
    descEn: 'Gore, brutal violence, and self-destructive dangerous stunts.',
    seeds: ['bestgore.com', 'crazyshit.com', 'liveleak.com', 'kaotic.com'],
  },
  {
    id: 'social',
    name: 'Mạng Xã Hội & Video Ngắn',
    nameEn: 'Social Media & Feeds',
    icon: '💬',
    desc: 'TikTok, Facebook, Instagram... Giúp kiểm soát xao nhãng trong giờ học.',
    descEn: 'TikTok, Facebook, Instagram... Minimize distraction during study hours.',
    seeds: ['tiktok.com', 'facebook.com', 'instagram.com', 'x.com', 'reddit.com'],
  },
  {
    id: 'gaming',
    name: 'Game & Nền Tảng Trò Chơi',
    nameEn: 'Gaming Platforms',
    icon: '🎮',
    desc: 'Steam, Roblox, Riot, Epic Games... Quản lý thời gian chơi game trực tuyến.',
    descEn: 'Steam, Roblox, Riot, Epic Games... Online gaming and store platforms.',
    seeds: ['roblox.com', 'steampowered.com', 'steamcommunity.com', 'riotgames.com'],
  },
];

export const SYSTEM_KEYWORD_CATEGORIES = [
  {
    id: 'self_harm',
    name: 'Tự Hại / Nguy Hiểm',
    nameEn: 'Self-Harm / Suicide Risk',
    desc: 'Nhận diện dấu hiệu khủng hoảng tâm lý, tự tử, tự làm đau bản thân.',
    descEn: 'Detect depression, self-harm cues, and psychological crises.',
    icon: '💀',
    severity: 'critical',
    samples: ['tu tu', 'tu sat', 'rach tay', 'muon chet', 'suicide'],
  },
  {
    id: 'adult',
    name: 'Người Lớn / 18+',
    nameEn: 'Adult & NSFW Words',
    desc: 'Từ khóa nhạy cảm, khiêu dâm, đồi trụy và tình dục không lành mạnh.',
    descEn: 'Sexually explicit words, adult search terms, and NSFW phrases.',
    icon: '🔥',
    severity: 'high',
    samples: ['sex', 'phim sex', 'porn', 'xxx', 'hentai', 'jav'],
  },
  {
    id: 'bullying',
    name: 'Bắt Nạt / Đe Dọa',
    nameEn: 'Cyberbullying & Threats',
    desc: 'Đe dọa tính mạng, bạo lực ngôn từ, uy hiếp tinh thần bạn bè.',
    descEn: 'Cyberbullying, death threats, school intimidation, and harassment.',
    icon: '⚠️',
    severity: 'high',
    samples: ['chet di', 'tao giet', 'danh chet', 'con cho'],
  },
  {
    id: 'profanity',
    name: 'Chửi Bậy / Thô Tục',
    nameEn: 'Profanity & Vulgar Slang',
    desc: 'Ngôn từ thô tục, chửi thề xúc phạm trong văn hóa mạng.',
    descEn: 'Profane language, vulgar insults, and offensive slang.',
    icon: '🛡️',
    severity: 'medium',
    samples: ['dm', 'dmm', 'vcl', 'clgt', 'dit me', 'du ma'],
  },
];
