import { GoogleGenAI } from '@google/genai';

export default {
	async fetch(request, env, ctx) {
		const gemini = new GoogleGenAI({ apiKey: env.GEMINI_API_KEY });
		const { readable, writable } = new TransformStream();
		const writer = writable.getWriter();
		const encoder = new TextEncoder();
		const body = await request.json();
		//TODO: Payload validation, custom prompting, Response cleaning, and post processing
		async function streamResponse() {
			try {
				const response = await gemini.models.generateContentStream({
					model: 'gemini-2.5-flash',
					contents: body.prompt,
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
};
