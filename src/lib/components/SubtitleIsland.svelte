<script lang="ts">
	import type { SessionState } from '$lib/subtitle-session';
	import {
		audioSourceLabel,
		displayModeLabel,
		sessionStatusDetail,
		sessionStatusTitle,
		sourceLanguageLabel
	} from '$lib/subtitle-session';
	import type { AudioSource, DisplayMode, SourceLanguage, TranscriptSegment } from '$lib/tauri';

	type Props = {
		sessionState: SessionState;
		audioSource: AudioSource;
		sourceLanguage: SourceLanguage;
		displayMode: DisplayMode;
		transcriptLines: TranscriptSegment[];
		errorMessage: string;
		translationFailed: boolean;
		audioLevel: number;
		onSelectAudioSource: (audioSource: AudioSource) => void;
		onSelectSourceLanguage: (sourceLanguage: SourceLanguage) => void;
		onSelectDisplayMode: (displayMode: DisplayMode) => void;
		onStartSession: () => void;
		onStopSession: () => void;
		onCloseApp: () => void;
		onStartWindowDrag: (event: PointerEvent) => void;
	};

	let {
		sessionState,
		audioSource,
		sourceLanguage,
		displayMode,
		transcriptLines,
		errorMessage,
		translationFailed,
		audioLevel,
		onSelectAudioSource,
		onSelectSourceLanguage,
		onSelectDisplayMode,
		onStartSession,
		onStopSession,
		onCloseApp,
		onStartWindowDrag
	}: Props = $props();

	const audioSources: AudioSource[] = ['microphone', 'system'];
	const sourceLanguages: SourceLanguage[] = ['auto', 'en', 'zh'];
	// Bilingual Mode is selectable once its layout is rendered.
	const displayModes: DisplayMode[] = ['original', 'chinese'];
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
				<span class="audio-source-label">{sessionStatusDetail(sessionState, audioLevel, audioSource, translationFailed)}</span>
				<div class="source-toggle" aria-label="Audio Source">
					{#each audioSources as source}
						<button
							type="button"
							class:active-source={audioSource === source}
							aria-pressed={audioSource === source}
							disabled={sessionState === 'starting' || sessionState === 'listening'}
							onclick={() => onSelectAudioSource(source)}>{audioSourceLabel(source)}</button
						>
					{/each}
				</div>
				<div class="source-toggle language-toggle" aria-label="Source Language">
					{#each sourceLanguages as language}
						<button
							type="button"
							class:active-source={sourceLanguage === language}
							aria-pressed={sourceLanguage === language}
							disabled={sessionState === 'starting' || sessionState === 'listening'}
							onclick={() => onSelectSourceLanguage(language)}>{sourceLanguageLabel(language)}</button
						>
					{/each}
				</div>
				<div class="source-toggle" aria-label="Display Mode">
					{#each displayModes as mode}
						<button
							type="button"
							class:active-source={displayMode === mode}
							aria-pressed={displayMode === mode}
							disabled={sessionState === 'starting' || sessionState === 'listening'}
							onclick={() => onSelectDisplayMode(mode)}>{displayModeLabel(mode)}</button
						>
					{/each}
				</div>
				<div class="control-row" aria-label="Session controls">
					<button
						type="button"
						onclick={onStartSession}
						disabled={sessionState === 'starting' || sessionState === 'listening'}>Start</button
					>
					<button type="button" onclick={onStopSession} disabled={sessionState === 'idle'}>Stop</button>
				</div>
			</div>
			<button class="window-close" type="button" aria-label="Close app" onclick={onCloseApp}>X</button>
		</div>

		<div class="audio-meter" aria-label={`${audioSourceLabel(audioSource)} activity`}>
			<span style:transform={`scaleX(${audioLevel})`}></span>
		</div>

		<div class="subtitle-lines" aria-live="polite">
			{#if sessionState === 'error'}
				<p class="error-line">{errorMessage || 'Session failed.'}</p>
			{:else if transcriptLines.length === 0}
				<p class="muted-line">Ready for live subtitles.</p>
			{:else}
				{#each transcriptLines as line (line.id)}
					<p class:final-line={line.isFinal}>{line.translation ?? line.text}</p>
				{/each}
			{/if}
		</div>
	</section>
</main>
