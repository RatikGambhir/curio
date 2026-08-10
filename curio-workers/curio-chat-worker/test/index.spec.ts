import {
	SELF,
	createExecutionContext,
	env,
	waitOnExecutionContext,
} from "cloudflare:test";
import { beforeAll, describe, expect, it } from "vitest";

import { initializeSqliteSchema, persistChat } from "../src/database";
import worker from "../src/index";

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

describe("chat worker", () => {
	beforeAll(async () => {
		await initializeSqliteSchema(env.CURIO_DB);
	});

	it("handles CORS preflight requests", async () => {
		const request = new IncomingRequest(
			"http://example.com/v1/chat/stream",
			{ method: "OPTIONS" },
		);
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		expect(response.status).toBe(204);
		expect(response.headers.get("Access-Control-Allow-Methods")).toBe(
			"POST, OPTIONS",
		);
	});

	it("rejects non-POST chat requests", async () => {
		const response = await SELF.fetch(
			"https://example.com/v1/chat/stream",
		);

		expect(response.status).toBe(405);
		expect(await response.text()).toBe("Method not allowed");
	});

	it("stores messages and attachment BLOBs in SQLite", async () => {
		const attachment = new File(["hello sqlite"], "notes.txt", {
			type: "text/plain",
			lastModified: 1_700_000_000_000,
		});
		const stored = await persistChat(
			env.CURIO_DB,
			{
				userId: "mock-user-1",
				conversationId: "conversation-d1-test",
				userMessageId: "user-message-d1-test",
				assistantMessageId: "assistant-message-d1-test",
				prompt: "Save this",
				attachments: [attachment],
			},
			"Saved",
		);

		const messages = await env.CURIO_DB.prepare(
			"SELECT role, content FROM messages WHERE conversation_id = ?1 ORDER BY created_at, rowid",
		)
			.bind(stored.threadId)
			.all<{ role: string; content: string }>();
		const asset = await env.CURIO_DB.prepare(
			"SELECT file_name, content FROM assets WHERE id = ?1",
		)
			.bind(stored.assets[0].id)
			.first<{ file_name: string; content: number[] }>();

		expect(messages.results).toEqual([
			{ role: "user", content: "Save this" },
			{ role: "assistant", content: "Saved" },
		]);
		expect(asset?.file_name).toBe("notes.txt");
		expect(new TextDecoder().decode(Uint8Array.from(asset?.content ?? []))).toBe(
			"hello sqlite",
		);
	});
});
