# Tauri Frontend and Backend Boundary

本文件定義 Tauri app 中 `src/` 與 `src-tauri/` 的責任邊界。目標是讓新增功能時能一致判斷：哪些程式應留在 WebView frontend，哪些程式必須進入 Rust backend。

## Audience and Scope

本文適用於開發本專案的 Tauri frontend、backend、plugin、audio、transcription、subtitle export 與 app setting 功能。

本文不定義 UI 設計規範、資料模型細節、dependency 管理流程，或 Tauri plugin 的完整安裝步驟。

## Core Boundary

`src/` 是 WebView frontend。它負責 UI、使用者互動、顯示用狀態，以及不需要 native 權限的 client-side 邏輯。

`src-tauri/` 是 native backend。它負責系統權限、持久化、Tauri command、app lifecycle、plugin registration，以及需要信任邊界保護的邏輯。

Frontend 只能透過 Tauri IPC 呼叫 backend 暴露且授權的 command。Backend command 不應把任意 filesystem、process 或 native API 能力直接轉交給 frontend。

## Put UI and Client Logic in `src/`

把符合下列條件的程式放在 `src/`：

- 只影響 UI 呈現或使用者互動。
- 不需要直接存取 OS、filesystem、process 或 native API。
- 不處理 secret、private path、license、token 或其他敏感資料。
- 不需要在 app 重啟後保留為權威資料。
- 即使 frontend 被惡意腳本控制，也不會造成系統資源外洩或權限提升。
- 可用 browser 或 WebView API 安全完成。

常見例子：

- Svelte component、route、layout、CSS。
- UI state，例如目前選取項目、表單輸入、modal 開關、loading state、error display。
- 顯示用資料轉換，例如 sorting、filtering、時間格式化、view model mapping。
- Tauri command 的 client wrapper，例如集中封裝 `invoke(...)`。
- 純視覺或互動邏輯，例如字幕行高、目前高亮文字、控制列狀態。

## Put Native and Trusted Logic in `src-tauri/`

把符合下列任一條件的程式放在 `src-tauri/`：

- 需要 OS、filesystem、process、window、tray、menu、global shortcut 或 native plugin 能力。
- 需要讀寫 app data、設定檔、cache、project metadata 或 job state。
- 需要保護 secret、private path、license、token 或其他可信任資料。
- 需要限制、驗證或審計 frontend 對系統資源的存取。
- 執行時間長、CPU-heavy，或可能造成 WebView 卡頓。
- 涉及 app lifecycle、Tauri permissions、capabilities、plugin registration 或 native event。

常見例子：

- Tauri commands。
- App-level state 與 session state。
- Filesystem 讀寫、路徑解析、路徑正規化與 scope 驗證。
- Database、config file、cache、recent projects。
- 呼叫外部 binary 或 native library。
- Audio capture、transcription pipeline、subtitle export。
- Window configuration、menu、tray、global shortcut。
- Tauri plugin 註冊與 `src-tauri/capabilities/` 權限設定。

## Decide Where New Code Belongs

新增功能時，依序使用以下規則。

1. 如果功能需要 OS 或 native 權限，放在 `src-tauri/`。
2. 如果功能需要讀寫本機檔案或 app data，放在 `src-tauri/`。
3. 如果資料需要在 app 重啟後保留為權威狀態，放在 `src-tauri/` 管理。
4. 如果資料不應暴露給 WebView，放在 `src-tauri/`。
5. 如果 frontend 被 XSS 攻破後濫用該能力會造成傷害，放在 `src-tauri/`，並縮小 command 介面。
6. 如果功能只處理 UI 呈現、使用者互動或非敏感 client-side state，放在 `src/`。
7. 如果功能是 heavy work 或長時間工作，優先放在 `src-tauri/`，並用 event 回報進度。
8. 如果兩邊都能實作，選擇權限較小、資料流較清楚、測試成本較低的位置。

## Design Narrow Commands

Tauri command 是 frontend 進入 backend 的信任邊界。Command 應描述 app-level 行為，不應暴露任意系統操作。

偏好的 command 介面：

```rust
load_project(project_id)
export_subtitles(session_id, format)
start_transcription_session(options)
stop_transcription_session(session_id)
```

避免的 command 介面：

```rust
read_file(path)
write_file(path, contents)
run_command(command, args)
set_config(key, value)
```

如果 command 必須接受 path、command、key 或其他自由輸入，backend 必須限制 scope，並使用型別化輸入、白名單或明確驗證。

## Assign State Ownership

Frontend 可以持有暫時 UI state。Backend 應持有跨視窗、跨 command、跨重啟，或需要權限保護的 app state。

放在 frontend 的 state：

- 目前頁面或 tab。
- 控制項輸入值。
- 暫時選取狀態。
- loading state 與 error display。
- 顯示用 derived state。

放在 backend 的 state：

- transcription session 是否正在執行。
- microphone 或 audio pipeline 狀態。
- project、file、export job 的權威狀態。
- recent projects 與 app config。
- 需要跨視窗、跨重啟或跨 command 一致的資料。

## Apply the Boundary to Subtitle Island

放在 `src/`：

- 字幕島 UI。
- Start/Stop button 狀態。
- 目前顯示的字幕文字。
- 字幕行數、字級、透明度、位置等顯示偏好。
- 接收 backend event 後更新畫面。

放在 `src-tauri/`：

- `start_session`、`stop_session` 等 commands。
- Session lifecycle 與防止重複啟動。
- Microphone capture。
- Realtime transcription connection。
- Transcript event emission。
- API key 讀取與保護。
- 設定檔讀寫。
- Subtitle file export。

## Verify Boundary Changes

新增或移動功能後，至少執行以下檢查：

- Frontend type/check：`bun run check`。
- Backend Rust check：從 `src-tauri/` 執行 `cargo check --locked`。
- 新增 Tauri plugin 時，確認 Rust registration 與 `src-tauri/capabilities/` 權限設定。
- 新增 command 時，確認 command 名稱、輸入型別、權限邊界與錯誤回傳都清楚。
