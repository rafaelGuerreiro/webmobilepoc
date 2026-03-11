import Phaser from 'phaser';
import type { Identity } from 'spacetimedb';
import { DbConnection } from '../module_bindings';
import type { ChatBubbleV1, UserPositionV1, UserV1 } from '../module_bindings/types';
import '../style.css';
import { WorldScene, type RenderPlayer } from '../game/WorldScene';

type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface AppState {
  status: ConnectionStatus;
  identity: string | null;
  error: string | null;
  me: UserV1 | null;
  currentPosition: UserPositionV1 | null;
  nearbyPositions: UserPositionV1[];
  chatBubbles: ChatBubbleV1[];
}

const DEFAULT_STATE: AppState = {
  status: 'connecting',
  identity: null,
  error: null,
  me: null,
  currentPosition: null,
  nearbyPositions: [],
  chatBubbles: [],
};

const HOST = import.meta.env.VITE_STDB_URI ?? 'https://maincloud.spacetimedb.com';
const DATABASE = import.meta.env.VITE_STDB_NAME ?? 'webmobiledb';
const TOKEN_KEY = `${HOST}/${DATABASE}/auth_token`;

export class App {
  private readonly root: HTMLElement;
  private readonly state: AppState = { ...DEFAULT_STATE };
  private connection: DbConnection | null = null;
  private readonly worldScene = new WorldScene();
  private readonly gameInstance: Phaser.Game;

  private readonly statusValue: HTMLElement;
  private readonly statusDot: HTMLElement;
  private readonly identityValue: HTMLElement;
  private readonly presencePanel: HTMLElement;
  private readonly playersPanel: HTMLElement;
  private readonly chatLog: HTMLElement;
  private readonly chatForm: HTMLFormElement;
  private readonly chatInput: HTMLInputElement;
  private readonly errorBanner: HTMLElement;

  constructor(root: HTMLElement) {
    this.root = root;
    this.root.innerHTML = this.template();

    const stage = this.root.querySelector<HTMLElement>('[data-game-stage]');
    this.statusValue = this.root.querySelector<HTMLElement>('[data-status-value]')!;
    this.statusDot = this.root.querySelector<HTMLElement>('[data-status-dot]')!;
    this.identityValue = this.root.querySelector<HTMLElement>('[data-identity-value]')!;
    this.presencePanel = this.root.querySelector<HTMLElement>('[data-presence-panel]')!;
    this.playersPanel = this.root.querySelector<HTMLElement>('[data-players-panel]')!;
    this.chatLog = this.root.querySelector<HTMLElement>('[data-chat-log]')!;
    this.chatForm = this.root.querySelector<HTMLFormElement>('[data-chat-form]')!;
    this.chatInput = this.root.querySelector<HTMLInputElement>('[data-chat-input]')!;
    this.errorBanner = this.root.querySelector<HTMLElement>('[data-error-banner]')!;

    if (!stage) {
      throw new Error('Missing game stage container');
    }

    this.gameInstance = new Phaser.Game({
      type: Phaser.AUTO,
      parent: stage,
      width: 720,
      height: 720,
      backgroundColor: '#0d1728',
      scale: {
        mode: Phaser.Scale.FIT,
        autoCenter: Phaser.Scale.CENTER_BOTH,
      },
      scene: [this.worldScene],
    });

    window.addEventListener('beforeunload', () => {
      this.gameInstance.destroy(true);
    });

    this.chatForm.addEventListener('submit', (event) => {
      event.preventDefault();
      void this.sendChat();
    });

    this.connect();
    this.render();
  }

  private template(): string {
    return `
      <div class="app-shell">
        <header class="topbar panel">
          <div>
            <h1>WebMobile</h1>
            <p>Anonymous users are placed automatically and can chat in place.</p>
          </div>
          <div class="stack" style="justify-items:end;">
            <div class="status-pill">
              <span class="status-dot" data-status-dot></span>
              <span data-status-value>Connecting…</span>
            </div>
            <div class="muted">Identity: <span data-identity-value>—</span></div>
          </div>
        </header>

        <div class="content-grid">
          <section class="panel game-panel">
            <div class="game-stage" data-game-stage></div>
          </section>

          <aside class="sidebar">
            <section class="panel section">
              <h2>Presence</h2>
              <div class="stack" data-presence-panel></div>
            </section>

            <section class="panel section">
              <h2>Visible users</h2>
              <div class="stack" data-players-panel></div>
            </section>

            <section class="panel section">
              <h2>Chat</h2>
              <div class="chat-log" data-chat-log></div>
              <form class="chat-form" data-chat-form>
                <input data-chat-input type="text" maxlength="1024" placeholder="Say something…" />
                <button type="submit">Send</button>
              </form>
              <div class="error-banner" data-error-banner hidden></div>
            </section>
          </aside>
        </div>
      </div>
    `;
  }

