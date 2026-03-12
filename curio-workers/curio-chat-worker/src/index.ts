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
import { createRemoteJWKSet, errors, jwtVerify } from 'jose';

interface Env {
	GEMINI_API_KEY: string;
	SUPABASE_URL: string;
	SUPABASE_ANON_KEY: string;
	SUPABASE_JWT_AUDIENCE?: string;
	readonly CURIO_QUESTION_QUEUE: Queue;
}

interface ChatRequestBody {
	prompt: string;
}

interface SupabaseUser {
	id: string;
	email?: string | null;
}

const supabaseJwksCache = new Map<string, ReturnType<typeof createRemoteJWKSet>>();

const cors = {
	'Access-Control-Allow-Origin': '*',
	'Access-Control-Allow-Headers': 'Content-Type, Authorization',
	'Access-Control-Allow-Methods': 'POST, OPTIONS',
};

function getBearerToken(request: Request): string | null {
	const authorization = request.headers.get('Authorization');
	if (!authorization) {
		return null;
	}

	const [scheme, token] = authorization.split(' ');
	if (scheme !== 'Bearer' || !token) {
		return null;
	}

	return token;
}

function getSupabaseIssuer(env: Env): string {
	return `${env.SUPABASE_URL.replace(/\/$/, '')}/auth/v1`;
}

function getSupabaseJwks(env: Env) {
	const issuer = getSupabaseIssuer(env);
	const jwksUrl = `${issuer}/.well-known/jwks.json`;
	const cachedJwks = supabaseJwksCache.get(jwksUrl);
	if (cachedJwks) {
		return cachedJwks;
	}

	const jwks = createRemoteJWKSet(new URL(jwksUrl), {
		cooldownDuration: 5 * 60 * 1000,
		cacheMaxAge: 60 * 60 * 1000,
	});
	supabaseJwksCache.set(jwksUrl, jwks);
	return jwks;
}

async function authenticateRequest(request: Request, env: Env): Promise<SupabaseUser | null> {
	const token = getBearerToken(request);
	if (!token) {
		return null;
	}

	try {
		const { payload } = await jwtVerify(token, getSupabaseJwks(env), {
			issuer: getSupabaseIssuer(env),
			audience: env.SUPABASE_JWT_AUDIENCE ?? 'authenticated',
		});

		if (typeof payload.sub !== 'string' || payload.sub.length === 0) {
			return null;
		}

		return {
			id: payload.sub,
			email: typeof payload.email === 'string' ? payload.email : null,
		};
	} catch (error) {
		if (error instanceof errors.JOSEError) {
			return null;
		}

		throw error;
	}
}

export default {
	async fetch(request, env: Env, ctx): Promise<Response> {
		const url = new URL(request.url);
		let text = 'DATA -> ';

		// if (request.method === 'OPTIONS') {
		// 	return new Response(null, {
		// 		status: 204,
		// 		headers: cors,
		// 	});
		// }
		//
		// if (url.pathname !== '/chat') {
		// 	return new Response('Not found', {
		// 		status: 404,
		// 		headers: {
		// 			...cors,
		// 			'Content-Type': 'text/plain; charset=utf-8',
		// 		},
		// 	});
		// }
		//
		// if (!env.SUPABASE_URL) {
		// 	return new Response('Supabase auth is not configured for this worker.', {
		// 		status: 500,
		// 		headers: {
		// 			...cors,
		// 			'Content-Type': 'text/plain; charset=utf-8',
		// 		},
		// 	});
		// }
		//
		// let authenticatedUser: SupabaseUser | null;
		// try {
		// 	authenticatedUser = await authenticateRequest(request, env);
		// } catch (error) {
		// 	console.error('Supabase auth verification failed', error);
		// 	return new Response('Authentication service unavailable.', {
		// 		status: 503,
		// 		headers: {
		// 			...cors,
		// 			'Content-Type': 'text/plain; charset=utf-8',
		// 		},
		// 	});
		// }
		//
		// if (!authenticatedUser) {
		// 	return new Response('Unauthorized', {
		// 		status: 401,
		// 		headers: {
		// 			...cors,
		// 			'Content-Type': 'text/plain; charset=utf-8',
		// 		},
		// 	});
		// }
		// const user = authenticatedUser;

		const gemini = new GoogleGenAI({ apiKey: env.GEMINI_API_KEY });
		const { readable, writable } = new TransformStream();
		const writer = writable.getWriter();
		const encoder = new TextEncoder();
		const body = await request.json<ChatRequestBody>();
		const prompt = body.prompt;
		//TODO: Payload validation, custom prompting, Response cleaning, and post processing
		//TODO: DO VALIDATION
		async function streamResponse() {
			try {
				const response = await gemini.models.generateContentStream({
					model: 'gemini-2.5-flash',
					contents: prompt,
				});

				for await (const chunk of response) {
					if (chunk.text) {
						text += chunk.text;
						const json = JSON.stringify({ token: chunk.text });
						await writer.write(encoder.encode(`data: ${json}\n\n`));
					}
				}
				await env.CURIO_QUESTION_QUEUE.send(text);
				console.log('Generated response for user', text);
			} catch (error) {
				const json = JSON.stringify({
					error: error instanceof Error ? error.message : 'Unknown error',
				});
				await writer.write(encoder.encode(`ERROR: ${json}\n\n`));
			} finally {
				await writer.close();
			}
		}

		ctx.waitUntil(streamResponse());

		return new Response(readable, {
			headers: {
				...cors,
				'Content-Type': 'text/event-stream',
				'Cache-Control': 'no-cache',
				Connection: 'keep-alive',
				'X-Accel-Buffering': 'no',
			},
		});
	},
} satisfies ExportedHandler<Env>;
