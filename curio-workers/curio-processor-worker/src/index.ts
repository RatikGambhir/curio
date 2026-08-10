import { type EmbedContentResponse, GoogleGenAI } from "@google/genai";

import {
	loadAssets,
	loadMessages,
	storeEmbeddings,
	type EmbeddingSource,
	type QueueRecord,
} from "./database";

interface Env {
	GEMINI_API_KEY: string;
	CURIO_DB: D1Database;
}

const EMBEDDING_MODEL = "gemini-embedding-001";

const queryEmbedding = async (
	content: string,
	geminiClient: GoogleGenAI,
): Promise<EmbedContentResponse> =>
	geminiClient.models.embedContent({
		model: EMBEDDING_MODEL,
		contents: content,
	});

const encodeBytes = (bytes: Uint8Array) => {
	let binary = "";
	const chunkSize = 0x8000;
	for (let index = 0; index < bytes.length; index += chunkSize) {
		binary += String.fromCharCode(...bytes.subarray(index, index + chunkSize));
	}
	return btoa(binary);
};

export default {
	async queue(batch: MessageBatch<QueueRecord>, env: Env): Promise<void> {
		const geminiClient = new GoogleGenAI({ apiKey: env.GEMINI_API_KEY });

		for (const message of batch.messages) {
			try {
				const [messages, assets] = await Promise.all([
					loadMessages(env.CURIO_DB, message.body),
					loadAssets(env.CURIO_DB, message.body.assetPath ?? []),
				]);

				const [messageEmbeddings, assetEmbeddings] = await Promise.all([
					Promise.all(
						messages.map((record) =>
							queryEmbedding(record.content, geminiClient),
						),
					),
					Promise.all(
						assets.map((asset) =>
							queryEmbedding(encodeBytes(asset.content), geminiClient),
						),
					),
				]);

				const sources: EmbeddingSource[] = [
					...messages.map((record, index) => ({
						type: "message" as const,
						id: record.id,
						value: messageEmbeddings[index],
					})),
					...assets.map((asset, index) => ({
						type: "asset" as const,
						id: asset.id,
						value: assetEmbeddings[index],
					})),
				];

				await storeEmbeddings(env.CURIO_DB, EMBEDDING_MODEL, sources);

				console.log("[processor] Stored SQLite embeddings", {
					userId: message.body.userId,
					threadId: message.body.threadId,
					messageCount: messages.length,
					assetCount: assets.length,
				});
			} catch (error) {
				console.error("[processor] SQLite processing failed", {
					threadId: message.body.threadId,
					error,
				});
				message.retry();
			}
		}
	},
} satisfies ExportedHandler<Env, QueueRecord>;
