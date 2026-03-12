import { enableIosKeyboardOverlay, isIosNativeKeyboardOverlay } from '../native/keyboard';

interface ViewportSnapshot {
  width: number;
  height: number;
  offsetTop: number;
}

const KEYBOARD_THRESHOLD_PX = 120;

function readViewport(): ViewportSnapshot {
  const viewport = window.visualViewport;

  return {
    width: viewport?.width ?? window.innerWidth,
    height: viewport?.height ?? window.innerHeight,
    offsetTop: viewport?.offsetTop ?? 0,
  };
}

export class ViewportLayoutController {
  private baseViewport = readViewport();
  private nativeKeyboardOffset = 0;
  private destroyNativeKeyboardOverlay: (() => void) | null = null;
  private destroyed = false;
  private readonly handleViewportChange = () => {
    if (isIosNativeKeyboardOverlay()) {
      this.applyNativeLayout();
      return;
    }

    this.applyLayout();
  };

  constructor(private readonly root: HTMLElement) {
    if (isIosNativeKeyboardOverlay()) {
      window.addEventListener('resize', this.handleViewportChange);
      this.applyNativeLayout();
      void this.initializeNativeKeyboardOverlay();
      return;
    }

    const viewport = window.visualViewport;

    viewport?.addEventListener('resize', this.handleViewportChange);
    viewport?.addEventListener('scroll', this.handleViewportChange);
    window.addEventListener('resize', this.handleViewportChange);

    this.applyLayout();
  }

  destroy(): void {
    this.destroyed = true;

    if (isIosNativeKeyboardOverlay()) {
      window.removeEventListener('resize', this.handleViewportChange);
      this.destroyNativeKeyboardOverlay?.();
      this.root.style.removeProperty('--app-height');
      this.root.style.removeProperty('--keyboard-offset');
      return;
    }

    const viewport = window.visualViewport;

    viewport?.removeEventListener('resize', this.handleViewportChange);
    viewport?.removeEventListener('scroll', this.handleViewportChange);
    window.removeEventListener('resize', this.handleViewportChange);

    this.root.style.removeProperty('--app-height');
    this.root.style.removeProperty('--keyboard-offset');
  }

  private async initializeNativeKeyboardOverlay(): Promise<void> {
    try {
      const destroyKeyboardOverlay = await enableIosKeyboardOverlay((offset) => {
        this.nativeKeyboardOffset = offset;
        this.applyNativeLayout();
      });

      if (this.destroyed) {
        destroyKeyboardOverlay();
        return;
      }

      this.destroyNativeKeyboardOverlay = destroyKeyboardOverlay;
    } catch (error) {
      console.warn('Failed to initialize iOS keyboard overlay handling', error);
    }
  }

  private applyLayout(): void {
    const current = readViewport();
    const widthChanged = Math.abs(current.width - this.baseViewport.width) > 1;
    const heightGrew = current.height > this.baseViewport.height + 1;

    if (widthChanged || heightGrew) {
      this.baseViewport = current;
    }

    const keyboardOffset = this.keyboardOffset(current);

    this.root.style.setProperty('--app-height', `${Math.round(this.baseViewport.height)}px`);
    this.root.style.setProperty('--keyboard-offset', `${Math.round(keyboardOffset)}px`);
  }

  private applyNativeLayout(): void {
    const current = {
      width: window.innerWidth,
      height: window.innerHeight,
      offsetTop: 0,
    } satisfies ViewportSnapshot;

    const widthChanged = Math.abs(current.width - this.baseViewport.width) > 1;
    const heightChanged = Math.abs(current.height - this.baseViewport.height) > 1;

    if (widthChanged || (this.nativeKeyboardOffset === 0 && heightChanged)) {
      this.baseViewport = current;
    }

    this.root.style.setProperty('--app-height', `${Math.round(this.baseViewport.height)}px`);
    this.root.style.setProperty('--keyboard-offset', `${Math.round(this.nativeKeyboardOffset)}px`);
  }

  private keyboardOffset(current: ViewportSnapshot): number {
    const obscuredHeight = this.baseViewport.height - current.height - current.offsetTop;

    if (obscuredHeight <= KEYBOARD_THRESHOLD_PX) {
      return 0;
    }

    return Math.max(0, obscuredHeight);
  }
}
