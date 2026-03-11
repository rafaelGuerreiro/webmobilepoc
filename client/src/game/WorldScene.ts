import Phaser from 'phaser';

export interface RenderPlayer {
  id: string;
  name: string;
  x: number;
  y: number;
  isSelf: boolean;
  chat?: string;
}

interface PlayerSprites {
  body: Phaser.GameObjects.Arc;
  label: Phaser.GameObjects.Text;
  bubble?: Phaser.GameObjects.Text;
}

const TILE_SIZE = 72;
const ORIGIN_OFFSET = 88;

export class WorldScene extends Phaser.Scene {
  private readonly sprites = new Map<string, PlayerSprites>();
  private grid?: Phaser.GameObjects.Graphics;

  constructor() {
    super('world-scene');
  }

  create(): void {
    this.grid = this.add.graphics();
    this.drawGrid();
    this.scale.on('resize', () => this.drawGrid());
  }

  renderPlayers(players: RenderPlayer[]): void {
    const stale = new Set(this.sprites.keys());

    for (const player of players) {
      stale.delete(player.id);
      let sprite = this.sprites.get(player.id);
      if (!sprite) {
        sprite = {
          body: this.add.circle(0, 0, 18, player.isSelf ? 0x5b8cff : 0x3ddc97),
          label: this.add.text(0, 0, player.name, {
            fontFamily: 'Inter, sans-serif',
            fontSize: '16px',
            color: '#ffffff',
            align: 'center',
            stroke: '#08101d',
            strokeThickness: 4,
          }).setOrigin(0.5, 1),
        };
        this.sprites.set(player.id, sprite);
      }

      sprite.body.fillColor = player.isSelf ? 0x5b8cff : 0x3ddc97;
      const screenX = ORIGIN_OFFSET + player.x * TILE_SIZE;
      const screenY = ORIGIN_OFFSET + player.y * TILE_SIZE;

      sprite.body.setPosition(screenX, screenY);
      sprite.label.setText(player.name).setPosition(screenX, screenY - 22);

      if (player.chat) {
        sprite.bubble ??= this.add.text(0, 0, '', {
          fontFamily: 'Inter, sans-serif',
          fontSize: '14px',
          color: '#0c1424',
          backgroundColor: '#d6e4ff',
          padding: { x: 10, y: 6 },
          wordWrap: { width: 220 },
        }).setOrigin(0.5, 1);
        sprite.bubble.setText(player.chat).setPosition(screenX, screenY - 54).setVisible(true);
      } else if (sprite.bubble) {
        sprite.bubble.setVisible(false);
      }
    }

    for (const id of stale) {
      const sprite = this.sprites.get(id);
      sprite?.body.destroy();
      sprite?.label.destroy();
      sprite?.bubble?.destroy();
      this.sprites.delete(id);
    }
  }

  private drawGrid(): void {
    if (!this.grid) return;

    const width = this.scale.width;
    const height = this.scale.height;
    this.grid.clear();
    this.grid.fillStyle(0x0d1728, 1);
    this.grid.fillRect(0, 0, width, height);
    this.grid.lineStyle(1, 0x1f314d, 0.7);

    for (let x = ORIGIN_OFFSET; x < width; x += TILE_SIZE) {
      this.grid.lineBetween(x, 0, x, height);
    }
    for (let y = ORIGIN_OFFSET; y < height; y += TILE_SIZE) {
      this.grid.lineBetween(0, y, width, y);
    }

    this.grid.fillStyle(0x5b8cff, 1);
    this.grid.fillCircle(ORIGIN_OFFSET, ORIGIN_OFFSET, 5);
  }
}
