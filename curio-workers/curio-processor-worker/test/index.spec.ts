import { env } from "cloudflare:test";
import { beforeAll, describe, expect, it } from "vitest";

import {
	initializeSqliteSchema,
	loadAssets,
	loadMessages,
	storeEmbeddings,
} from "../src/database";
import worker from "../src/index";

const queueRecord = {
	userId: "mock-user-1",
	threadId: "processor-conversation-test",
	userMessageId: "processor-user-message-test",
	assistantMessageId: "processor-assistant-message-test",
	assetPath: ["assets/mock-user-1/processor-asset-test/notes.txt"],
};

describe("processor worker", () => {
	beforeAll(async () => {
		await initializeSqliteSchema(env.CURIO_DB);
		await env.CURIO_DB.batch([
			env.CURIO_DB.prepare(
				"INSERT INTO conversations (id, user_id) VALUES (?1, ?2)",
			).bind(queueRecord.threadId, queueRecord.userId),
			env.CURIO_DB.prepare(
				`INSERT INTO messages
           (id, conversation_id, user_id, role, content, status)
         VALUES (?1, ?2, ?3, 'user', 'Hello', 'completed')`,
			).bind(
				queueRecord.userMessageId,
				queueRecord.threadId,
				queueRecord.userId,
			),
			env.CURIO_DB.prepare(
				`INSERT INTO messages
           (id, conversation_id, user_id, role, content, status)
         VALUES (?1, ?2, ?3, 'assistant', 'Hi there', 'completed')`,
			).bind(
				queueRecord.assistantMessageId,
				queueRecord.threadId,
				queueRecord.userId,
			),
			env.CURIO_DB.prepare(
				`INSERT INTO assets
           (id, conversation_id, message_id, user_id, path, file_name,
            file_type, file_size, last_modified, content)
         VALUES (?1, ?2, ?3, ?4, ?5, 'notes.txt', 'text/plain', 5, 0, ?6)`,
			).bind(
				"processor-asset-test",
				queueRecord.threadId,
				queueRecord.userMessageId,
				queueRecord.userId,
				queueRecord.assetPath[0],
				new TextEncoder().encode("asset"),
			),
		]);
	});

	it("exports a queue handler", () => {
		expect(worker.queue).toBeTypeOf("function");
	});

	it("reads message and attachment records from SQLite", async () => {
		const [messages, assets] = await Promise.all([
			loadMessages(env.CURIO_DB, queueRecord),
			loadAssets(env.CURIO_DB, queueRecord.assetPath),
		]);

		expect(messages.map((message) => message.content)).toEqual([
			"Hello",
			"Hi there",
		]);
		expect(new TextDecoder().decode(assets[0].content)).toBe("asset");
	});

	it("writes generated embeddings back to SQLite", async () => {
		await storeEmbeddings(env.CURIO_DB, "test-model", [
			{
				type: "message",
				id: queueRecord.userMessageId,
				value: { embeddings: [{ values: [0.1, 0.2] }] },
			},
		]);

		const stored = await env.CURIO_DB.prepare(
			"SELECT source_id, model, value_json FROM embeddings WHERE source_id = ?1",
		)
			.bind(queueRecord.userMessageId)
			.first<{ source_id: string; model: string; value_json: string }>();

		expect(stored?.source_id).toBe(queueRecord.userMessageId);
		expect(stored?.model).toBe("test-model");
		expect(JSON.parse(stored?.value_json ?? "{}")).toEqual({
			embeddings: [{ values: [0.1, 0.2] }],
		});
	});
});
