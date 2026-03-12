import { Capacitor, registerPlugin } from '@capacitor/core';

type NativeHapticsPlugin = {
  light: () => Promise<void>;
  hard: () => Promise<void>;
};

const NativeHaptics = registerPlugin<NativeHapticsPlugin>('NativeHaptics');

function isIosNative(): boolean {
  return Capacitor.isNativePlatform() && Capacitor.getPlatform() === 'ios';
}

async function trigger(method: keyof NativeHapticsPlugin): Promise<void> {
  if (!isIosNative()) {
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
