import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type TranscriptSegment = {
	id: string;
	audioSource: 'microphone' | 'system';
	text: string;
	translation?: string;
	sourceLanguage: string;
	isFinal: boolean;
	timestamp: number;
};

export type AudioActivity = {
	level: number;
	peak: number;
	timestamp: number;
};

export type SessionError = {
	message: string;
};

export const isTauriRuntime = () => typeof window !== 'undefined' && isTauri();

export const startSubtitleSession = () => invoke<void>('start_session');

export const stopSubtitleSession = () => invoke<void>('stop_session');

export const closeApp = () => invoke<void>('close_app');

export const beginSubtitleIslandDrag = () => invoke<void>('begin_subtitle_island_drag');

export const listenForTranscriptUpdates = (handler: (segment: TranscriptSegment) => void) =>
	listen<TranscriptSegment>('transcript-update', (event) => handler(event.payload));

export const listenForAudioActivity = (handler: (activity: AudioActivity) => void) =>
	listen<AudioActivity>('audio-activity', (event) => handler(event.payload));

export const listenForSessionErrors = (handler: (error: SessionError) => void) =>
	listen<SessionError>('session-error', (event) => handler(event.payload));
