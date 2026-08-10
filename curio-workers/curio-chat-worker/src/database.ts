export type ChatPersistenceInput = {
	userId: string;
	conversationId: string;
	userMessageId: string;
	assistantMessageId: string;
	prompt: string;
	attachments: File[] | null;
};

export type StoredAsset = {
	id: string;
	path: string;
	fileName: string;
	fileType: string;
	fileSize: number;
	lastModified: number;
	content: ArrayBuffer;
};

export type StoredChat = {
	userId: string;
	threadId: string;
	userMessageId: string;
	assistantMessageId: string;
	assets: StoredAsset[];
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
];

export async function initializeSqliteSchema(database: D1Database) {
	await database.batch(
		SQLITE_SCHEMA_STATEMENTS.map((statement) => database.prepare(statement)),
	);
}

async function createStoredAssets(
	userId: string,
	attachments: File[],
): Promise<StoredAsset[]> {
	return Promise.all(
		attachments.map(async (attachment) => {
			const id = crypto.randomUUID();
			return {
				id,
				path: `assets/${encodeURIComponent(userId)}/${id}/${encodeURIComponent(attachment.name)}`,
				fileName: attachment.name,
				fileType: attachment.type || "application/octet-stream",
				fileSize: attachment.size,
				lastModified: attachment.lastModified,
				content: await attachment.arrayBuffer(),
			};
		}),
	);
}

export async function persistChat(
	database: D1Database,
	request: ChatPersistenceInput,
	response: string,
): Promise<StoredChat> {
	const assets = await createStoredAssets(
		request.userId,
		request.attachments ?? [],
	);

	const statements = [
		database
			.prepare(
				`INSERT INTO conversations (id, user_id, kind)
         VALUES (?1, ?2, 'chat')
         ON CONFLICT(id) DO UPDATE SET
           user_id = excluded.user_id,
           updated_at = unixepoch()`,
			)
			.bind(request.conversationId, request.userId),
		database
			.prepare(
				`INSERT INTO messages
           (id, conversation_id, user_id, role, content, status)
         VALUES (?1, ?2, ?3, 'user', ?4, 'completed')`,
			)
			.bind(
				request.userMessageId,
				request.conversationId,
				request.userId,
				request.prompt,
			),
		database
			.prepare(
				`INSERT INTO messages
           (id, conversation_id, user_id, role, content, status)
         VALUES (?1, ?2, ?3, 'assistant', ?4, 'completed')`,
			)
			.bind(
				request.assistantMessageId,
				request.conversationId,
				request.userId,
				response,
			),
		...assets.map((asset) =>
			database
				.prepare(
					`INSERT INTO assets
             (id, conversation_id, message_id, user_id, path, file_name,
              file_type, file_size, last_modified, content)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)`,
				)
				.bind(
					asset.id,
					request.conversationId,
					request.userMessageId,
					request.userId,
					asset.path,
					asset.fileName,
					asset.fileType,
					asset.fileSize,
					asset.lastModified,
					asset.content,
				),
		),
	];

	await database.batch(statements);

	return {
		userId: request.userId,
		threadId: request.conversationId,
		userMessageId: request.userMessageId,
		assistantMessageId: request.assistantMessageId,
		assets,
	};
}
