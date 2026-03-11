import Phaser from 'phaser';
import type { Identity } from 'spacetimedb';
import { DbConnection } from '../module_bindings';
import type { ChatBubbleV1, UserPositionV1, UserV1 } from '../module_bindings/types';
import '../style.css';
import { WorldScene, type RenderPlayer } from '../game/WorldScene';

type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

/** How long a chat bubble stays visible on the map (ms). */
const BUBBLE_TTL_MS = 8_000;

interface AppState {
  status: ConnectionStatus;
  identity: string | null;
  error: string | null;
  me: UserV1 | null;
  currentPosition: UserPositionV1 | null;
  nearbyPositions: UserPositionV1[];
}

const DEFAULT_STATE: AppState = {
  status: 'connecting',
  identity: null,
  error: null,
  me: null,
  currentPosition: null,
  nearbyPositions: [],
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

  private readonly statusDot: HTMLElement;
  private readonly statusLabel: HTMLElement;
  private readonly chatForm: HTMLFormElement;
  private readonly chatInput: HTMLInputElement;
  private readonly chatButton: HTMLButtonElement;
  private readonly errorToast: HTMLElement;

  /** Local accumulator for chat bubbles (event table rows are ephemeral). */
  private readonly liveBubbles = new Map<string, { bubble: ChatBubbleV1; timer: ReturnType<typeof setTimeout> }>();

  constructor(root: HTMLElement) {
    this.root = root;
    this.root.innerHTML = this.template();

    const gameContainer = this.root.querySelector<HTMLElement>('[data-game-container]')!;
    this.statusDot = this.root.querySelector<HTMLElement>('[data-status-dot]')!;
    this.statusLabel = this.root.querySelector<HTMLElement>('[data-status-label]')!;
    this.chatForm = this.root.querySelector<HTMLFormElement>('[data-chat-form]')!;
    this.chatInput = this.root.querySelector<HTMLInputElement>('[data-chat-input]')!;
    this.chatButton = this.root.querySelector<HTMLButtonElement>('[data-chat-button]')!;
    this.errorToast = this.root.querySelector<HTMLElement>('[data-error-toast]')!;

    this.gameInstance = new Phaser.Game({
      type: Phaser.AUTO,
      parent: gameContainer,
      width: 400,
      height: 600,
      backgroundColor: '#0b1020',
      scale: {
        mode: Phaser.Scale.RESIZE,
      },
      scene: [this.worldScene],
    });

    window.addEventListener('beforeunload', () => {
      this.gameInstance.destroy(true);
    });

    this.chatForm.addEventListener('submit', (e) => {
      e.preventDefault();
      void this.sendChat();
    });

    this.connect();
    this.render();
  }

  private template(): string {
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

    // Event table: rows are inserted then auto-deleted.
    // Accumulate locally so bubbles survive the server-side deletion.
    conn.db.chat_bubble_v1.onInsert((_ctx, bubble) => {
      this.addBubble(bubble);
    });
  }

  private addBubble(bubble: ChatBubbleV1): void {
    const key = this.identityKey(bubble.userId);

    // Clear previous timer for this user if any
    const existing = this.liveBubbles.get(key);
    if (existing) clearTimeout(existing.timer);

    const timer = setTimeout(() => {
      this.liveBubbles.delete(key);
      this.render();
    }, BUBBLE_TTL_MS);

    this.liveBubbles.set(key, { bubble, timer });
    this.render();
  }

  private syncFromConnection(): void {
    if (!this.connection) return;

    const me = [...this.connection.db.vw_user_me_v1.iter()][0] ?? null;
    const currentPosition = [...this.connection.db.vw_world_my_position_v1.iter()][0] ?? null;
    const nearbyPositions = [...this.connection.db.vw_nearby_positions_v1.iter()];

    // Chat bubbles are NOT read from .iter() — they're accumulated via onInsert
    // because chat_bubble_v1 is an event table whose rows auto-delete.
    this.setState({ me, currentPosition, nearbyPositions });
  }

  private setState(patch: Partial<AppState>): void {
    Object.assign(this.state, patch);
    this.render();
  }

  private render(): void {
    this.statusLabel.textContent = this.readableStatus(this.state.status);
    this.statusDot.className = `status-dot ${this.state.status}`;

    this.errorToast.hidden = !this.state.error;
    this.errorToast.textContent = this.state.error ?? '';

    const canChat = this.state.status === 'connected' && this.state.currentPosition !== null;
    this.chatInput.disabled = !canChat;
    this.chatButton.disabled = !canChat;

    this.worldScene.renderPlayers(this.buildRenderPlayers());
  }

  private buildRenderPlayers(): RenderPlayer[] {
    const myUserId = this.state.me?.userId ?? null;

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
        const liveBubble = this.liveBubbles.get(id);
        return {
          id,
          name: this.userLabel(position.userId, isSelf),
          x: position.x,
          y: position.y,
          isSelf,
          chat: liveBubble?.bubble.content,
        } satisfies RenderPlayer;
      })
      .sort((a, b) => a.y - b.y || a.x - b.x || a.name.localeCompare(b.name));
  }

  private async sendChat(): Promise<void> {
    if (!this.connection) return;
    const content = this.chatInput.value.trim();
    if (!content) return;
    await this.connection.reducers.sayV1({ content });
    this.chatInput.value = '';
    this.chatInput.blur();
    this.setState({ error: null });
  }

  private readableStatus(status: ConnectionStatus): string {
    switch (status) {
      case 'connecting': return 'Connecting…';
      case 'connected': return 'Connected';
      case 'disconnected': return 'Disconnected';
      case 'error': return 'Error';
    }
  }

  private userLabel(userId: Identity, isSelf: boolean): string {
    return isSelf ? 'You' : `User ${this.shortIdentity(userId)}`;
  }

  private shortIdentity(identity: Identity): string {
    return this.identityKey(identity).slice(0, 8);
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
}
