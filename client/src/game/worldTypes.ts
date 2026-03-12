export interface RenderPlayer {
  id: string;
  name: string;
  x: number;
  y: number;
  isSelf: boolean;
  chat?: string;
}

export interface WorldLayout {
  width: number;
  height: number;
  cols: number;
  rows: number;
  tileSize: number;
  originX: number;
  originY: number;
}
