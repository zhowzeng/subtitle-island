<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
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

	const isTauriRuntime = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

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
	<section class="subtitle-island" aria-label="Subtitle island">
		<div class="status-row">
			<div class="status-pill">
				<span class="status-dot"></span>
				<span>{sessionState === 'listening' ? 'Listening' : sessionState === 'error' ? 'Error' : 'Idle'}</span>
			</div>
			<span class="source-label">Microphone</span>
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
