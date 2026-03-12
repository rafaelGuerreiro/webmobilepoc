import type { ChatBubbleV1 } from '../module_bindings/types';

interface LiveBubbleEntry {
  bubble: ChatBubbleV1;
  timer: ReturnType<typeof setTimeout>;
}

export class LiveBubbleStore {
  private readonly bubbles = new Map<string, LiveBubbleEntry>();

  constructor(
    private readonly ttlMs: number,
    private readonly onChange: () => void,
  ) {}

  set(identityKey: string, bubble: ChatBubbleV1): void {
    const existing = this.bubbles.get(identityKey);
    if (existing) {
      clearTimeout(existing.timer);
    }

    const timer = setTimeout(() => {
      this.bubbles.delete(identityKey);
      this.onChange();
    }, this.ttlMs);

    this.bubbles.set(identityKey, { bubble, timer });
    this.onChange();
  }

  contentFor(identityKey: string): string | undefined {
    return this.bubbles.get(identityKey)?.bubble.content;
  }

  destroy(): void {
    for (const entry of this.bubbles.values()) {
      clearTimeout(entry.timer);
    }
    this.bubbles.clear();
  }
}
