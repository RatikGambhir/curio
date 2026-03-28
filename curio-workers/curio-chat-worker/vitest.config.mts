import { defineConfig } from 'vitest/config';
import { cloudflarePool } from '@cloudflare/vitest-pool-workers';

export default defineConfig({
	plugins: [
		cloudflarePool({
			wrangler: { configPath: './wrangler.jsonc' },
		}),
	],
});
