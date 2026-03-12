import type { RenderPlayer, WorldLayout } from './worldTypes';

const MIN_COLS = 4;
const MIN_ROWS = 4;
const EDGE_PADDING_TILES = 0.5;

export function createWorldLayout(players: RenderPlayer[], width: number, height: number): WorldLayout {
  let cols = MIN_COLS;
  let rows = MIN_ROWS;

  for (const player of players) {
    if (player.x + 1 > cols) cols = player.x + 1;
    if (player.y + 1 > rows) rows = player.y + 1;
  }

  const tileSize = Math.min(width / (cols + 1), height / (rows + 1));

  return {
    width,
    height,
    cols,
    rows,
    tileSize,
    originX: (width - cols * tileSize) / 2,
    originY: tileSize * EDGE_PADDING_TILES,
  };
}
