import type { AudioActivity, AudioSource, TranscriptSegment } from './tauri';

export type SessionState = 'idle' | 'starting' | 'listening' | 'error';

export function sessionStatusTitle(sessionState: SessionState) {
	if (sessionState === 'starting') return 'Starting';
	if (sessionState === 'listening') return 'Listening';
	if (sessionState === 'error') return 'Needs attention';
	return 'Idle';
}

export function audioSourceLabel(audioSource: AudioSource) {
	if (audioSource === 'system') return 'System Audio';
	return 'Microphone';
}

export function sessionStatusDetail(
	sessionState: SessionState,
	audioLevel: number,
	audioSource: AudioSource,
	translationFailed: boolean
) {
	const sourceLabel = audioSourceLabel(audioSource);

	if (sessionState === 'starting') return 'Connecting API';
	if (sessionState === 'listening') {
		const apiStatus = translationFailed ? 'Translation failed' : 'API connected';
		return `${sourceLabel} ${Math.round(audioLevel * 100)}% - ${apiStatus}`;
	}
	if (sessionState === 'error') return 'Session stopped';
	return `${sourceLabel} ready`;
}

export function conciseSessionErrorMessage(message: string) {
	const trimmed = message.trim();

	if (!trimmed) return 'Subtitle session failed.';
	if (trimmed.includes('OPENAI_API_KEY')) return 'Missing OpenAI API key.';
	if (trimmed.includes('default microphone')) return 'No default microphone was found.';
	if (trimmed.includes('default system audio') || trimmed.includes('System Audio Capture')) {
		return 'System Audio Capture is unavailable.';
	}
	if (trimmed.includes('system audio')) return 'System Audio Capture failed.';
	if (trimmed.includes('microphone') || trimmed.includes('Microphone')) {
		return 'Microphone capture failed.';
	}
	if (trimmed.includes('OpenAI Realtime') || trimmed.includes('Realtime')) {
		return 'Could not connect to the transcription API.';
	}
	if (trimmed.length > 96) return `${trimmed.slice(0, 93)}...`;

	return trimmed;
}

export function visibleTranscriptSegments(
	currentSegments: TranscriptSegment[],
	nextSegment: TranscriptSegment
) {
	const existingIndex = currentSegments.findIndex((line) => line.id === nextSegment.id);

	if (existingIndex === -1) {
		return [...currentSegments, nextSegment].slice(-2);
	}

	return currentSegments.map((line, index) => (index === existingIndex ? nextSegment : line));
}

export function clampedAudioLevel(activity: AudioActivity) {
	return Math.max(0, Math.min(activity.level, 1));
}
