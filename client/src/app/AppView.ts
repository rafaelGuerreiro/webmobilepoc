import { readableStatus, type ConnectionStatus } from './appState';

interface AppViewModel {
  status: ConnectionStatus;
  error: string | null;
  canChat: boolean;
}

function template(): string {
  return `
    <div class="app-shell">
      <div class="game-container" data-game-container>
        <div class="status-overlay">
          <span class="status-dot" data-status-dot></span>
          <span data-status-label>Connecting…</span>
        </div>
        <div class="error-toast" data-error-toast hidden></div>
      </div>
      <form class="chat-bar" data-chat-form>
        <input
          data-chat-input
          type="text"
          maxlength="1024"
          placeholder="Say something…"
          enterkeyhint="send"
          autocomplete="off"
          autocapitalize="sentences"
        />
        <button type="submit" data-chat-button disabled>Send</button>
      </form>
    </div>
  `;
}

function requiredElement<T extends HTMLElement>(root: ParentNode, selector: string): T {
  const element = root.querySelector<T>(selector);
  if (!element) {
    throw new Error(`Missing required element: ${selector}`);
  }
  return element;
}

export class AppView {
  readonly gameContainer: HTMLElement;

  private readonly statusDot: HTMLElement;
  private readonly statusLabel: HTMLElement;
  private readonly chatInput: HTMLInputElement;
  private readonly chatButton: HTMLButtonElement;
  private readonly errorToast: HTMLElement;

  constructor(root: HTMLElement, onChatSubmit: () => void) {
    root.innerHTML = template();

    this.gameContainer = requiredElement<HTMLElement>(root, '[data-game-container]');
    this.statusDot = requiredElement<HTMLElement>(root, '[data-status-dot]');
    this.statusLabel = requiredElement<HTMLElement>(root, '[data-status-label]');
    const chatForm = requiredElement<HTMLFormElement>(root, '[data-chat-form]');
    this.chatInput = requiredElement<HTMLInputElement>(root, '[data-chat-input]');
    this.chatButton = requiredElement<HTMLButtonElement>(root, '[data-chat-button]');
    this.errorToast = requiredElement<HTMLElement>(root, '[data-error-toast]');

    chatForm.addEventListener('submit', (event) => {
      event.preventDefault();
      onChatSubmit();
    });
  }

  render(model: AppViewModel): void {
    this.statusLabel.textContent = readableStatus(model.status);
    this.statusDot.className = `status-dot ${model.status}`;
    this.errorToast.hidden = !model.error;
    this.errorToast.textContent = model.error ?? '';
    this.chatInput.disabled = !model.canChat;
    this.chatButton.disabled = !model.canChat;
  }

  readChatContent(): string {
    return this.chatInput.value.trim();
  }

  clearChatInput(): void {
    this.chatInput.value = '';
    this.chatInput.blur();
  }
}
