import Phaser from 'phaser';
import type { WorldLayout } from './worldTypes';

const BACKGROUND_COLOR = 0x0b1020;
const GRID_LINE_COLOR = 0x1f314d;
const ORIGIN_MARKER_COLOR = 0x5b8cff;
const LIGHT_CELL_COLOR = 0x0e1528;
const DARK_CELL_COLOR = 0x101a30;

export class WorldGridRenderer {
  private grid?: Phaser.GameObjects.Graphics;

  constructor(private readonly scene: Phaser.Scene) {}

  create(): void {
    this.grid = this.scene.add.graphics();
  }

  draw(layout: WorldLayout): void {
    if (!this.grid) {
      return;
    }

    const { width, height, cols, rows, tileSize, originX, originY } = layout;

    this.grid.clear();
    this.grid.fillStyle(BACKGROUND_COLOR, 1);
    this.grid.fillRect(0, 0, width, height);

    for (let col = 0; col < cols; col++) {
      for (let row = 0; row < rows; row++) {
        const shade = (col + row) % 2 === 0 ? LIGHT_CELL_COLOR : DARK_CELL_COLOR;
        this.grid.fillStyle(shade, 1);
        this.grid.fillRect(originX + col * tileSize, originY + row * tileSize, tileSize, tileSize);
      }
    }

    this.grid.lineStyle(1, GRID_LINE_COLOR, 0.4);
    for (let col = 0; col <= cols; col++) {
      const x = originX + col * tileSize;
      this.grid.lineBetween(x, originY, x, originY + rows * tileSize);
    }

    for (let row = 0; row <= rows; row++) {
      const y = originY + row * tileSize;
      this.grid.lineBetween(originX, y, originX + cols * tileSize, y);
    }

    this.grid.fillStyle(ORIGIN_MARKER_COLOR, 0.12);
    this.grid.fillCircle(originX + tileSize * 0.5, originY + tileSize * 0.5, tileSize * 0.38);
  }
}
