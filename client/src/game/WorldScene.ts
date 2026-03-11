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

// Minimum visible grid (covers the first 10 diagonal-sweep positions)
const MIN_COLS = 4;
const MIN_ROWS = 4;

export class WorldScene extends Phaser.Scene {
  private readonly sprites = new Map<string, PlayerSprites>();
  private grid?: Phaser.GameObjects.Graphics;
  private pendingPlayers: RenderPlayer[] = [];

  constructor() {
    super('world-scene');
  }

  create(): void {
    this.grid = this.add.graphics();
    this.scale.on('resize', () => this.redraw());
    this.redraw();
  }

  renderPlayers(players: RenderPlayer[]): void {
    this.pendingPlayers = players;
    if (this.grid) {
      this.redraw();
    }
  }

  private redraw(): void {
    const players = this.pendingPlayers;
    const width = this.scale.width;
    const height = this.scale.height;

    // Grid extent adapts to visible players
    let cols = MIN_COLS;
    let rows = MIN_ROWS;
    for (const p of players) {
      if (p.x + 1 > cols) cols = p.x + 1;
      if (p.y + 1 > rows) rows = p.y + 1;
    }

    // Tile size: fit grid + half-tile padding on each side
    const tileSize = Math.min(width / (cols + 1), height / (rows + 1));
    const gridW = cols * tileSize;
    const gridH = rows * tileSize;
    const originX = (width - gridW) / 2;
    const originY = (height - gridH) / 2;

    this.drawGrid(width, height, cols, rows, tileSize, originX, originY);
    this.drawPlayers(players, tileSize, originX, originY);
  }

  private drawGrid(
    w: number, h: number,
    cols: number, rows: number,
    tile: number, ox: number, oy: number,
  ): void {
    if (!this.grid) return;
    this.grid.clear();

    this.grid.fillStyle(0x0b1020, 1);
    this.grid.fillRect(0, 0, w, h);

    // Cell backgrounds
    for (let c = 0; c < cols; c++) {
      for (let r = 0; r < rows; r++) {
        const shade = (c + r) % 2 === 0 ? 0x0e1528 : 0x101a30;
        this.grid.fillStyle(shade, 1);
        this.grid.fillRect(ox + c * tile, oy + r * tile, tile, tile);
      }
    }

    // Grid lines
    this.grid.lineStyle(1, 0x1f314d, 0.4);
    for (let c = 0; c <= cols; c++) {
      const x = ox + c * tile;
      this.grid.lineBetween(x, oy, x, oy + rows * tile);
    }
    for (let r = 0; r <= rows; r++) {
      const y = oy + r * tile;
      this.grid.lineBetween(ox, y, ox + cols * tile, y);
    }

    // Origin marker
    this.grid.fillStyle(0x5b8cff, 0.12);
    this.grid.fillCircle(ox + tile * 0.5, oy + tile * 0.5, tile * 0.38);
  }

  private drawPlayers(
    players: RenderPlayer[],
    tile: number, ox: number, oy: number,
  ): void {
    const stale = new Set(this.sprites.keys());
    const circleR = Math.max(10, tile * 0.24);
    const labelPx = Math.max(11, Math.round(tile * 0.17));
    const bubblePx = Math.max(10, Math.round(tile * 0.15));
    const bubbleWrap = Math.max(120, tile * 1.6);

    for (const player of players) {
      stale.delete(player.id);
      const cx = ox + (player.x + 0.5) * tile;
      const cy = ox === oy ? oy + (player.y + 0.5) * tile : oy + (player.y + 0.5) * tile;

      let sprite = this.sprites.get(player.id);
      if (!sprite) {
        sprite = {
          body: this.add.circle(cx, cy, circleR, player.isSelf ? 0x5b8cff : 0x3ddc97),
          label: this.add.text(cx, cy, player.name, {
            fontFamily: 'Inter, system-ui, sans-serif',
            fontSize: `${labelPx}px`,
            color: '#ffffff',
            align: 'center',
            stroke: '#08101d',
            strokeThickness: 3,
          }).setOrigin(0.5, 1),
        };
        this.sprites.set(player.id, sprite);
      }

      sprite.body.setRadius(circleR);
      sprite.body.fillColor = player.isSelf ? 0x5b8cff : 0x3ddc97;
      sprite.body.setPosition(cx, cy);

      sprite.label
        .setFontSize(labelPx)
        .setText(player.name)
        .setPosition(cx, cy - circleR - 4);

      if (player.chat) {
        if (!sprite.bubble) {
          sprite.bubble = this.add.text(cx, cy, '', {
            fontFamily: 'Inter, system-ui, sans-serif',
            fontSize: `${bubblePx}px`,
            color: '#0c1424',
            backgroundColor: '#d6e4ff',
            padding: { x: 8, y: 5 },
            wordWrap: { width: bubbleWrap },
            maxLines: 3,
          }).setOrigin(0.5, 1);
        }
        sprite.bubble
          .setFontSize(bubblePx)
          .setWordWrapWidth(bubbleWrap)
          .setText(player.chat)
          .setPosition(cx, cy - circleR - labelPx - 10)
          .setVisible(true);
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
}
