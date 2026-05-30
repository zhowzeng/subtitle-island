<script lang="ts">
	import { onMount } from 'svelte';
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

	type SessionState = 'idle' | 'starting' | 'listening' | 'error';

	let sessionState: SessionState = $state('idle');
	let transcriptLines: TranscriptSegment[] = $state([]);
	let errorMessage = $state('');
	let audioLevel = $state(0);
	let localMockTimer: ReturnType<typeof setInterval> | undefined = $state();
	let unlistenTranscript: (() => void) | undefined;
	let unlistenAudioActivity: (() => void) | undefined;
	let unlistenSessionError: (() => void) | undefined;

	const fallbackLines = [
		'Testing the subtitle island layout.',
		'Live transcript updates should stay readable.',
		'Only the latest lines remain visible.',
		'Start and stop control the session state.'
	];

	function statusTitle() {
		if (sessionState === 'starting') return 'Starting';
		if (sessionState === 'listening') return 'Listening';
		if (sessionState === 'error') return 'Needs attention';
		return 'Idle';
	}

	function statusDetail() {
		if (sessionState === 'starting') return 'Connecting API';
		if (sessionState === 'listening') return `Microphone ${Math.round(audioLevel * 100)}% - API connected`;
		if (sessionState === 'error') return 'Session stopped';
		return 'Microphone ready';
	}

	function conciseErrorMessage(message: string) {
		const trimmed = message.trim();

		if (!trimmed) return 'Subtitle session failed.';
		if (trimmed.includes('OPENAI_API_KEY')) return 'Missing OpenAI API key.';
		if (trimmed.includes('default microphone')) return 'No default microphone was found.';
		if (trimmed.includes('microphone') || trimmed.includes('Microphone')) {
			return 'Microphone capture failed.';
		}
		if (trimmed.includes('OpenAI Realtime') || trimmed.includes('Realtime')) {
			return 'Could not connect to the transcription API.';
		}
		if (trimmed.length > 96) return `${trimmed.slice(0, 93)}...`;

		return trimmed;
	}

	function pushTranscript(segment: TranscriptSegment) {
		const existingIndex = transcriptLines.findIndex((line) => line.id === segment.id);

		if (existingIndex === -1) {
			transcriptLines = [...transcriptLines, segment].slice(-2);
			return;
		}

		transcriptLines = transcriptLines.map((line, index) => (index === existingIndex ? segment : line));
	}

	function startLocalMock() {
		let index = 0;

		audioLevel = 0.28;

		pushTranscript({
			id: `local-${Date.now()}`,
			audioSource: 'microphone',
			text: fallbackLines[index],
			sourceLanguage: 'en',
			isFinal: false,
			timestamp: Date.now()
		});

		localMockTimer = setInterval(() => {
			index = (index + 1) % fallbackLines.length;
			audioLevel = 0.18 + (index % 3) * 0.2;
			pushTranscript({
				id: `local-${Date.now()}`,
				audioSource: 'microphone',
				text: fallbackLines[index],
				sourceLanguage: 'en',
				isFinal: index % 2 === 0,
				timestamp: Date.now()
			});
		}, 1200);
	}

	function stopLocalMock() {
		if (localMockTimer) {
			clearInterval(localMockTimer);
			localMockTimer = undefined;
		}
		audioLevel = 0;
	}

	async function startSession() {
		errorMessage = '';
		transcriptLines = [];
		audioLevel = 0;
		sessionState = 'starting';

		if (!isTauriRuntime()) {
			sessionState = 'listening';
			startLocalMock();
			return;
		}

		try {
			await startSubtitleSession();
			sessionState = 'listening';
		} catch (error) {
			sessionState = 'error';
			errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
		}
	}

	async function stopSession() {
		stopLocalMock();

		if (isTauriRuntime()) {
			try {
				await stopSubtitleSession();
			} catch (error) {
				sessionState = 'error';
				errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
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
			errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
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
			errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
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
				errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
			});

		listenForAudioActivity((activity) => {
			audioLevel = Math.max(0, Math.min(activity.level, 1));
		})
			.then((unlisten) => {
				unlistenAudioActivity = unlisten;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
			});

		listenForSessionErrors((error) => {
			sessionState = 'error';
			errorMessage = conciseErrorMessage(error.message);
			audioLevel = 0;
		})
			.then((unlisten) => {
				unlistenSessionError = unlisten;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = conciseErrorMessage(error instanceof Error ? error.message : String(error));
			});

		return () => {
			stopLocalMock();
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

<main class="island-shell" data-state={sessionState}>
	<section
		class="subtitle-island"
		aria-label="Subtitle island"
		data-tauri-drag-region
		onpointerdown={startWindowDrag}
	>
		<div class="status-row" role="group" aria-label="Window controls">
			<div class="status-pill">
				<span class="status-dot"></span>
				<span>{statusTitle()}</span>
			</div>
			<div class="window-actions">
				<span class="audio-source-label">{statusDetail()}</span>
				<div class="control-row" aria-label="Session controls">
					<button
						type="button"
						onclick={startSession}
						disabled={sessionState === 'starting' || sessionState === 'listening'}>Start</button
					>
					<button type="button" onclick={stopSession} disabled={sessionState === 'idle'}>Stop</button>
				</div>
				<button class="window-close" type="button" aria-label="Close app" onclick={closeApp}>X</button>
			</div>
		</div>

		<div class="audio-meter" aria-label="Microphone activity">
			<span style:transform={`scaleX(${audioLevel})`}></span>
		</div>

		<div class="subtitle-lines" aria-live="polite">
			{#if sessionState === 'error'}
				<p class="error-line">{errorMessage || 'Session failed.'}</p>
			{:else if transcriptLines.length === 0}
				<p class="muted-line">Ready for live subtitles.</p>
			{:else}
				{#each transcriptLines as line (line.id)}
					<p class:final-line={line.isFinal}>{line.text}</p>
				{/each}
			{/if}
		</div>

	</section>
</main>
