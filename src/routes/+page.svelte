<script lang="ts">
	import { onMount } from 'svelte';
	import SubtitleIsland from '$lib/components/SubtitleIsland.svelte';
	import { startLocalSubtitleSession } from '$lib/local-subtitle-session';
	import {
		clampedAudioLevel,
		conciseSessionErrorMessage,
		isChineseModeSegment,
		visibleTranscriptSegments,
		type SessionState
	} from '$lib/subtitle-session';
	import {
		beginSubtitleIslandDrag,
		closeApp as closeSubIs,
		defaultSettings,
		getSettings,
		isTauriRuntime,
		listenForAudioActivity,
		listenForSessionErrors,
		listenForTranscriptUpdates,
		listenForTranslationErrors,
		saveSettings,
		startSubtitleSession,
		stopSubtitleSession,
		type AudioSource,
		type DisplayMode,
		type Settings,
		type SourceLanguage,
		type TranscriptSegment
	} from '$lib/tauri';

	let sessionState: SessionState = $state('idle');
	let settings: Settings = $state({ ...defaultSettings });
	let settingsSaved: Promise<void> = Promise.resolve();
	let transcriptLines: TranscriptSegment[] = $state([]);
	let errorMessage = $state('');
	let translationFailed = $state(false);
	let audioLevel = $state(0);
	let stopLocalSubtitleSession: (() => void) | undefined;
	let unlistenTranscript: (() => void) | undefined;
	let unlistenAudioActivity: (() => void) | undefined;
	let unlistenSessionError: (() => void) | undefined;
	let unlistenTranslationError: (() => void) | undefined;

	function pushTranscript(segment: TranscriptSegment) {
		if (segment.translation) {
			translationFailed = false;
		}
		if (settings.displayMode === 'chinese' && !isChineseModeSegment(segment)) {
			return;
		}
		transcriptLines = visibleTranscriptSegments(transcriptLines, segment);
	}

	function stopLocalSession() {
		stopLocalSubtitleSession?.();
		stopLocalSubtitleSession = undefined;
		audioLevel = 0;
	}

	async function startSession() {
		errorMessage = '';
		translationFailed = false;
		transcriptLines = [];
		audioLevel = 0;
		sessionState = 'starting';

		if (!isTauriRuntime()) {
			sessionState = 'listening';
			stopLocalSubtitleSession = startLocalSubtitleSession({
				audioSource: settings.audioSource,
				translate: settings.displayMode !== 'original',
				pushTranscript,
				setAudioLevel: (level) => {
					audioLevel = level;
				}
			});
			return;
		}

		try {
			// The backend starts from the stored settings, so wait for the latest save.
			await settingsSaved;
			await startSubtitleSession();
			sessionState = 'listening';
		} catch (error) {
			sessionState = 'error';
			errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
		}
	}

	function updateSettings(nextSettings: Partial<Settings>) {
		if (sessionState === 'starting' || sessionState === 'listening') {
			return;
		}

		settings = { ...settings, ...nextSettings };

		if (isTauriRuntime()) {
			settingsSaved = saveSettings(settings).catch((error) => {
				sessionState = 'error';
				errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			});
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

		getSettings()
			.then((storedSettings) => {
				settings = storedSettings;
			})
			.catch((error) => {
				sessionState = 'error';
				errorMessage = conciseSessionErrorMessage(error instanceof Error ? error.message : String(error));
			});

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

		listenForTranslationErrors(() => {
			translationFailed = true;
		})
			.then((unlisten) => {
				unlistenTranslationError = unlisten;
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
			unlistenTranslationError?.();
		};
	});
</script>

<svelte:head>
	<title>SubIs</title>
</svelte:head>

<SubtitleIsland
	{sessionState}
	audioSource={settings.audioSource}
	sourceLanguage={settings.sourceLanguage}
	displayMode={settings.displayMode}
	{transcriptLines}
	{errorMessage}
	{translationFailed}
	{audioLevel}
	onSelectAudioSource={(audioSource: AudioSource) => updateSettings({ audioSource })}
	onSelectSourceLanguage={(sourceLanguage: SourceLanguage) => updateSettings({ sourceLanguage })}
	onSelectDisplayMode={(displayMode: DisplayMode) => updateSettings({ displayMode })}
	onStartSession={startSession}
	onStopSession={stopSession}
	onCloseApp={closeApp}
	onStartWindowDrag={startWindowDrag}
/>
