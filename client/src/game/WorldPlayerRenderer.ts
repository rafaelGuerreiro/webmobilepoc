import Phaser from 'phaser';
import type { RenderPlayer, WorldLayout } from './worldTypes';

interface PlayerSprites {
  body: Phaser.GameObjects.Arc;
  label: Phaser.GameObjects.Text;
  bubble?: Phaser.GameObjects.Text;
  lastChat?: string;
}

interface PlayerLayout {
  circleRadius: number;
  labelFontSize: number;
  bubbleFontSize: number;
  bubbleWrapWidth: number;
}

const PLAYER_SELF_COLOR = 0x5b8cff;
const PLAYER_OTHER_COLOR = 0x3ddc97;
const PLAYER_RADIUS_RATIO = 0.24;
const LABEL_FONT_RATIO = 0.17;
const BUBBLE_FONT_RATIO = 0.15;
const MIN_PLAYER_RADIUS = 10;
const MIN_LABEL_FONT_SIZE = 11;
const MIN_BUBBLE_FONT_SIZE = 10;
const BUBBLE_WRAP_RATIO = 1.6;
const MIN_BUBBLE_WRAP_WIDTH = 120;
const LABEL_OFFSET_Y = 4;
const BUBBLE_OFFSET_Y = 10;
const CHARACTER_TWEEN_SCALE = 1.14;
const CHARACTER_TWEEN_DURATION_MS = 140;
const UI_FONT_FAMILY = 'Inter, system-ui, sans-serif';

export class WorldPlayerRenderer {
  private readonly sprites = new Map<string, PlayerSprites>();

  constructor(private readonly scene: Phaser.Scene) {}

  render(players: RenderPlayer[], layout: WorldLayout): void {
    const stale = new Set(this.sprites.keys());
    const playerLayout = this.layoutForTile(layout.tileSize);

    for (const player of players) {
      stale.delete(player.id);

      const cx = layout.originX + (player.x + 0.5) * layout.tileSize;
      const cy = layout.originY + (player.y + 0.5) * layout.tileSize;
      const sprite = this.getOrCreateSprites(player, cx, cy, playerLayout);

      sprite.body.setRadius(playerLayout.circleRadius);
      sprite.body.fillColor = this.playerColor(player);
      sprite.body.setPosition(cx, cy);

      sprite.label
        .setFontSize(playerLayout.labelFontSize)
        .setText(player.name)
        .setPosition(cx, cy - playerLayout.circleRadius - LABEL_OFFSET_Y);

      this.updateBubble(sprite, player, cx, cy, playerLayout);
    }

    for (const id of stale) {
      this.destroyPlayer(id);
    }
  }

  destroy(): void {
    for (const id of [...this.sprites.keys()]) {
      this.destroyPlayer(id);
    }
  }

  private layoutForTile(tileSize: number): PlayerLayout {
    return {
      circleRadius: Math.max(MIN_PLAYER_RADIUS, tileSize * PLAYER_RADIUS_RATIO),
      labelFontSize: Math.max(MIN_LABEL_FONT_SIZE, Math.round(tileSize * LABEL_FONT_RATIO)),
      bubbleFontSize: Math.max(MIN_BUBBLE_FONT_SIZE, Math.round(tileSize * BUBBLE_FONT_RATIO)),
      bubbleWrapWidth: Math.max(MIN_BUBBLE_WRAP_WIDTH, tileSize * BUBBLE_WRAP_RATIO),
    };
  }

  private getOrCreateSprites(
    player: RenderPlayer,
    cx: number,
    cy: number,
    layout: PlayerLayout,
  ): PlayerSprites {
    const existing = this.sprites.get(player.id);
    if (existing) {
      return existing;
    }

    const sprite: PlayerSprites = {
      body: this.scene.add.circle(cx, cy, layout.circleRadius, this.playerColor(player)),
      label: this.scene.add.text(cx, cy, player.name, {
        fontFamily: UI_FONT_FAMILY,
        fontSize: `${layout.labelFontSize}px`,
        color: '#ffffff',
        align: 'center',
        stroke: '#08101d',
        strokeThickness: 3,
      }).setOrigin(0.5, 1),
    };

    this.sprites.set(player.id, sprite);
    return sprite;
  }

  private updateBubble(
    sprite: PlayerSprites,
    player: RenderPlayer,
    cx: number,
    cy: number,
    layout: PlayerLayout,
  ): void {
    if (!player.chat) {
      sprite.bubble?.setVisible(false);
      sprite.lastChat = undefined;
      return;
    }

    if (!sprite.bubble) {
      sprite.bubble = this.scene.add.text(cx, cy, '', {
        fontFamily: UI_FONT_FAMILY,
        fontSize: `${layout.bubbleFontSize}px`,
        color: '#0c1424',
        backgroundColor: '#d6e4ff',
        padding: { x: 8, y: 5 },
        wordWrap: { width: layout.bubbleWrapWidth },
        maxLines: 3,
      }).setOrigin(0.5, 1);
    }

    const shouldAnimate = sprite.lastChat !== player.chat || !sprite.bubble.visible;

    sprite.bubble
      .setFontSize(layout.bubbleFontSize)
      .setWordWrapWidth(layout.bubbleWrapWidth)
      .setText(player.chat)
      .setPosition(cx, cy - layout.circleRadius - layout.labelFontSize - BUBBLE_OFFSET_Y)
      .setVisible(true);

    sprite.lastChat = player.chat;
    if (shouldAnimate) {
      this.animateCharacter(sprite);
    }
  }

  private animateCharacter(sprite: PlayerSprites): void {
    this.scene.tweens.killTweensOf(sprite.body);
    sprite.body.setScale(1);
    this.scene.tweens.add({
      targets: sprite.body,
      scaleX: CHARACTER_TWEEN_SCALE,
      scaleY: CHARACTER_TWEEN_SCALE,
      duration: CHARACTER_TWEEN_DURATION_MS,
      yoyo: true,
      ease: 'Quad.easeOut',
    });
  }

  private destroyPlayer(id: string): void {
    const sprite = this.sprites.get(id);
    if (!sprite) {
      return;
    }

    sprite.body.destroy();
    sprite.label.destroy();
    sprite.bubble?.destroy();
    this.sprites.delete(id);
  }

  private playerColor(player: RenderPlayer): number {
    return player.isSelf ? PLAYER_SELF_COLOR : PLAYER_OTHER_COLOR;
  }
}
