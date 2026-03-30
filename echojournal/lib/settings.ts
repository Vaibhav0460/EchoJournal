import { writeTextFile, readTextFile, exists, BaseDirectory } from '@tauri-apps/plugin-fs';

export interface UserSettings {
  userName: string;
  defaultTone: string;
  tagTones: Record<string, string>;
}

const SETTINGS_FILE = 'settings.json';

export const saveSettings = async (settings: UserSettings) => {
  await writeTextFile(SETTINGS_FILE, JSON.stringify(settings, null, 2), {
    baseDir: BaseDirectory.Document,
  });
};

export const loadSettings = async (): Promise<UserSettings> => {
  const path = SETTINGS_FILE;
  if (await exists(path, { baseDir: BaseDirectory.Document })) {
    const content = await readTextFile(path, { baseDir: BaseDirectory.Document });
    return JSON.parse(content);
  }
  // Default fallback if file doesn't exist
  return {
    userName: "Explorer",
    defaultTone: "Professional",
    tagTones: {
      "Coding": "Technical & precise",
      "Career": "Formal & ambitious",
      "Wellness": "Empathetic & soft",
      "Personal": "Sarcastic & witty",
      "Finance": "Calculated & serious",
      "Social": "Friendly & upbeat"
    }
  };
};