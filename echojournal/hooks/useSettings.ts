import { useState } from 'react';

export interface UserSettings {
  globalTone: string;
  tagTones: Record<string, string>;
}

export const useSettings = () => {
  const [settings, setSettings] = useState<UserSettings>({
    globalTone: "Professional",
    tagTones: {
      "Coding": "Technical & Precise",
      "Career": "Formal & Ambitious",
      "Wellness": "Empathetic & Motivating",
      "Personal": "Sarcastic & Witty"
    }
  });

  return { settings, setSettings };
};