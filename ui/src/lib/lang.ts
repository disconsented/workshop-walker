export function whichLang(lang: Number): String {
	switch (lang) {
		case 1:
			return 'English';
		case 2:
			return 'Russian';
		case 3:
			return 'Chinese';
		case 4:
			return 'Japanese';
		case 5:
			return 'Korean';
		case 6:
			return 'Spanish';
		case 7:
			return 'Portuguese';
		default:
			return 'Unknown';
	}
}
export function inToLangShort(int: number) {
	switch (int) {
		case 1:
			return 'EN';
		case 2:
			return 'RU';
		case 3:
			return 'CN';
		case 4:
			return 'JP';
		case 5:
			return 'KR';
		case 6:
			return 'ES';
		case 7:
			return 'PT';
	}
}