  private connect(): void {
    const savedToken = window.localStorage.getItem(TOKEN_KEY) ?? undefined;
    this.setState({ status: 'connecting', error: null });

    this.connection = DbConnection.builder()
      .withUri(HOST)
      .withDatabaseName(DATABASE)
      .withToken(savedToken)
      .onConnect((conn, identity, token) => {
        window.localStorage.setItem(TOKEN_KEY, token);
        this.setState({ status: 'connected', identity: this.formatIdentity(identity), error: null });
        this.registerSubscriptions(conn);
        this.registerRowObservers(conn);
      })
      .onConnectError((_ctx, error) => {
        this.setState({ status: 'error', error: error.message });
      })
      .onDisconnect((_ctx, error) => {
        this.setState({ status: error ? 'error' : 'disconnected', error: error?.message ?? null });
      })
      .build();
  }

  private registerSubscriptions(conn: DbConnection): void {
    conn.subscriptionBuilder()
      .onApplied(() => {
        this.syncFromConnection();
      })
      .subscribe([
        'SELECT * FROM vw_user_me_v1',
        'SELECT * FROM vw_world_my_position_v1',
        'SELECT * FROM vw_nearby_positions_v1',
        'SELECT * FROM chat_bubble_v1',
      ]);
  }

  private registerRowObservers(conn: DbConnection): void {
    const refresh = (): void => {
      this.syncFromConnection();
    };

    conn.db.vw_user_me_v1.onInsert(() => refresh());
    conn.db.vw_user_me_v1.onDelete(() => refresh());
    conn.db.vw_user_me_v1.onUpdate(() => refresh());
    conn.db.vw_world_my_position_v1.onInsert(() => refresh());
    conn.db.vw_world_my_position_v1.onDelete(() => refresh());
    conn.db.vw_world_my_position_v1.onUpdate(() => refresh());
    conn.db.vw_nearby_positions_v1.onInsert(() => refresh());
    conn.db.vw_nearby_positions_v1.onDelete(() => refresh());
    conn.db.vw_nearby_positions_v1.onUpdate(() => refresh());
    conn.db.chat_bubble_v1.onInsert(() => refresh());
  }

  private syncFromConnection(): void {
    if (!this.connection) return;

    const me = [...this.connection.db.vw_user_me_v1.iter()][0] ?? null;
    const currentPosition = [...this.connection.db.vw_world_my_position_v1.iter()][0] ?? null;
    const nearbyPositions = [...this.connection.db.vw_nearby_positions_v1.iter()];
    const chatBubbles = [...this.connection.db.chat_bubble_v1.iter()].slice(-20);

    this.setState({
      me,
      currentPosition,
      nearbyPositions,
      chatBubbles,
    });
  }

  private setState(patch: Partial<AppState>): void {
    Object.assign(this.state, patch);
    this.render();
  }

  private render(): void {
    this.statusValue.textContent = this.readableStatus(this.state.status);
    this.statusDot.className = `status-dot ${this.state.status === 'connected' ? 'connected' : ''} ${this.state.status === 'error' ? 'error' : ''}`;
    this.identityValue.textContent = this.state.identity ?? '—';
    this.errorBanner.hidden = !this.state.error;
    this.errorBanner.textContent = this.state.error ?? '';

    const canChat = this.state.status === 'connected' && this.state.currentPosition !== null;
    this.chatInput.disabled = !canChat;
    this.chatForm.querySelector('button')!.toggleAttribute('disabled', !canChat);

    this.renderPresencePanel();
    this.renderPlayersPanel();
    this.renderChatLog();
    this.renderWorld();
  }

