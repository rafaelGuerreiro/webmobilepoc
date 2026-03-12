/** How long a chat bubble stays visible on the map (ms). */
export const BUBBLE_TTL_MS = 8_000;

export const HOST = import.meta.env.VITE_STDB_URI ?? 'https://maincloud.spacetimedb.com';
export const DATABASE = import.meta.env.VITE_STDB_NAME ?? 'webmobiledb';
export const TOKEN_KEY = `${HOST}/${DATABASE}/auth_token`;
