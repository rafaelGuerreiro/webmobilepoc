import Phaser from 'phaser';
import { createRef, h, render as renderPreact } from 'preact';
import { DbConnection } from '../module_bindings';
import { triggerHardHaptic, triggerLightHaptic } from '../native/haptics';
import '../style.css';
import { WorldScene } from '../game/WorldScene';
import { AppView } from './AppView';
import { BUBBLE_TTL_MS, TOKEN_KEY } from './appConfig';
import { LiveBubbleStore } from './LiveBubbleStore';
import { DEFAULT_STATE, type AppState } from './appState';
import { buildRenderPlayers, formatIdentity, identityKey } from './renderPlayers';
import { connectApp } from './spacetimedb';
import { ViewportLayoutController } from './ViewportLayoutController';

export class App {
  private readonly root: HTMLElement;
  private readonly state: AppState = { ...DEFAULT_STATE };
  private connection: DbConnection | null = null;
  private readonly worldScene = new WorldScene();
  private readonly liveBubbles: LiveBubbleStore;
  private readonly viewportLayout: ViewportLayoutController;
  private readonly gameContainerRef = createRef<HTMLDivElement>();
  private readonly chatInputRef = createRef<HTMLInputElement>();
  private readonly handleChatSubmit = () => {
    void this.sendChat();
  };
  private readonly gameInstance: Phaser.Game;

  constructor(root: HTMLElement) {
    this.root = root;
    this.viewportLayout = new ViewportLayoutController(root);
    this.liveBubbles = new LiveBubbleStore(BUBBLE_TTL_MS, () => {
      this.render();
    });
    this.renderView();

    const gameContainer = this.gameContainerRef.current;
    if (!gameContainer) {
      throw new Error('Missing Phaser game container');
    }

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
      this.viewportLayout.destroy();
      this.liveBubbles.destroy();
      this.gameInstance.destroy(true);
      renderPreact(null, this.root);
    });

    this.connect();
    this.render();
  }

  private connect(): void {
    const savedToken = window.localStorage.getItem(TOKEN_KEY) ?? undefined;
    this.setState({ status: 'connecting', error: null });

    this.connection = connectApp(savedToken, {
      onConnected: (identity, token) => {
        window.localStorage.setItem(TOKEN_KEY, token);
        this.setState({ status: 'connected', identity: formatIdentity(identity), error: null });
        triggerHardHaptic();
      },
      onConnectionError: (error) => {
        this.setState({ status: 'error', error: error.message });
      },
      onDisconnected: (error) => {
        this.setState({ status: error ? 'error' : 'disconnected', error: error?.message ?? null });
      },
      onStateChanged: (state) => {
        this.setState(state);
      },
      onBubble: (bubble) => {
        this.liveBubbles.set(identityKey(bubble.userId), bubble);
        triggerLightHaptic();
      },
    });
  }

  private setState(patch: Partial<AppState>): void {
    Object.assign(this.state, patch);
    this.render();
  }

  private render(): void {
    this.renderView();
    this.worldScene.renderPlayers(buildRenderPlayers(this.state, this.liveBubbles));
  }

  private async sendChat(): Promise<void> {
    if (!this.connection) return;
    const content = this.chatInputRef.current?.value.trim() ?? '';
    if (!content) return;

    await this.connection.reducers.sayV1({ content });
    if (this.chatInputRef.current) {
      this.chatInputRef.current.value = '';
    }
    this.setState({ error: null });

    requestAnimationFrame(() => {
      this.chatInputRef.current?.focus({ preventScroll: true });
    });
  }

  private renderView(): void {
    renderPreact(
      h(AppView, {
        status: this.state.status,
        error: this.state.error,
        canChat: this.state.status === 'connected' && this.state.currentPosition !== null,
        gameContainerRef: this.gameContainerRef,
        chatInputRef: this.chatInputRef,
        onChatSubmit: this.handleChatSubmit,
      }),
      this.root,
    );
  }
}
