/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.jsonc`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `npm run cf-typegen`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

import { GoogleGenAI } from '@google/genai';

interface Env {
	GEMINI_API_KEY: string;
}

interface Request {
	prompt: string;
}

function isValidReq(req: any): boolean {
	return true
}

export default {
	async fetch(request, env: Env, ctx): Promise<Response> {
		const gemini = new GoogleGenAI({ apiKey: env.GEMINI_API_KEY });
		const { readable, writable } = new TransformStream();
		const writer = writable.getWriter();
		const encoder = new TextEncoder();
		const body = await request.json();
		//TODO: Payload validation, custom prompting, Response cleaning, and post processing
		//TODO: DO VALIDATION
		async function streamResponse() {
			try {
				const response = await gemini.models.generateContentStream({
					model: 'gemini-2.5-flash',
					contents: 'Hello!',
				});
				for await (const chunk of response) {
					if (chunk.text) {
						const json = JSON.stringify({ token: chunk.text });
						await writer.write(encoder.encode(`data: ${json}\n\n`));
					}
				}
			} catch (error) {
				console.error('Error generating content:', error);
			} finally {
				await writer.close();
			}
		}

		ctx.waitUntil(streamResponse());

		return new Response(readable, {
			headers: {
				'Content-Type': 'text/event-stream',
				'Cache-Control': 'no-cache',
				Connection: 'keep-alive',
				'X-Accel-Buffering': 'no',
			},
		});
	},
} satisfies ExportedHandler<Env>;
