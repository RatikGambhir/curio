import { GoogleGenAI } from "@google/genai";

import { persistChat, type ChatPersistenceInput } from "./database";

interface Env {
	GEMINI_API_KEY: string;
	CURIO_DB: D1Database;
	CURIO_ALLOWED_ORIGINS?: string;
	readonly CURIO_QUESTION_QUEUE?: Queue<QueueBody>;
}

interface ChatWorkerRequest extends ChatPersistenceInput {}

interface QueueBody {
	userId: string;
	userMessageId: string;
	threadId: string;
	assistantMessageId: string;
	assetPath: string[];
}

const baseCors = {
	"Access-Control-Allow-Headers": "Content-Type",
	"Access-Control-Allow-Methods": "POST, OPTIONS",
	"Access-Control-Max-Age": "86400",
};

const localDevelopmentOrigins = [
	"http://localhost:1420",
	"http://127.0.0.1:1420",
	"http://localhost:5173",
	"http://127.0.0.1:5173",
];

const allowedOrigins = (env: Env) =>
	new Set(
		(env.CURIO_ALLOWED_ORIGINS?.split(",") ?? localDevelopmentOrigins)
			.map((origin) => origin.trim())
			.filter(Boolean),
	);

const corsHeaders = (request: Request, env: Env): Record<string, string> => {
	const origin = request.headers.get("Origin");
	if (!origin || !allowedOrigins(env).has(origin)) {
		return baseCors;
	}

	return {
		...baseCors,
		"Access-Control-Allow-Origin": origin,
		Vary: "Origin",
	};
};

const textResponse = (
	body: string | null,
	status: number,
	cors: HeadersInit,
	headers: HeadersInit = {},
) =>
	new Response(body, {
		status,
		headers: {
			...cors,
			"Content-Type": "text/plain; charset=utf-8",
			...headers,
		},
	});

const extractRequestData = async (request: Request): Promise<ChatWorkerRequest> => {
	const contentType = request.headers.get("Content-Type") ?? "";
	if (contentType.includes("application/json")) {
		const body = (await request.json()) as Partial<ChatWorkerRequest>;
		return {
			userId: String(body.userId ?? ""),
			conversationId: String(body.conversationId ?? ""),
			userMessageId: String(body.userMessageId ?? ""),
			assistantMessageId: String(body.assistantMessageId ?? ""),
			prompt: String(body.prompt ?? ""),
			attachments: null,
		};
	}

	const form = await request.formData();
	return {
		userId: String(form.get("userId") ?? ""),
		conversationId: String(form.get("conversationId") ?? ""),
		userMessageId: String(form.get("userMessageId") ?? ""),
		assistantMessageId: String(form.get("assistantMessageId") ?? ""),
		prompt: String(form.get("prompt") ?? ""),
		attachments: form
			.getAll("attachment")
			.filter((value): value is File => value instanceof File),
	};
};

const validateRequest = (request: ChatWorkerRequest): string | null => {
	if (!request.userId) return "userId is required";
	if (!request.conversationId) return "conversationId is required";
	if (!request.userMessageId) return "userMessageId is required";
	if (!request.assistantMessageId) return "assistantMessageId is required";
	if (!request.prompt.trim()) return "prompt is required";
	return null;
};

export default {
	async fetch(request, env: Env, ctx): Promise<Response> {
		const cors = corsHeaders(request, env);
		const respondText = (
			body: string | null,
			status: number,
			headers: HeadersInit = {},
		) => textResponse(body, status, cors, headers);

		try {
			const url = new URL(request.url);
			if (request.method === "OPTIONS") {
				const origin = request.headers.get("Origin");
				if (origin && !allowedOrigins(env).has(origin)) {
					return respondText("Origin not allowed", 403);
				}
				return new Response(null, { status: 204, headers: cors });
			}

			if (url.pathname !== "/v1/chat/stream" && url.pathname !== "/chat") {
				return respondText("Not found", 404);
			}

			if (request.method !== "POST") {
				return respondText("Method not allowed", 405);
			}

			if (!env.GEMINI_API_KEY) {
				return respondText(
					"GEMINI_API_KEY is not configured for this worker.",
					500,
				);
			}

			let data: ChatWorkerRequest;
			try {
				data = await extractRequestData(request);
			} catch {
				return respondText("Invalid request body", 400);
			}

			const validationError = validateRequest(data);
			if (validationError) {
				return respondText(validationError, 400);
			}

			const gemini = new GoogleGenAI({ apiKey: env.GEMINI_API_KEY });
			const { readable, writable } = new TransformStream();
			const writer = writable.getWriter();
			const encoder = new TextEncoder();
			const writeEvent = async (event: string, payload: object) => {
				await writer.write(
					encoder.encode(`event: ${event}\ndata: ${JSON.stringify(payload)}\n\n`),
				);
			};

			async function streamResponse() {
				let accumulatedResponse = "";
				try {
					const response = await gemini.models.generateContentStream({
						model: "gemini-2.5-flash",
						contents: data.prompt,
					});

					for await (const chunk of response) {
						if (chunk.text) {
							accumulatedResponse += chunk.text;
							await writeEvent("token", {
								conversationId: data.conversationId,
								messageId: data.assistantMessageId,
								token: chunk.text,
							});
						}
					}

					const record = await persistChat(env.CURIO_DB, data, accumulatedResponse);
					console.log("[sqlite] Persisted chat record", {
						userId: record.userId,
						threadId: record.threadId,
						assetCount: record.assets.length,
					});

					if (env.CURIO_QUESTION_QUEUE) {
						await env.CURIO_QUESTION_QUEUE.send({
							userId: record.userId,
							threadId: record.threadId,
							userMessageId: record.userMessageId,
							assistantMessageId: record.assistantMessageId,
							assetPath: record.assets.map((asset) => asset.path),
						});
					}

					await writeEvent("done", {
						conversationId: data.conversationId,
						messageId: data.assistantMessageId,
						responseId: crypto.randomUUID(),
					});
				} catch (error) {
					await writeEvent("error", {
						conversationId: data.conversationId,
						messageId: data.assistantMessageId,
						code: "worker_error",
						message:
							error instanceof Error
								? error.message
								: "The chat worker could not complete the request.",
					});
				} finally {
					await writer.close();
				}
			}

			ctx.waitUntil(streamResponse());

			return new Response(readable, {
				headers: {
					...cors,
					"Content-Type": "text/event-stream",
					"Cache-Control": "no-cache",
					Connection: "keep-alive",
					"X-Accel-Buffering": "no",
				},
			});
		} catch (error) {
			console.error("[chat] Unhandled request failure", error);
			return respondText("Internal server error", 500);
		}
	},
} satisfies ExportedHandler<Env>;
