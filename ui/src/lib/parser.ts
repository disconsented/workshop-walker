// I really, really, really despise how JS doesn't support large integers properly.
export const parseSafeJSON = (data: string) => {
	return JSON.parse(data, reviver);
};
export const reviver = (key: any, value: any, context: any) => {
	// If _anything_ isn't a safe int, that's actually an int, treat it as a big int
	if (typeof value === 'number' && Number.isInteger(value) && !Number.isSafeInteger(value)) {
		return BigInt(context.source);
	}
	return value;
};

// The other half of the problem: JSON.stringify refuses a BigInt. Put a token in
// place of each one, then write the digits back over the token. The token holds
// a nonce because a string in the data could otherwise match it.
export const stringifySafeJSON = (data: unknown): string => {
	const nonce = crypto.randomUUID();
	const bigints = new Map<string, string>();

	const json = JSON.stringify(data, (_key, value) => {
		if (typeof value !== 'bigint') {
			return value;
		}
		const token = `${nonce}:${bigints.size}`;
		bigints.set(token, value.toString());
		return token;
	});

	return json.replace(new RegExp(`"${nonce}:(\\d+)"`, 'g'), (match, index) => {
		return bigints.get(`${nonce}:${index}`) ?? match;
	});
};
