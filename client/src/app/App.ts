import Phaser from 'phaser';
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

export class App {
  private readonly state: AppState = { ...DEFAULT_STATE };
  private connection: DbConnection | null = null;
  private readonly worldScene = new WorldScene();
  private readonly view: AppView;
  private readonly liveBubbles: LiveBubbleStore;
  private readonly gameInstance: Phaser.Game;

  constructor(root: HTMLElement) {
    this.view = new AppView(root, () => {
      void this.sendChat();
    });
    this.liveBubbles = new LiveBubbleStore(BUBBLE_TTL_MS, () => {
      this.render();
    });

    this.gameInstance = new Phaser.Game({
      type: Phaser.AUTO,
      parent: this.view.gameContainer,
      width: 400,
      height: 600,
      backgroundColor: '#0b1020',
      scale: {
        mode: Phaser.Scale.RESIZE,
      },
      scene: [this.worldScene],
    });

    window.addEventListener('beforeunload', () => {
      this.liveBubbles.destroy();
      this.gameInstance.destroy(true);
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
    this.view.render({
      status: this.state.status,
      error: this.state.error,
      canChat: this.state.status === 'connected' && this.state.currentPosition !== null,
    });
    this.worldScene.renderPlayers(buildRenderPlayers(this.state, this.liveBubbles));
  }

  private async sendChat(): Promise<void> {
    if (!this.connection) return;
    const content = this.view.readChatContent();
    if (!content) return;
    await this.connection.reducers.sayV1({ content });
    this.view.clearChatInput();
    this.setState({ error: null });
  }
}
