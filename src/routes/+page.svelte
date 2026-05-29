<script lang="ts">
	import { invoke, isTauri } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	type SessionState = 'idle' | 'listening' | 'error';

	type TranscriptSegment = {
		id: string;
		source: 'mic' | 'system';
		text: string;
		translation?: string;
		language: string;
		isFinal: boolean;
		timestamp: number;
	};

	let sessionState: SessionState = $state('idle');
	let transcriptLines: TranscriptSegment[] = $state([]);
	let errorMessage = $state('');
	let localMockTimer: ReturnType<typeof setInterval> | undefined = $state();
	let unlistenTranscript: (() => void) | undefined;

	const fallbackLines = [
		'Testing the subtitle island layout.',
		'Live transcript updates should stay readable.',
		'Only the latest lines remain visible.',
		'Start and stop control the session state.'
	];

	const isTauriRuntime = () => typeof window !== 'undefined' && isTauri();

	function pushTranscript(segment: TranscriptSegment) {
		transcriptLines = [...transcriptLines, segment].slice(-2);
	}

	function startLocalMock() {
		let index = 0;

		pushTranscript({
			id: `local-${Date.now()}`,
			source: 'mic',
			text: fallbackLines[index],
			language: 'en',
			isFinal: false,
			timestamp: Date.now()
		});

		localMockTimer = setInterval(() => {
			index = (index + 1) % fallbackLines.length;
			pushTranscript({
				id: `local-${Date.now()}`,
				source: 'mic',
				text: fallbackLines[index],
				language: 'en',
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
	}

	async function startSession() {
		errorMessage = '';
		transcriptLines = [];
		sessionState = 'listening';

		if (!isTauriRuntime()) {
			startLocalMock();
			return;
		}

		try {
			await invoke('start_session');
		} catch (error) {
			sessionState = 'error';
			errorMessage = error instanceof Error ? error.message : String(error);
		}
	}

	async function stopSession() {
		stopLocalMock();

		if (isTauriRuntime()) {
			try {
				await invoke('stop_session');
			} catch (error) {
				sessionState = 'error';
				errorMessage = error instanceof Error ? error.message : String(error);
				return;
			}
		}

		sessionState = 'idle';
	}

	async function closeApp() {
		try {
			await invoke('close_window');
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
			sessionState = 'error';
		}
	}

	async function startWindowDrag(event: PointerEvent) {
		const target = event.target instanceof Element ? event.target : undefined;

		if (event.button !== 0 || target?.closest('button, a, input, textarea, select')) {
			return;
		}

		event.preventDefault();
		await invoke('start_window_drag').catch((error) => {
			errorMessage = error instanceof Error ? error.message : String(error);
			sessionState = 'error';
		});
	}

	onMount(() => {
		if (!isTauriRuntime()) {
			return;
		}

		listen<TranscriptSegment>('transcript-update', (event) => {
			pushTranscript(event.payload);
		})
			.then((unlisten) => {
				unlistenTranscript = unlisten;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = error instanceof Error ? error.message : String(error);
			});

		return () => {
			stopLocalMock();
			void invoke('stop_session').catch(() => undefined);
			unlistenTranscript?.();
		};
	});
</script>

<svelte:head>
	<title>Subtitle Island</title>
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
				<span>{sessionState === 'listening' ? 'Listening' : sessionState === 'error' ? 'Error' : 'Idle'}</span>
			</div>
			<div class="window-actions">
				<span class="source-label">Microphone</span>
				<button class="window-close" type="button" aria-label="Close app" onclick={closeApp}>X</button>
			</div>
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

		<div class="control-row">
			<button type="button" onclick={startSession} disabled={sessionState === 'listening'}>Start</button>
			<button type="button" onclick={stopSession} disabled={sessionState === 'idle'}>Stop</button>
		</div>
	</section>
</main>
