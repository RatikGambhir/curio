import type { EmbedContentResponse } from "@google/genai";

export type QueueRecord = {
	userId: string;
	userMessageId: string;
	threadId: string;
	assistantMessageId: string;
	assetPath: string[];
};

export type StoredMessage = {
	id: string;
	role: "user" | "assistant";
	content: string;
};

type AssetRow = {
	id: string;
	path: string;
	content: number[];
};

export type StoredAsset = {
	id: string;
	path: string;
	content: Uint8Array;
};

export type EmbeddingSource = {
	type: "message" | "asset";
	id: string;
	value: EmbedContentResponse;
};

export const SQLITE_SCHEMA_STATEMENTS = [
	"PRAGMA foreign_keys = ON",
	`CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'chat',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT`,
	`CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
  content TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'failed')),
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
) STRICT`,
	`CREATE TABLE IF NOT EXISTS assets (
  id TEXT PRIMARY KEY NOT NULL,
  conversation_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  file_name TEXT NOT NULL,
  file_type TEXT NOT NULL,
  file_size INTEGER NOT NULL,
  last_modified INTEGER NOT NULL,
  content BLOB NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
  FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
) STRICT`,
	`CREATE TABLE IF NOT EXISTS embeddings (
  id TEXT PRIMARY KEY NOT NULL,
  source_type TEXT NOT NULL CHECK (source_type IN ('message', 'asset')),
  source_id TEXT NOT NULL,
  model TEXT NOT NULL,
  value_json TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  UNIQUE (source_type, source_id, model)
) STRICT`,
];

export async function initializeSqliteSchema(database: D1Database) {
	await database.batch(
		SQLITE_SCHEMA_STATEMENTS.map((statement) => database.prepare(statement)),
	);
}

export async function loadMessages(
	database: D1Database,
	body: QueueRecord,
): Promise<StoredMessage[]> {
	const result = await database
		.prepare(
			`SELECT id, role, content
       FROM messages
       WHERE id IN (?1, ?2)
       ORDER BY CASE id WHEN ?1 THEN 0 ELSE 1 END`,
		)
		.bind(body.userMessageId, body.assistantMessageId)
		.all<StoredMessage>();

	if (result.results.length !== 2) {
		throw new Error("SQLite message records are incomplete.");
	}

	return result.results;
}

export async function loadAssets(
	database: D1Database,
	paths: string[],
): Promise<StoredAsset[]> {
	if (paths.length === 0) {
		return [];
	}

	const statement = database.prepare(
		"SELECT id, path, content FROM assets WHERE path = ?1",
	);
	const results = await database.batch<AssetRow>(
		paths.map((path) => statement.bind(path)),
	);

	return results.map((result, index) => {
		const row = result.results[0];
		if (!row) {
			throw new Error(`SQLite asset record is missing for ${paths[index]}.`);
		}
		return {
			id: row.id,
			path: row.path,
			content: Uint8Array.from(row.content),
		};
	});
}

export async function storeEmbeddings(
	database: D1Database,
	model: string,
	sources: EmbeddingSource[],
): Promise<void> {
	if (sources.length === 0) {
		return;
	}

	await database.batch(
		sources.map((source) =>
			database
				.prepare(
					`INSERT INTO embeddings
             (id, source_type, source_id, model, value_json)
           VALUES (?1, ?2, ?3, ?4, ?5)
           ON CONFLICT(source_type, source_id, model) DO UPDATE SET
             value_json = excluded.value_json,
             updated_at = unixepoch()`,
				)
				.bind(
					crypto.randomUUID(),
					source.type,
					source.id,
					model,
					JSON.stringify(source.value),
				),
		),
	);
}
