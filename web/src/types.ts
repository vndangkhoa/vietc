export interface KeycapCustomization {
  baseColor: string;
  stemColor: string;
  dragonColor: string;
  material: 'resin_clear' | 'resin_frosted' | 'matte' | 'glass';
  ledColor: string;
  ledIntensity: number; // 0 to 100
  selectedLetter: string; // e.g. "ă", "â", "đ", "ê", "ô", "ơ", "ư", "Sắc (s)", "Huyền (f)"...
  showStem: boolean;
}

export interface TerminalLog {
  id: string;
  type: 'input' | 'system' | 'ime_state' | 'diff';
  text: string;
  timestamp: string;
}

export interface SetupStep {
  id: number;
  title: string;
  description: string;
  command?: string;
  notes?: string;
}

export interface KeycapModel {
  id: string;
  name: string;
  letter: string;
  desc: string;
  rarity: 'Common' | 'Rare' | 'Epic' | 'Legendary';
  price?: string;
  stlUrl: string;
}
