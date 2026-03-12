import type { RefObject } from 'preact';
import { readableStatus, type ConnectionStatus } from './appState';

export interface AppViewProps {
  status: ConnectionStatus;
  error: string | null;
  canChat: boolean;
  gameContainerRef: RefObject<HTMLDivElement>;
  chatInputRef: RefObject<HTMLInputElement>;
  onChatSubmit: () => void;
}

export function AppView(props: AppViewProps) {
  return (
    <div class="app-shell">
      <div class="game-container">
        <div class="game-surface" ref={props.gameContainerRef} />
        <div class="status-overlay">
          <span class={`status-dot ${props.status}`}></span>
          <span>{readableStatus(props.status)}</span>
        </div>
        <div class="error-toast" hidden={!props.error}>
          {props.error ?? ''}
        </div>
      </div>
      <form
        class="chat-bar"
        onSubmit={(event) => {
          event.preventDefault();
          props.onChatSubmit();
        }}
      >
        <input
          ref={props.chatInputRef}
          type="text"
          maxLength={1024}
          placeholder="Say something…"
          enterKeyHint="send"
          autoComplete="off"
          autoCapitalize="sentences"
          disabled={!props.canChat}
        />
        <button type="submit" disabled={!props.canChat}>Send</button>
      </form>
    </div>
  );
}
