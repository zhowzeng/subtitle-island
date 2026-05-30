<script lang="ts">
	import type { SessionState } from '$lib/subtitle-session';
	import { sessionStatusDetail, sessionStatusTitle } from '$lib/subtitle-session';
	import type { TranscriptSegment } from '$lib/tauri';

	type Props = {
		sessionState: SessionState;
		transcriptLines: TranscriptSegment[];
		errorMessage: string;
		audioLevel: number;
		onStartSession: () => void;
		onStopSession: () => void;
		onCloseApp: () => void;
		onStartWindowDrag: (event: PointerEvent) => void;
	};

	let {
		sessionState,
		transcriptLines,
		errorMessage,
		audioLevel,
		onStartSession,
		onStopSession,
		onCloseApp,
		onStartWindowDrag
	}: Props = $props();
</script>

<main class="island-shell" data-state={sessionState}>
	<section
		class="subtitle-island"
		aria-label="Subtitle island"
		data-tauri-drag-region
		onpointerdown={onStartWindowDrag}
	>
		<div class="status-row" role="group" aria-label="Window controls">
			<div class="status-pill">
				<span class="status-dot"></span>
				<span>{sessionStatusTitle(sessionState)}</span>
			</div>
			<div class="window-actions">
				<span class="audio-source-label">{sessionStatusDetail(sessionState, audioLevel)}</span>
				<div class="control-row" aria-label="Session controls">
					<button
						type="button"
						onclick={onStartSession}
						disabled={sessionState === 'starting' || sessionState === 'listening'}>Start</button
					>
					<button type="button" onclick={onStopSession} disabled={sessionState === 'idle'}>Stop</button>
				</div>
				<button class="window-close" type="button" aria-label="Close app" onclick={onCloseApp}>X</button>
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
