import { describe, expect, it } from 'vitest';
import { parseSafeJSON, stringifySafeJSON } from './parser';

describe('stringifySafeJSON', () => {
	it('writes a BigInt as a JSON number', () => {
		expect(stringifySafeJSON({ id: 76561197960287930n, admin: true })).toBe(
			'{"id":76561197960287930,"admin":true}'
		);
	});

	it('keeps a string that looks like the placeholder', () => {
		const data = { a: 1n, note: '__bigint_0__' };

		expect(parseSafeJSON(stringifySafeJSON(data))).toStrictEqual({ a: 1, note: '__bigint_0__' });
	});

	it('reaches a BigInt inside an array or an object', () => {
		expect(stringifySafeJSON({ nested: { list: [1n, 'x', 2n] } })).toBe(
			'{"nested":{"list":[1,"x",2]}}'
		);
	});

	it('round-trips a Steam ID', () => {
		const id = 76561197960287930n;

		expect(parseSafeJSON(stringifySafeJSON({ id })).id).toBe(id);
	});
});
