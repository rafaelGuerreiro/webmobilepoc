import Phaser from 'phaser';
import { WorldGridRenderer } from './WorldGridRenderer';
import { createWorldLayout } from './worldLayout';
import { WorldPlayerRenderer } from './WorldPlayerRenderer';
import type { RenderPlayer } from './worldTypes';

export class WorldScene extends Phaser.Scene {
  private readonly gridRenderer = new WorldGridRenderer(this);
  private readonly playerRenderer = new WorldPlayerRenderer(this);
  private readonly handleResize = () => this.redraw();
  private pendingPlayers: RenderPlayer[] = [];
  private isReady = false;

  constructor() {
    super('world-scene');
  }

  create(): void {
    this.isReady = true;
    this.gridRenderer.create();
    this.scale.on('resize', this.handleResize);
    this.events.once(Phaser.Scenes.Events.DESTROY, () => {
      this.isReady = false;
      this.scale.off('resize', this.handleResize);
      this.playerRenderer.destroy();
    });
    this.redraw();
  }

  renderPlayers(players: RenderPlayer[]): void {
    this.pendingPlayers = players;
    if (this.isReady) {
      this.redraw();
    }
  }

  private redraw(): void {
    const layout = createWorldLayout(this.pendingPlayers, this.scale.width, this.scale.height);
    this.gridRenderer.draw(layout);
    this.playerRenderer.render(this.pendingPlayers, layout);
  }
}

export type { RenderPlayer } from './worldTypes';
