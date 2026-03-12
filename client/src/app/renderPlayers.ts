import type { Identity } from 'spacetimedb';
import type { UserPositionV1 } from '../module_bindings/types';
import type { RenderPlayer } from '../game/worldTypes';
import type { AppDataState } from './appState';

interface BubbleLookup {
  contentFor(identityKey: string): string | undefined;
}

export function buildRenderPlayers(state: AppDataState, liveBubbles: BubbleLookup): RenderPlayer[] {
  const myUserId = state.me?.userId ?? null;
  const positionsById = new Map<string, UserPositionV1>();

  for (const position of state.nearbyPositions) {
    positionsById.set(identityKey(position.userId), position);
  }

  if (state.currentPosition) {
    positionsById.set(identityKey(state.currentPosition.userId), state.currentPosition);
  }

  return [...positionsById.values()]
    .map((position) => {
      const isSelf = myUserId ? sameIdentity(position.userId, myUserId) : false;
      const id = identityKey(position.userId);
      return {
        id,
        name: userLabel(position.userId, isSelf),
        x: position.x,
        y: position.y,
        isSelf,
        chat: liveBubbles.contentFor(id),
      } satisfies RenderPlayer;
    })
    .sort((left, right) => left.y - right.y || left.x - right.x || left.name.localeCompare(right.name));
}

export function formatIdentity(identity: Identity): string {
  return identityKey(identity);
}

export function identityKey(identity: Identity): string {
  return identity.toHexString();
}

function sameIdentity(left: Identity, right: Identity): boolean {
  return identityKey(left) === identityKey(right);
}

function shortIdentity(identity: Identity): string {
  return identityKey(identity).slice(0, 8);
}

function userLabel(userId: Identity, isSelf: boolean): string {
  return isSelf ? 'You' : `User ${shortIdentity(userId)}`;
}
