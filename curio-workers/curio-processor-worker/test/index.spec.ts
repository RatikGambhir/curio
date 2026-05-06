import { describe, it, expect } from 'vitest';
import worker from '../src/index';

describe('processor worker', () => {
	it('exports a queue handler', () => {
		expect(worker.queue).toBeTypeOf('function');
	});
});
