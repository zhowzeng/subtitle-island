# SubIs PRD

## 1. Product Overview

### Product Name

SubIs

### Product Vision

提供一個 local-first 的桌面即時字幕工具，可在：

* 線上會議
* 現場演講
* 語音通話
* YouTube / Podcast

即時顯示字幕與翻譯，並以低干擾的懸浮字幕島形式存在於桌面。

核心特性：

* 即時字幕
* 即時翻譯
* 桌面 overlay
* local audio capture
* 不依賴 meeting bot
* 不依賴會議平台 API

---

# 2. Goals

## Primary Goals

### G1

支援即時語音轉文字（ASR）

### G2

支援即時翻譯成繁體中文

### G3

支援桌面懸浮字幕島

### G4

支援：

* 線上會議
* 現場演講

### G5

低延遲（目標 < 2 秒）

---

# 3. Non-Goals (MVP)

以下功能不在 MVP：

* speaker diarization
* 多人身份辨識
* cloud sync
* account system
* 團隊協作
* AI summary
* meeting bot
* Zoom/Meet 官方整合
* transcript search
* mobile app

---

# 4. Target Users

## Primary User

開發者本人

## Secondary Users

技術使用者：

* AI engineers
* remote workers
* bilingual users
* conference attendees

---

# 5. User Scenarios

## Scenario A — Online Meeting Translation

使用者正在參加英文 Google Meet。

系統：

1. 擷取 system audio
2. 即時 ASR
3. 翻譯成繁中
4. 顯示字幕島

---

## Scenario B — Live Speech Subtitle

使用者參加現場演講。

系統：

1. 使用 microphone
2. 即時 ASR
3. 必要時翻譯
4. 顯示字幕島

---

## Scenario C — YouTube Translation

使用者觀看英文影片。

系統：

1. 擷取 system audio
2. 即時翻譯
3. 顯示中文字幕

---

# 6. Functional Requirements

# 6.1 Audio Input

## FR-001

支援 microphone capture

## FR-002

支援 system audio capture

### Windows

WASAPI loopback

### macOS

BlackHole / ScreenCaptureKit

---

# 6.2 Speech Recognition

## FR-003

支援 OpenAI Realtime transcription API

## FR-004

支援 partial transcript

## FR-005

支援 final transcript

## FR-006

支援自動語言偵測

---

# 6.3 Translation

## FR-007

當來源語言非中文時：
翻譯為繁體中文

## FR-008

支援：

* 原文模式
* 中文模式
* 雙語模式

---

# 6.4 Subtitle Island

## FR-009

提供 always-on-top overlay window

## FR-010

支援透明背景

## FR-011

支援拖曳位置

## FR-012

支援字體大小調整

## FR-013

支援 opacity 調整

## FR-014

支援 click-through mode

---

# 6.5 Session Control

## FR-015

支援：

* Start session
* Stop session

## FR-016

顯示目前音訊來源

## FR-017

顯示目前語言

---

# 7. Non-Functional Requirements

## NFR-001 Latency

目標：

* partial transcript < 1 sec
* translated subtitle < 2 sec

---

## NFR-002 Resource Usage

MVP 不跑本地模型。

主要資源：

* audio capture
* websocket
* rendering

---

## NFR-003 Privacy

音訊會送至 OpenAI API。

需明確提示：

* currently recording
* using external transcription API

---

# 8. Technical Architecture

## Frontend

### Stack

* SvelteKit
* TypeScript
* Tailwind

### Responsibilities

* subtitle island UI
* settings
* transcript rendering

---

## Desktop Shell

### Stack

* Tauri v2
* Rust

### Responsibilities

* audio capture
* native window control
* OS integrations

---

## Backend Service

### Responsibilities

* audio chunking
* websocket streaming
* OpenAI API bridge
* translation pipeline

---

# 9. Audio Pipeline

```text
Audio Source
    ↓
PCM Stream
    ↓
Chunking
    ↓
OpenAI Realtime API
    ↓
Transcript Delta
    ↓
Translation
    ↓
Subtitle Renderer
```

---

# 10. UI Specification

# 10.1 Subtitle Island

## Layout

```text
┌────────────────────────────┐
│ Original transcript         │
│ 中文翻譯                     │
└────────────────────────────┘
```

---

## States

### Idle

未錄音

### Listening

正在接收音訊

### Translating

翻譯中

### Error

音訊/API 錯誤

---

# 11. Settings

## Audio

* microphone selector
* system audio selector

## Translation

* auto translate
* target language

## UI

* font size
* opacity
* width
* position

---

# 12. OpenAI Integration

## Realtime Transcription

### Model

gpt-realtime-whisper

### Input

PCM16 mono audio

### Output

transcript deltas

---

## Translation

### Model

gpt-4.1-mini

### Prompt

Translate the following transcript into Traditional Chinese used in Taiwan.
Keep terminology concise and subtitle-friendly.

---

# 13. Data Model

## TranscriptSegment

```ts
type TranscriptSegment = {
  id: string
  source: 'mic' | 'system'
  text: string
  translation?: string
  language: string
  isFinal: boolean
  timestamp: number
}
```

---

# 14. MVP Scope

## Included

* microphone capture
* system audio capture
* realtime ASR
* realtime translation
* subtitle island
* settings panel

---

## Excluded

* transcript persistence
* speaker diarization
* meeting summaries
* AI notes
* cloud sync

---

# 15. Development Phases

# Phase 1

## Goal

現場演講字幕

## Includes

* microphone capture
* realtime transcription
* subtitle island

---

# Phase 2

## Goal

會議翻譯

## Includes

* system audio capture
* translation pipeline

---

# Phase 3

## Goal

雙語字幕與 UX 優化

## Includes

* bilingual subtitles
* settings
* shortcut keys
* click-through mode

---

# 16. Risks

## R1

macOS system audio capture complexity

## R2

Realtime API latency fluctuations

## R3

ASR instability in noisy environments

## R4

Mixed-language speech quality

---

# 17. Success Metrics

## MVP Success

* 可穩定顯示即時字幕
* 延遲 < 2 秒
* 可連續運作 1 小時以上
* 翻譯可閱讀
* UI 不影響正常工作

---

# 18. Future Extensions

## Possible Future Features

* local whisper fallback
* speaker diarization
* transcript history
* AI summary
* RAG search
* meeting export
* OBS integration
* stream overlay mode
* multilingual translation
* mobile companion app