  private renderPresencePanel(): void {
    const me = this.state.me;
    const position = this.state.currentPosition;

    if (!me || !position) {
      this.presencePanel.innerHTML = '<p class="muted">Waiting for the server to place you on the grid.</p>';
      return;
    }

    this.presencePanel.innerHTML = `
      <article class="presence-card">
        <header>
          <div>
            <strong>You</strong>
            <div class="muted">Anonymous user ${this.escapeHtml(this.shortIdentity(me.userId))}</div>
          </div>
          <span class="muted">(${position.x}, ${position.y})</span>
        </header>
      </article>
      <p class="muted">Your position is chosen by the server and stays fixed while connected.</p>
    `;
  }

  private renderPlayersPanel(): void {
    const players = this.buildRenderPlayers();
    if (players.length === 0) {
      this.playersPanel.innerHTML = '<p class="muted">No visible users yet.</p>';
      return;
    }

    this.playersPanel.innerHTML = players
      .map((player) => `
        <article class="player-card">
          <header>
            <strong>${this.escapeHtml(player.name)}</strong>
            <span class="muted">(${player.x}, ${player.y})</span>
          </header>
          ${player.chat ? `<p>${this.escapeHtml(player.chat)}</p>` : ''}
        </article>
      `)
      .join('');
  }

  private renderChatLog(): void {
    if (this.state.chatBubbles.length === 0) {
      this.chatLog.innerHTML = '<p class="muted">Chat messages will appear here.</p>';
      return;
    }

    const myUserId = this.state.me?.userId ?? null;

    this.chatLog.innerHTML = this.state.chatBubbles
      .map((bubble) => {
        const isSelf = myUserId ? this.sameIdentity(bubble.userId, myUserId) : false;
        return `
          <article class="message-card">
            <header>
              <strong>${this.escapeHtml(this.userLabel(bubble.userId, isSelf))}</strong>
              <span class="muted">(${bubble.x}, ${bubble.y})</span>
            </header>
            <p>${this.escapeHtml(bubble.content)}</p>
          </article>
        `;
      })
      .join('');
    this.chatLog.scrollTop = this.chatLog.scrollHeight;
  }

  private renderWorld(): void {
    this.worldScene.renderPlayers(this.buildRenderPlayers());
  }

  private buildRenderPlayers(): RenderPlayer[] {
    const myUserId = this.state.me?.userId ?? null;
    const bubbleById = new Map<string, ChatBubbleV1>();

    for (const bubble of this.state.chatBubbles) {
      bubbleById.set(this.identityKey(bubble.userId), bubble);
    }

    const positionsById = new Map<string, UserPositionV1>();
    for (const position of this.state.nearbyPositions) {
      positionsById.set(this.identityKey(position.userId), position);
    }
    if (this.state.currentPosition) {
      positionsById.set(this.identityKey(this.state.currentPosition.userId), this.state.currentPosition);
    }

    return [...positionsById.values()]
      .map((position) => {
        const isSelf = myUserId ? this.sameIdentity(position.userId, myUserId) : false;
        const id = this.identityKey(position.userId);
        return {
          id,
          name: this.userLabel(position.userId, isSelf),
          x: position.x,
          y: position.y,
          isSelf,
          chat: bubbleById.get(id)?.content,
        } satisfies RenderPlayer;
      })
      .sort((left, right) => left.y - right.y || left.x - right.x || left.name.localeCompare(right.name));
  }

  private async sendChat(): Promise<void> {
    if (!this.connection) return;
    const content = this.chatInput.value.trim();
    if (!content) return;
    await this.connection.reducers.sayV1({ content });
    this.chatInput.value = '';
    this.setState({ error: null });
  }

  private readableStatus(status: ConnectionStatus): string {
    switch (status) {
      case 'connecting':
        return 'Connecting';
      case 'connected':
        return 'Connected';
      case 'disconnected':
        return 'Disconnected';
      case 'error':
        return 'Connection error';
    }
  }

  private userLabel(userId: Identity, isSelf: boolean): string {
    return isSelf ? 'You' : `User ${this.shortIdentity(userId)}`;
  }

  private shortIdentity(identity: Identity): string {
    const hex = this.identityKey(identity);
    return hex.slice(0, 8);
  }

  private sameIdentity(left: Identity, right: Identity): boolean {
    return this.identityKey(left) === this.identityKey(right);
  }

  private identityKey(identity: Identity): string {
    return identity.toHexString();
  }

  private formatIdentity(identity: Identity): string {
    return this.identityKey(identity);
  }

  private escapeHtml(value: string): string {
    return value
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }
}
