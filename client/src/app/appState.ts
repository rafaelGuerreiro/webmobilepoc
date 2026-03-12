import type { UserPositionV1, UserV1 } from '../module_bindings/types';

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

export interface AppDataState {
  me: UserV1 | null;
  currentPosition: UserPositionV1 | null;
  nearbyPositions: UserPositionV1[];
}

export interface AppState extends AppDataState {
  status: ConnectionStatus;
  identity: string | null;
  error: string | null;
}

export const DEFAULT_STATE: AppState = {
  status: 'connecting',
  identity: null,
  error: null,
  me: null,
  currentPosition: null,
  nearbyPositions: [],
};

export function readableStatus(status: ConnectionStatus): string {
  switch (status) {
    case 'connecting': return 'Connecting…';
    case 'connected': return 'Connected';
    case 'disconnected': return 'Disconnected';
    case 'error': return 'Error';
  }
}
