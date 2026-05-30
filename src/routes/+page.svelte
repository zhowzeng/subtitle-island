<script lang="ts">
	import { onMount } from 'svelte';
	import SubtitleIsland from '$lib/components/SubtitleIsland.svelte';
	import { startLocalSubtitleSession } from '$lib/local-subtitle-session';
	import {
		clampedAudioLevel,
		conciseSessionErrorMessage,
		visibleTranscriptSegments,
		type SessionState
	} from '$lib/subtitle-session';
	import {
		beginSubtitleIslandDrag,
		closeApp as closeSubIs,
		isTauriRuntime,
		listenForAudioActivity,
		listenForSessionErrors,
		listenForTranscriptUpdates,
		startSubtitleSession,
		stopSubtitleSession,
		type TranscriptSegment
	} from '$lib/tauri';

	let sessionState: SessionState = $state('idle');
	let transcriptLines: TranscriptSegment[] = $state([]);
	let errorMessage = $state('');
	let audioLevel = $state(0);
	let stopLocalSubtitleSession: (() => void) | undefined;
	let unlistenTranscript: (() => void) | undefined;
	let unlistenAudioActivity: (() => void) | undefined;
	let unlistenSessionError: (() => void) | undefined;

	function pushTranscript(segment: TranscriptSegment) {
		transcriptLines = visibleTranscriptSegments(transcriptLines, segment);
	}

	function stopLocalSession() {
		stopLocalSubtitleSession?.();
		stopLocalSubtitleSession = undefined;
		audioLevel = 0;
	}

	async function startSession() {
		errorMessage = '';
		transcriptLines = [];
		audioLevel = 0;
		sessionState = 'starting';

		if (!isTauriRuntime()) {
			sessionState = 'listening';
			stopLocalSubtitleSession = startLocalSubtitleSession({
				pushTranscript,
				setAudioLevel: (level) => {
					audioLevel = level;
				}
			});
			return;
		}

		try {
			await startSubtitleSession();
			sessionState = 'listening';
		} catch (error) {
			sessionState = 'error';
			errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
		}
	}

	async function stopSession() {
		stopLocalSession();

		if (isTauriRuntime()) {
			try {
				await stopSubtitleSession();
			} catch (error) {
				sessionState = 'error';
				errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
				return;
			}
		}

		sessionState = 'idle';
		audioLevel = 0;
	}

	async function closeApp() {
		try {
			await closeSubIs();
		} catch (error) {
			errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			sessionState = 'error';
		}
	}

	async function startWindowDrag(event: PointerEvent) {
		const target = event.target instanceof Element ? event.target : undefined;

		if (event.button !== 0 || target?.closest('button, a, input, textarea, select')) {
			return;
		}

		event.preventDefault();
		await beginSubtitleIslandDrag().catch((error) => {
			errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			sessionState = 'error';
		});
	}

	onMount(() => {
		if (!isTauriRuntime()) {
			return;
		}

		listenForTranscriptUpdates((segment) => {
			pushTranscript(segment);
		})
			.then((unlisten) => {
				unlistenTranscript = unlisten;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			});

		listenForAudioActivity((activity) => {
			audioLevel = clampedAudioLevel(activity);
		})
			.then((unlisten) => {
				unlistenAudioActivity = unlisten;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			});

		listenForSessionErrors((error) => {
			sessionState = 'error';
			errorMessage = conciseSessionErrorMessage(error.message);
			audioLevel = 0;
		})
			.then((unlisten) => {
				unlistenSessionError = unlisten;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			});

		return () => {
			stopLocalSession();
			void stopSubtitleSession().catch(() => undefined);
			unlistenTranscript?.();
			unlistenAudioActivity?.();
			unlistenSessionError?.();
		};
	});
</script>

<svelte:head>
	<title>SubIs</title>
</svelte:head>

<SubtitleIsland
	{sessionState}
	{transcriptLines}
	{errorMessage}
	{audioLevel}
	onStartSession={startSession}
	onStopSession={stopSession}
	onCloseApp={closeApp}
	onStartWindowDrag={startWindowDrag}
/>
