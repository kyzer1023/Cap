# Five contribution proposals for Cap’s SDKs, Studio editor, Wayland screenshots, and status history

This draft proposes five independent changes based on a review of [Cap main at `031ad14ec`](https://github.com/CapSoftware/Cap/commit/031ad14ec6f3024a193d95c517d06210d9368c92). Each can ship and be verified separately. I’d start with the embed SDK because the upload API already returns the exact URL the player needs. The status-history fix is the smallest option with a deterministic reproduction.

## 1. Let the embed SDK consume the URL returned by a developer upload

A developer upload returns both a video ID and an embed URL. The server stores the video in `developerVideos` and builds the embed URL as `/dev/{playbackId}/embed`. The React and vanilla SDK entry points both use a builder that takes a video ID and constructs `/embed/{videoId}`. That route looks up an ordinary Cap in the `videos` table. A developer who passes the video ID from a successful upload to the embed SDK therefore gets a player URL that cannot resolve that developer recording. [Upload response](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/web/lib/infra-video-upload.ts#L493-L517) · [SDK builder](https://github.com/CapSoftware/Cap/blob/031ad14ec/packages/sdk-embed/src/vanilla/cap-embed.ts#L5-L15) · [Player lookup](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/web/app/embed/%5BvideoId%5D/page.tsx#L172)

**Proposed change.** Let callers pass the returned `embedUrl` to either SDK entry point. Preserve the existing video-ID form for ordinary Cap share links. Using the returned URL also preserves the host on self-hosted deployments.

**Acceptance check.** Use an upload fixture with distinct video and playback IDs and a self-hosted origin. Both entry points should produce the issued developer player URL. An ordinary Cap embed should retain its existing path and autoplay behavior.

## 2. Give recorder SDK users a way to save a finished take after upload creation fails

By the time `createUpload()` runs, the SDK has received the encoder’s final chunk, released capture devices, and assembled a playable blob. If the host application’s callback rejects, the SDK emits an error and clears the chunks. The host learns that the upload failed but has no public way to offer “Save recording.” The Chrome extension already offers a recovery path for this situation. [Recorder lifecycle](https://github.com/CapSoftware/Cap/blob/031ad14ec/packages/sdk-recorder/src/index.ts#L211-L302) · [Public error event](https://github.com/CapSoftware/Cap/blob/031ad14ec/packages/sdk-recorder/src/types.ts#L36-L41) · [Extension recovery](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/chrome-extension/src/offscreen/recorder.ts#L1422)

**Proposed change.** Expose the finished blob to the host on upload-creation failure, for example as an optional field on the error event. The host can then offer a download and decide how long to retain the blob. Limit the first change to this failure point. A lost response during multipart completion requires separate handling because the server may already have committed the upload.

**Acceptance check.** Make `createUpload()` reject after the final data event. Verify the host receives the original error and the complete bytes in order, including the last chunk. Verify the microphone and camera have already stopped.

## 3. Make the Studio keyboard timeline’s empty state actionable

The Windows report contains two observations. Four older projects had empty raw keyboard-event files. A fresh recording contained a confirmed A-key press and release, yet the timeline still said “No keyboard events.” The second observation follows from the current UI. The lane checks for generated overlay segments, while the separate generation action reads raw input from the project. The older projects’ capture failure remains undiagnosed. [Issue #4467](https://github.com/CapSoftware/Cap/issues/4467) · [Lane rendering](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/desktop-gpui/src/editor_timeline.rs#L2344-L2359) · [Generation action](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/desktop-gpui/src/editor_tabs.rs#L2369-L2405)

**Proposed change.** Label the empty lane “No keyboard segments” and expose the existing “Generate Keyboard Segments” action there. A user can then discover whether a project contains usable presses without leaving the timeline to hunt for the command.

**Acceptance check.** Open a project with raw presses and zero segments, generate from the lane, and verify segments appear. Repeat with an empty event file and verify generation reports that no presses were found. Track the older recordings separately so the UI fix does not prematurely close the capture investigation.

## 4. Explain why a Wayland screenshot cannot capture the selected display

The Wayland screenshot path rejects area and display captures when the display list contains more than one entry. The desktop layer logs the reason, passes an empty result to the UI, and can return the user to the main window without an explanation. This matches the reported failure on a laptop with an external monitor. The single-display guard prevents Cap from returning pixels outside the selection. [Issue #4311](https://github.com/CapSoftware/Cap/issues/4311) · [Display guard](https://github.com/CapSoftware/Cap/blob/031ad14ec/crates/recording/src/screenshot.rs#L1205-L1229) · [Desktop error path](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/desktop-gpui/src/screenshot.rs#L105-L128)

**Proposed change.** Surface the display-isolation error in the screenshot UI and preserve the selection boundary. Treat actual multi-display support as a follow-up that needs a reliable mapping from portal image pixels to the selected output.

**Acceptance check.** On a two-display Wayland session, try both area and display capture. Each rejected capture should explain the limitation without requiring logs or saving a screenshot. Verify a supported single-display capture still succeeds. For the follow-up, include the reporter’s negative monitor origin and fractional laptop scale in the geometry tests.

## 5. Keep an isolated outage from turning the previous day red

The status reducer counts outages per UTC day. When it confirms an outage, it also marks the day on which the current unhealthy sequence started. Those two facts can belong to different episodes. This sequence reproduces the problem against the reducer. [Incident bookkeeping](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/status/lib/state.ts#L128-L153) · [Daily classification](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/status/lib/state.ts#L47-L51)

| UTC | Probe result | Significance |
| --- | --- | --- |
| 23:45 | Outage | One failed sample |
| 23:50 | Operational | Ends the first unhealthy sequence |
| 23:55 | Degraded | Starts a new unhealthy sequence |
| 00:00 | Outage | Confirms the new incident on the next day |

The reducer colors both days as outages. On the first day, the outage count came from 23:45, while the confirmation flag came from the incident that started at 23:55. The intervening success is what makes this a useful regression case.

**Proposed change.** Attribute the confirmed outage to the days with outage observations in that incident. Keep the daily sample counts intact.

**Acceptance check.** The sequence above should render the first day degraded and the second day as an outage. The existing case with consecutive outage samples on either side of midnight should continue to render both days as outages. [Existing cross-midnight tests](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/status/tests/state.test.ts#L164-L181)

## Verification and overlap

The scan exercised the embed URL builder and the midnight sequence locally. All 29 existing tests in the embed autoplay, recorder capture-release, and status-state suites passed with Bun 1.3.13. The repository pins Bun 1.4.4. Native desktop findings were traced through current code and reporter evidence; Windows and Wayland hardware verification remains part of the proposed implementation work.

The scan also found active fixes for CLI export fallback and mono headset playback, and a custom-thumbnail picker already on main. Those are useful places to review or test existing work. [CLI export PR #4454](https://github.com/CapSoftware/Cap/pull/4454) · [Mono playback PR #4459](https://github.com/CapSoftware/Cap/pull/4459) · [Thumbnail picker](https://github.com/CapSoftware/Cap/blob/031ad14ec/apps/web/components/custom-thumbnail/CustomThumbnailDialog.tsx#L260)
