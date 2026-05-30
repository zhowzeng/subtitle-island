import type { TranscriptSegment } from './tauri';

const fallbackLiveSubtitles = [
	'Testing the subtitle island layout.',
	'Live transcript updates should stay readable.',
	'Only the latest lines remain visible.',
	'Start and stop control the session state.'
];

type LocalSubtitleSessionOptions = {
	pushTranscript: (segment: TranscriptSegment) => void;
	setAudioLevel: (level: number) => void;
};

export function startLocalSubtitleSession({
	pushTranscript,
	setAudioLevel
}: LocalSubtitleSessionOptions) {
	let index = 0;

	setAudioLevel(0.28);

	pushTranscript({
		id: `local-${Date.now()}`,
		audioSource: 'microphone',
		text: fallbackLiveSubtitles[index],
		sourceLanguage: 'en',
		isFinal: false,
		timestamp: Date.now()
	});

	const timer = setInterval(() => {
		index = (index + 1) % fallbackLiveSubtitles.length;
		setAudioLevel(0.18 + (index % 3) * 0.2);
		pushTranscript({
			id: `local-${Date.now()}`,
			audioSource: 'microphone',
			text: fallbackLiveSubtitles[index],
			sourceLanguage: 'en',
			isFinal: index % 2 === 0,
			timestamp: Date.now()
		});
	}, 1200);

	return () => {
		clearInterval(timer);
		setAudioLevel(0);
	};
}
