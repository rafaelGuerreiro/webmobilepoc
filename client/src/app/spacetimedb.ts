import type { Identity } from 'spacetimedb';
import { DbConnection, type EventContext } from '../module_bindings';
import type { ChatBubbleV1 } from '../module_bindings/types';
import { DATABASE, HOST } from './appConfig';
import type { AppDataState } from './appState';

interface RowObserver<Row> {
  onInsert(callback: (ctx: EventContext, row: Row) => void): unknown;
  onDelete(callback: (ctx: EventContext, row: Row) => void): unknown;
  onUpdate(callback: (ctx: EventContext, oldRow: Row, newRow: Row) => void): unknown;
}

export interface AppConnectionHandlers {
  onConnected(identity: Identity, token: string): void;
  onConnectionError(error: Error): void;
  onDisconnected(error: Error | null): void;
  onStateChanged(state: AppDataState): void;
  onBubble(bubble: ChatBubbleV1): void;
}

export function connectApp(savedToken: string | undefined, handlers: AppConnectionHandlers): DbConnection {
  return DbConnection.builder()
    .withUri(HOST)
    .withDatabaseName(DATABASE)
    .withToken(savedToken)
    .onConnect((conn, identity, token) => {
      handlers.onConnected(identity, token);
      registerSubscriptions(conn, handlers);
      registerRowObservers(conn, handlers);
    })
    .onConnectError((_ctx, error) => {
      handlers.onConnectionError(error);
    })
    .onDisconnect((_ctx, error) => {
      handlers.onDisconnected(error ?? null);
    })
    .build();
}

function registerSubscriptions(conn: DbConnection, handlers: AppConnectionHandlers): void {
  conn.subscriptionBuilder()
    .onApplied(() => {
      handlers.onStateChanged(readAppDataState(conn));
    })
    // This app keeps a single always-on subscription, so the simple whole-db
    // subscription is easier to maintain than a hand-curated query list.
    .subscribeToAllTables();
}

function registerRowObservers(conn: DbConnection, handlers: AppConnectionHandlers): void {
  const refresh = (): void => {
    handlers.onStateChanged(readAppDataState(conn));
  };

  observeTableChanges(conn.db.vw_user_me_v1, refresh);
  observeTableChanges(conn.db.vw_world_my_position_v1, refresh);
  observeTableChanges(conn.db.vw_nearby_positions_v1, refresh);

  conn.db.chat_bubble_v1.onInsert((_ctx, bubble) => {
    handlers.onBubble(bubble);
  });
}

function observeTableChanges<Row>(table: RowObserver<Row>, onChange: () => void): void {
  table.onInsert(() => onChange());
  table.onDelete(() => onChange());
  table.onUpdate(() => onChange());
}

function readAppDataState(conn: DbConnection): AppDataState {
  return {
    me: [...conn.db.vw_user_me_v1.iter()][0] ?? null,
    currentPosition: [...conn.db.vw_world_my_position_v1.iter()][0] ?? null,
    nearbyPositions: [...conn.db.vw_nearby_positions_v1.iter()],
  };
}
