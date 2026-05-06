import { env, createExecutionContext, waitOnExecutionContext, SELF } from 'cloudflare:test';
import { describe, it, expect } from 'vitest';
import worker from '../src/index';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

describe('chat worker', () => {
	it('handles CORS preflight requests', async () => {
		const request = new IncomingRequest('http://example.com/chat', {
			method: 'OPTIONS',
		});
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		expect(response.status).toBe(204);
		expect(response.headers.get('Access-Control-Allow-Methods')).toBe('POST, OPTIONS');
	});

	it('rejects non-POST requests', async () => {
		const response = await SELF.fetch('https://example.com');

		expect(response.status).toBe(405);
		expect(await response.text()).toBe('Method not allowed');
	});
});
