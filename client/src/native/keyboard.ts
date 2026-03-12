import { Capacitor } from '@capacitor/core';
import { Keyboard, KeyboardResize } from '@capacitor/keyboard';

function isIosNative(): boolean {
  return Capacitor.isNativePlatform() && Capacitor.getPlatform() === 'ios';
}

export async function enableIosKeyboardOverlay(
  onKeyboardOffsetChange: (offset: number) => void,
): Promise<() => void> {
  if (!isIosNative()) {
    return () => {};
  }

  await Keyboard.setResizeMode({ mode: KeyboardResize.None });
  await Keyboard.setScroll({ isDisabled: true });

  const listeners = [
    await Keyboard.addListener('keyboardWillShow', (info) => {
      onKeyboardOffsetChange(info.keyboardHeight);
    }),
    await Keyboard.addListener('keyboardWillHide', () => {
      onKeyboardOffsetChange(0);
    }),
    await Keyboard.addListener('keyboardDidHide', () => {
      onKeyboardOffsetChange(0);
    }),
  ];

  return () => {
    for (const listener of listeners) {
      void listener.remove();
    }
  };
}

export function isIosNativeKeyboardOverlay(): boolean {
  return isIosNative();
}
