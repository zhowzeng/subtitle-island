import type { AudioSource, TranscriptSegment } from './tauri';

const fallbackLiveSubtitles = [
	{ text: 'Testing the subtitle island layout.', translation: '正在測試字幕島的版面。' },
	{
		text: 'Live transcript updates should stay readable even when a translated line wraps.',
		translation: '即使翻譯後的字幕換行，即時逐字稿的更新也應該保持清楚易讀，不會和下一行擠在一起。'
	},
	{ text: 'Only the latest lines remain visible.', translation: '只會保留最新的幾行字幕。' },
	{
		text: 'Start and stop control the session state.',
		translation: '開始與停止按鈕控制字幕工作階段的狀態。'
	}
];

type LocalSubtitleSessionOptions = {
	audioSource: AudioSource;
	translate: boolean;
	pushTranscript: (segment: TranscriptSegment) => void;
	setAudioLevel: (level: number) => void;
};

export function startLocalSubtitleSession({
	audioSource,
	translate,
	pushTranscript,
	setAudioLevel
}: LocalSubtitleSessionOptions) {
	let index = 0;

	setAudioLevel(0.28);

	pushTranscript({
		id: `local-${Date.now()}`,
		audioSource,
		text: fallbackLiveSubtitles[index].text,
		sourceLanguage: 'en',
		isFinal: false,
		timestamp: Date.now()
	});

	const timer = setInterval(() => {
		index = (index + 1) % fallbackLiveSubtitles.length;
		const isFinal = index % 2 === 1;
		setAudioLevel(0.18 + (index % 3) * 0.2);
		pushTranscript({
			id: `local-${Date.now()}`,
			audioSource,
			text: fallbackLiveSubtitles[index].text,
			translation: translate && isFinal ? fallbackLiveSubtitles[index].translation : undefined,
			sourceLanguage: 'en',
			isFinal,
			timestamp: Date.now()
		});
	}, 1200);

	return () => {
		clearInterval(timer);
		setAudioLevel(0);
	};
}
