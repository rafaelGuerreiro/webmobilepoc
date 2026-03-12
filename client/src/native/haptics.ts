import { Capacitor, registerPlugin } from '@capacitor/core';

type NativeHapticsPlugin = {
  light: () => Promise<void>;
  hard: () => Promise<void>;
};

const NativeHaptics = registerPlugin<NativeHapticsPlugin>('NativeHaptics');

function isNativeHapticsPlatform(): boolean {
  if (!Capacitor.isNativePlatform()) {
    return false;
  }

  const platform = Capacitor.getPlatform();
  return platform === 'ios' || platform === 'android';
}

async function trigger(method: keyof NativeHapticsPlugin): Promise<void> {
  if (!isNativeHapticsPlatform()) {
    return;
  }

  try {
    await NativeHaptics[method]();
  } catch (error) {
    console.warn(`Native haptics ${method} failed`, error);
  }
}

export function triggerLightHaptic(): void {
  void trigger('light');
}

export function triggerHardHaptic(): void {
  void trigger('hard');
}
