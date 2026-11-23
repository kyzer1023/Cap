# Cap contribution proposals

This fork-only draft tracks candidate contributions. Each scan syncs the board branch to upstream main. Kyzer selects proposals for implementation separately. A proposal can be proposed, approved, in progress, in an upstream PR, merged, or dropped.

## Status board

Rank is the recommended order to start work. The ID is the original proposal number and stays fixed across scans. The ranking reflects user impact, recent upstream decisions, and what a reviewer can verify. [Upstream reviewed on November 23, 2026](https://github.com/CapSoftware/Cap/commit/c0f1c186f0f2edd7006b053c7245a42ac0ae6ac7).

| Rank | ID | Proposal | Status | First verification |
| --- | --- | --- | --- | --- |
| 1 | 3 | Untagged video color in the editor and export | Done — Upstream PR [#4481](https://github.com/CapSoftware/Cap/pull/4481) (`8b8faf422`): PR-body Greptile 5/5, all bot threads addressed/replied, 70/70 non-Vercel CI green, mergeable | Compare a genuinely untagged HD clip across editor, export, and share GIF. Include untagged 480p and explicitly tagged BT.601 controls. [GIF fix](https://github.com/CapSoftware/Cap/pull/4471) · [SD import rule](https://github.com/CapSoftware/Cap/blob/c0f1c186f0f2edd7006b053c7245a42ac0ae6ac7/apps/desktop-gpui/src/import.rs#L1977) |
| 2 | 9 | Full editor preview on Windows and Linux | Done — Upstream PR [#4488](https://github.com/CapSoftware/Cap/pull/4488) (`9172bcbf1c`): PR-body Greptile 5/5, all bot threads replied, 32/32 non-Vercel CI green, mergeable (AMD 780M 2.70×, UHD 620 2.68×, MX150 2.57× Half vs Full) |
| 3 | 4 | BT.709 conversion and tags in the Windows hardware encoder | Done — Upstream PR [#4484](https://github.com/CapSoftware/Cap/pull/4484) (`150a723848`): PR-body Greptile 5/5, all bot threads replied, 67/67 non-Vercel CI green, mergeable |
| 4 | 6 | Continue a Windows take after a native encoder failure | Done — Upstream PR [#4490](https://github.com/CapSoftware/Cap/pull/4490) (`be8cd99e74`): PR-body Greptile 5/5, all bot threads replied, 61/61 non-Vercel CI green, mergeable (780M 0.133 s switch gap) |
| 5 | 2 | Wayland clicks for automatic zoom | Upstream PR [#4493](https://github.com/CapSoftware/Cap/pull/4493) (`c9881a4f22`): 61/61 non-Vercel CI green after Sway 1.9 PASS; PR-body Greptile 4/5 (1 P1 + 2 P2s); cloud agent `bc-f6cd467b` fixing to reach 5/5 |
| 6 | 1 | GPU texture encoding for default Windows Studio recording | Done — Upstream PR [#4491](https://github.com/CapSoftware/Cap/pull/4491) (`84c28a4b91`): PR-body Greptile 5/5, all 3 bot threads replied, 66/66 non-Vercel CI green, mergeable (780M 1080p 2.64×, 4K 3.27×) |
| 7 | 10 | Skip redundant X11 cursor processing | Dropped by Kyzer (2026-11-24 03:13 MYT); no PR | Measure cached PNG copies and hashing. Test a cursor whose hotspot changes while its pixels stay identical. [X11 cursor cache](https://github.com/CapSoftware/Cap/blob/c0f1c186f0f2edd7006b053c7245a42ac0ae6ac7/crates/recording/src/cursor.rs#L1421) |
| 8 | 7 | Handle a failed screen-frame copy on macOS | Dropped: no Mac hardware to validate | Reproduce a freeze after a copy failure on a Mac, including when the first frame fails. The current copier passes the original through and retries failed setup. [Upstream fix and review](https://github.com/CapSoftware/Cap/pull/4403) |
| 9 | 8 | End Linux Audio Only cleanly after permanent system-audio loss | Upstream PR [#4495](https://github.com/CapSoftware/Cap/pull/4495) (`7cb7681adb`, `MERGEABLE`) opened after all 3 real PipeWire takes PASSED on kyzer-X510UNR (corr 0.0084 / -0.0005 / clean stop at 15.73 s after 3.08 s grace); `@greptileai` + CI running |
| 10 | 11 | Reuse Windows Media Foundation input samples | 780M bench 02:52 MYT at e0ca2f473: no win. Median-of-medians ProcessInput allocate 244 vs reuse 253 µs @1080p, 497 vs 518 µs @4K; all ranges overlap; 330 frames intact. Parked, no PR | Profile allocations on the texture path and verify that the encoder has released each sample before reuse. [Allocation site](https://github.com/CapSoftware/Cap/blob/c0f1c186f0f2edd7006b053c7245a42ac0ae6ac7/crates/enc-mediafoundation/src/video/h264.rs#L615) |
| 11 | 5 | Change the Linux async camera converter | Dropped as written | Linux Studio uses the FFmpeg muxers for both camera paths. Reopen if a color fixture fails through a shipping path. [Camera-only recording](https://github.com/CapSoftware/Cap/blob/c0f1c186f0f2edd7006b053c7245a42ac0ae6ac7/crates/recording/src/studio_recording.rs#L3738) · [Camera track](https://github.com/CapSoftware/Cap/blob/c0f1c186f0f2edd7006b053c7245a42ac0ae6ac7/crates/recording/src/studio_recording.rs#L3963) |
| 12 | 12 | Cross-GPU shader cache crash (found in ID9) | Cloud fix + bundle ready (`b77eeb3515`); MX150-then-Intel hardware validation currently running on kyzer-X510UNR; PR only on pass |

## How to group the work

- **One Windows encoder color PR (ID 4).** Extract the common BT.709 conversion and tagging work from the open fragmented-Instant and AMD MP4 branches. The pixel conversion, the range and matrix written to the output, and the decoded-pixel regression test belong in the same change. The existing PRs can reuse that fix. [#2940](https://github.com/CapSoftware/Cap/pull/2940) · [#3223](https://github.com/CapSoftware/Cap/pull/3223)
- **One preview benchmark comparison (ID 9).** Use the same project, scenarios, and metrics on Windows and Linux. Report the adapter, playback frame rate, and frame latency at Half and Full. This can run alongside ID 3. Each platform can change its default once its hardware results support it. [#4392](https://github.com/CapSoftware/Cap/pull/4392)
- **Three Windows PRs in sequence.** Land color correctness (ID 4), then native mid-take recovery (ID 6), then the default Studio GPU route (ID 1). A color chart, an injected encoder failure, and cross-vendor capture runs give each PR its own acceptance test. [Upstream review](https://github.com/CapSoftware/Cap/pull/2940)
- **Two Linux buffer PRs in sequence.** Land PipeWire buffer lifetime and shutdown first. Rebase the frame pool afterward and keep it for X11 and the owned-copy fallback. [#3317](https://github.com/CapSoftware/Cap/pull/3317) → [#3145](https://github.com/CapSoftware/Cap/pull/3145)

## Local verification plan

Each step below needs Kyzer's approval first. Run on Kyzer's Linux PC kyzer-X510UNR (XFCE on X11, PipeWire with the PulseAudio layer, Intel UHD 620 and NVIDIA MX150) with Codex, `cargo -j1`, one build at a time, once each cloud branch is ready.

- **ID 2.** A real Wayland take, checking clicks.json, auto-zoom segments and crop alignment. Runs in the "Sway (Cap Wayland test)" session (sway 1.9 with `--unsupported-gpu`, xdg-desktop-portal-wlr 0.7.1). XFCE on X11 stays the default.
- **ID 8.** Remove the system-audio monitor during an Audio Only take, plus a temporary disconnect and a take with the mic on.
- **ID 10.** A real X11 take with changing cursor shapes.
- **ID 9.** Half vs Full with the editor latency benchmark on Intel UHD 620 and on the MX150 through PRIME offload.
- **ID 3.** Optional untagged export sanity check.
- **ID 4.** Validated on the Windows Radeon 780M only. NVIDIA and Intel Windows encoders have no hardware and are flagged as unverified in the PR.

## Scan history

- 2026-11-24 03:48 MYT: ID9 UHD 620 retry completed (1.0 fps both, present bottleneck, Full p95 1887 vs Half 957 ms). Opened upstream PR #4488 with all 4 adapter results; @greptileai requested.
- 2026-11-24 03:43 MYT: #4484 Greptile P1/P2 fixed at 12b60fcc2, thread replies posted, @greptileai requested.
- 2026-11-24 03:39 MYT: ID6 gap fix 12d98a072, Windows retake running. ID1 ring fix 0650913cc, rerun queued. #4481 P1 fixed 8b8faf422, Greptile re-requested.
- 2026-11-24 03:36 MYT: Kyzer rule: every PR must address all bot review comments and reach Greptile 5/5. #4481 and #4484 Greptile findings sent to their cloud agents.
- 2026-11-24 03:35 MYT: Kyzer away; standing OK for Linux laptop local steps. Added ID12 (cross-GPU shader cache crash found in ID9 Intel run); cloud agent bc-a3494044 launched. 2-hourly consolidated status to Grok Bot.
- 2026-11-24 03:32 MYT: Kyzer re-scoped ID8 to detect/stop the silent system-audio reroute to the mic; cloud agent implementing.
- 2026-11-24 03:31 MYT: ID1 780M rerun: texture path refused with "texture ring full"; fix sent to cloud agent.
- 2026-11-24 03:30 MYT: ID8 laptop takes saved Complete but fallback path never triggered on PipeWire (stream rerouted to mic); PR held. ID9 MX150 no win; UHD 620 blocked by cross-GPU shader cache crash; laptop dropped offline ~03:25.
- 2026-11-24 03:25 MYT: ID6 Windows take failed gap check (3.29 s at libx264 switch, static screen); fix sent to cloud agent. ID1 780M rerun at b6c301053 started.
- **2026-11-24 03:15 MYT**: Kyzer dropped ID10; approved ID2 Sway take after ID8/ID9 laptop runs. ID1 NV12 fix at b6c301053, 780M rerun queued after ID6.
- **2026-11-24 02:55 MYT**: ID11 780M: no win (reuse ≈ allocate, overlapping ranges); parked. ID6 Windows GUI take started.
- **2026-11-24 02:46 MYT**: ID1 780M: texture path refused (rgba unsupported); fix requested. ID11 780M bench started. ID8 laptop takes started.
- **2026-11-24 02:25 MYT**: ID11 cloud done at e0ca2f473; 780M bench queued.
- **2026-11-24 02:20 MYT**: ID9 780M bench: Half/Full both ~60 fps vsync-capped, overlapping ranges; Linux laptop bench pending.
- **2026-11-24 01:35 MYT**: ID 1 cloud work finished at `88450190e`. Fragmented Studio and Instant submit screen frames as D3D11 textures through an 8-slot ring when a hardware encoder accepts them, with Map plus upload as the fallback. No timing yet: waiting on the 780M bench. ID 11 was launched in the cloud.
- **2026-11-24 01:20 MYT**: ID 4 FULL PASS on the 780M at `230bdcc7b`. The cause of the earlier failures was the test asking for 64x64 at bitrate multiplier 8.0; the test now uses 1280x720 at 0.15. AMDh264Encoder accepted all four color tags. The center pixel decoded exactly to (83, 135, 154), and ffprobe shows bt709/tv. Opened upstream [#4484](https://github.com/CapSoftware/Cap/pull/4484).
- **2026-11-24 01:15 MYT**: ID 9 cloud work finished at `c2bfe409c`. The `--preview-compare` bench on lavapipe, release, 5 repeats: Half 23.1 fps with p95 33.23 ms at 1248x702, against Full 18.4 fps with p95 47.80 ms at 1920x1080. The ranges are disjoint and the default is unchanged. Waiting on local hardware runs.
- **2026-11-24 01:05 MYT**: kyzer-X510UNR came back after a reboot. The ID 8 cold build caused a hard shutdown, and the kernel log shows no OOM. ID 8 is on hold for Kyzer to re-approve; the proposal is to have the cloud build the binary, add swap, and run only the needed bin.
- **2026-11-24 01:03 MYT**: The ID 4 rerun on the 780M at `434ba9949` failed again. The transform is AMDh264Encoder, and it rejected the first SetOutputType even without color tags. The next suspect is the test's encode config (frame size, rate, bitrate), not the BT.709 change. The cloud agent is aligning the test with a real recording config and adding diagnostics.
- **2026-11-24 01:02 MYT**: kyzer-X510UNR disconnected during the ID 8 local verify, about 105 of roughly 1300 crates into a cold `-j 1` `nice` build of the desktop-gpui tests. At 00:53 the machine had 2.3 GiB RAM used, 5.1 GiB available, and swap 1.86 of 2 GiB used. No Audio Only take had started and the audio config was untouched. Paused until the machine reconnects and is checked.
- **2026-11-24 00:58 MYT**: ID 4 fix at `434ba9949`. The first SetOutputType is now plain. Each color tag is added only if the transform accepts it, and the H.264 VUI is tried through ICodecAPI. The video processor words still keep the pixels BT.709 limited. The 780M rerun waits on Kyzer's approval.
- **2026-11-24 00:46 MYT**: The ID 4 local test on the Radeon 780M failed. A hardware H.264 MFT was found, but `H264Encoder::new` failed at SetOutputType with 0xC00D36B4 (MF_E_INVALIDMEDIATYPE), so no file was written. As written, the branch would break AMD MF encodes. The cloud agent is fixing the tagging, and a rerun needs Kyzer's approval.
- **2026-11-24 00:45 MYT**: ID 6 at `8ba82e49f`. When native Media Foundation stops mid-take, the take continues on libx264 on the same stream. The no-GPU injected-failure test fails without the timestamp seed and passes with it, and the Windows target checks and passes clippy with cargo-xwin. It now waits on a forced-failure take on the 780M. ID 1 started.
- **2026-11-24 00:40 MYT**: ID 10 at `a514b5e15`. The unchanged cursor now skips the PNG copy (7000 of 7000 samples unchanged), but the live poll on Xvfb stays near 15 microseconds because the XFixesGetCursorImage round trip dominates. Before medians were 15335 and 15227 ns, after medians 14944 to 14971 ns, and the ranges overlap. The branch also adds several hundred lines of bench code. Recommended drop; waiting on Kyzer.
- **2026-11-24 00:37 MYT**: Kyzer approved the ID 8 local verify on kyzer-X510UNR: the unit test, plus Audio Only takes driven remotely in the XFCE session. ID 2 is still waiting.
- **2026-11-24 00:35 MYT**: Kyzer approved the ID 4 local test on DESKTOP-MQ99RT1. The ID 8 and ID 2 local steps are not approved yet.
- **2026-11-24 00:27 MYT**: ID 2 at `c9881a4f2`. Clicks are read from the session pointer nodes and written to cursor.json. In the cloud, the 10 wayland_pointer tests pass, including piped clicks reaching auto zoom. Headless Sway moved the cursor, but portal-wlr would not negotiate a screencopy format, and the VM has no /dev/input. The item now waits on a real Sway take on kyzer-X510UNR. GNOME and KDE are not available.
- **2026-11-24 00:22 MYT**: ID 8 at `f7982acab`. PulseAudio null-sink tests pass in the cloud: removing the monitor for good ends the take with both audio files decoding, and reloading the sink keeps the take running. The item now waits on a real device removal on kyzer-X510UNR.
- **2026-11-24 00:10 MYT**: ID 4 at `c0da80d5e`. The Windows target typechecks and passes clippy in the cloud. Fork Actions did not start, so no hosted Windows run happened. The item now waits on the local 780M encode test, and its upstream PR is held until then.
- **2026-11-23 23:58 MYT**: ID 3 opened upstream as [#4481](https://github.com/CapSoftware/Cap/pull/4481) from `fix/untagged-video-bt709-editor-export` (2328f2b9a). It is cloud-verified only. Greptile was requested.
- **2026-11-23 23:50 MYT**: Kyzer moved all implementation and as much verification as possible to the cloud. Local machines are off limits until he approves a specific step. Items that need kyzer-X510UNR or DESKTOP-MQ99RT1 stop at "waiting on local machine", and their upstream PRs wait until that check is approved and done. The stray Windows check for ID 4 was stopped.
- **2026-11-23 23:45 MYT**: ID 4 branch pushed (`e5c6c0ddf`, Linux unit tests pass, Windows code not yet compiled). Windows 780M check started. ID 6 started from upstream main.
- **2026-11-23 23:40 MYT**: Installed sway, xdg-desktop-portal-wlr, foot and wofi on kyzer-X510UNR for ID 2. A "Sway (Cap Wayland test)" session was added. XFCE stays the default.
- **2026-11-23 23:30 MYT**: Added the local verification plan. ID 2 cannot be verified on kyzer-X510UNR yet because it has no Wayland session.
- **2026-11-23 23:25 MYT**: Kyzer dropped ID 7 because there is no Mac to validate it. Its cloud agent was cancelled and no upstream PR will be opened. ID 5 stays dropped.
- **2026-11-23 23:30 MYT**: Kyzer approved implementing every open proposal. Synced to upstream `6103c2afa`. One Cursor cloud agent per proposal, working in poteto mode with pstack. IDs 3, 9, 4, 2, 10, 7 and 8 started. IDs 6, 1 and 11 wait for the Windows sequence (4, then 6, then 1, then 11). ID 5 stays dropped as written. Upstream PRs are opened from the fork once each branch is ready.
- **2026-11-23** — Reviewed upstream `c0f1c186f` and maintainer feedback. Reordered the 11 original IDs, grouped the Windows color work and preview benchmarks, and dropped ID 5 as written.
- **2026-11-23** — upstream `4102c569`. Two Cursor cloud scans (Windows/GPU, shared/Linux), re-scoped to Richie's direction since Oct 20. 11 proposals added; muxer unlock and banded swscale closed. No benchmarks run.

## Richie's direction since Oct 20

### From the Windows/GPU scan

Richie’s merges since 2026-10-20 are making the recording look like the screen, stay up when a device or encoder dies, and open immediately. Windows is the platform he has explicitly left unfinished.

- Sharper picture: Enhance as an export and paused-preview option ([#4332](https://github.com/CapSoftware/Cap/pull/4332)), f16 features and a fused last layer ([#4421](https://github.com/CapSoftware/Cap/pull/4421)), an 8-tap downscale ([#4398](https://github.com/CapSoftware/Cap/pull/4398)), and 4:4:4 kept beside the re-encode ([#4333](https://github.com/CapSoftware/Cap/pull/4333)).
- Honest color: untagged video decodes as BT.709 ([#4471](https://github.com/CapSoftware/Cap/pull/4471)), the stream matrix is read when the container has no tag ([#4329](https://github.com/CapSoftware/Cap/pull/4329)), and imports keep their colors ([#4268](https://github.com/CapSoftware/Cap/pull/4268)).
- Exports are ready to watch: 1080p60 capped at the source ([#4336](https://github.com/CapSoftware/Cap/pull/4336)), share links default to Social ([#4440](https://github.com/CapSoftware/Cap/pull/4440)), MP4s are faststart ([#4439](https://github.com/CapSoftware/Cap/pull/4439)), and the frame rate comes from the capture rate ([#4379](https://github.com/CapSoftware/Cap/pull/4379)).
- A recording should survive the failure: VideoToolbox falls back to libx264 mid-take ([#4358](https://github.com/CapSoftware/Cap/pull/4358)), macOS copies each screen frame so ScreenCaptureKit cannot stall the writer ([#4403](https://github.com/CapSoftware/Cap/pull/4403)), Windows keeps going when the mic is unplugged ([#4323](https://github.com/CapSoftware/Cap/pull/4323)), and Studio survives a camera switch ([#4388](https://github.com/CapSoftware/Cap/pull/4388)).
- Audio Only records without encoding video ([#4394](https://github.com/CapSoftware/Cap/pull/4394)).
- Preview quality is Full on Apple Silicon and Half everywhere else until Full is measured there ([#4392](https://github.com/CapSoftware/Cap/pull/4392), `d33f5deae`). The paused preview is already drawn at device pixels ([#4382](https://github.com/CapSoftware/Cap/pull/4382)), and the latency probe exists ([#4328](https://github.com/CapSoftware/Cap/pull/4328)).
- The default Windows Studio recording never uses the GPU encoder that code already has. Crash recovery defaults on (`DEFAULT_CRASH_RECOVERY_RECORDING` in `crates/recording/src/defaults.rs`), and that forces the fragmented muxer.

### From the shared/Linux scan

Richie McIlroy landed essentially every merged PR in this window. The last ten days are the live direction; mid-November was Cap Infra, then the work moved back to the recording and the share page.

- **Share-ready picture.** Social is the default for link exports ([#4440](https://github.com/CapSoftware/Cap/pull/4440)), MP4 exports are faststart ([#4439](https://github.com/CapSoftware/Cap/pull/4439)), and untagged video is decoded as BT.709 for preview GIFs because browsers play it that way ([#4471](https://github.com/CapSoftware/Cap/pull/4471)). Imports keep their colour ([#4268](https://github.com/CapSoftware/Cap/pull/4268)) and 4:4:4 up to 40M pixels ([#4333](https://github.com/CapSoftware/Cap/pull/4333)).
- **Enhance and a sharper preview.** Optional screen enhancement ([#4332](https://github.com/CapSoftware/Cap/pull/4332)), an 8-tap downscale ([#4398](https://github.com/CapSoftware/Cap/pull/4398)), fused f16 features ([#4421](https://github.com/CapSoftware/Cap/pull/4421)), and a paused preview at device-pixel size ([#4382](https://github.com/CapSoftware/Cap/pull/4382)). Full preview is the default on Apple Silicon only ([#4392](https://github.com/CapSoftware/Cap/pull/4392)); everywhere else stays Half until it is measured.
- **The editor follows the pointer.** New recordings open with auto zooms built from clicks ([#4389](https://github.com/CapSoftware/Cap/pull/4389), [#4393](https://github.com/CapSoftware/Cap/pull/4393)), hover anchors ([#4366](https://github.com/CapSoftware/Cap/pull/4366)), and shape changes under a still pointer ([#4267](https://github.com/CapSoftware/Cap/pull/4267), [#4452](https://github.com/CapSoftware/Cap/pull/4452)).
- **New capture modes.** Audio Only with an MP3/WAV export ([#4394](https://github.com/CapSoftware/Cap/pull/4394)), mic and camera switches from the controls bar ([#4262](https://github.com/CapSoftware/Cap/pull/4262)), Studio surviving a camera switch ([#4388](https://github.com/CapSoftware/Cap/pull/4388)), and the camera staying switchable after a Linux resume (`6dce22b4d`).
- **Account security.** Passkeys ([#4435](https://github.com/CapSoftware/Cap/pull/4435), [#4438](https://github.com/CapSoftware/Cap/pull/4438)), email change ([#4436](https://github.com/CapSoftware/Cap/pull/4436)), and Google One Tap ([#4434](https://github.com/CapSoftware/Cap/pull/4434)).
- **The public share page.** Brand colour, custom thumbnails, Spaces, comment-upload size checks ([#4463](https://github.com/CapSoftware/Cap/pull/4463)). The desktop version is stored on the video ([#4331](https://github.com/CapSoftware/Cap/pull/4331)) and stripped from share and embed pages.
- **Earlier in the window.** Cap Infra (URL import, delivery metering, MCP) around 13 Nov, on-screen drawing, separate-track export, and dashboard bulk actions. That Infra push has cooled; the open work is the recording and how it looks when shared.

## Original scan notes by proposal ID

These are the findings from the initial scan of `4102c569`. The implementation order and verification criteria above supersede the initial ranking and proposed designs.

### 1. Put the default Windows recording on the GPU texture path

**Fits.** The native Media Foundation encoder already takes the capture texture. The path users actually hit reads every frame back to the CPU, then FFmpeg uploads it again. That is the opposite of Audio Only skipping video encode, and it spends the CPU Enhance and Social exports need.

**Code.** `RecordingDefaults::default` sets crash recovery on, and `apply_to_studio_builder` passes that in as `with_fragmented` (`crates/recording/src/defaults.rs`). `make_studio_mode_pipeline` then builds `WindowsFragmentedM4SMuxer`; `WindowsMuxer` runs only when fragmented is false (`crates/recording/src/capture_pipeline.rs`). Instant always builds the fragmented muxer. Each frame calls `frame.as_ffmpeg()` (`crates/recording/src/output_pipeline/win_fragmented_m4s.rs`, the `Paced::Frame` arm). That Maps a staging texture with a blocking `D3D11_MAP_READ` (`crates/scap-direct3d/src/lib.rs`, `Frame::as_buffer`) and copies BGRA into a pooled `AVFrame` (`crates/scap-ffmpeg/src/direct3d.rs`). The studio MF loop already keeps `last_texture` on the GPU (`crates/recording/src/output_pipeline/win.rs`).

**Change.** Feed the fragmented screen encoder from `ScreenFrame::texture()` through the same D3D11 video-processor input `H264Encoder::run` uses, and keep a small texture ring for the pacer’s repeat frame. Leave the `as_ffmpeg` path for the libx264 fallback. Do the screen track first. The fragmented camera muxer has the same `as_ffmpeg` call and can follow.

**Value.** Closest published numbers are on the non-fragmented AMF path in [#3223](https://github.com/CapSoftware/Cap/pull/3223): at 3840×2160, MF texture was 7.7 ms/frame wall and 1.8 ms CPU against 12.5 ms and 13.3 ms for the CPU readback. 1080p was slightly slower on the wall and about 2.7× cheaper on CPU. The fragmented path adds the Map plus a BGRA copy, so it should be at least that CPU. Remeasure rather than reuse those numbers: the conversion timer is already around `as_ffmpeg` in `win_fragmented_m4s.rs` (`convert_start` / “D3D11 frame conversion exceeded”). Add a release example next to that muxer, 300 frames at 1920×1080 and 3840×2160, median wall time and process CPU, readback path against texture submit. Decode all frames afterward.

**Platform.** Windows, any GPU. The 780M is the machine that matters, because Studio defaults to this path and AMD is forced off the MF muxer today.

**Risk.** Fragment timestamps, pause repeats, and a vendor MFT that will not accept the capture texture. Keep the CPU path as the fallback that already ships.

**Overlap.** [#3223](https://github.com/CapSoftware/Cap/pull/3223) enables MF textures for AMD on the non-fragmented MP4 and says the fragmented path is untouched. [#3300](https://github.com/CapSoftware/Cap/pull/3300) scales on the GPU and still readbacks for the software encoder. [#3134](https://github.com/CapSoftware/Cap/pull/3134) is the camera upload into that software encoder. No overlap with [#3145](https://github.com/CapSoftware/Cap/pull/3145), [#3317](https://github.com/CapSoftware/Cap/pull/3317), or [#3810](https://github.com/CapSoftware/Cap/pull/3810).

**Local validation.** Yes, on DESKTOP-MQ99RT1 (AMD 780M), for both the AMF fallback and an MF texture attempt.

_Source: Windows/GPU scan._

### 2. Record Wayland pointer events so auto zoom and hover have something to follow

**Builds on:** [#4389](https://github.com/CapSoftware/Cap/pull/4389), [#4393](https://github.com/CapSoftware/Cap/pull/4393), [#4366](https://github.com/CapSoftware/Cap/pull/4366), [#4383](https://github.com/CapSoftware/Cap/pull/4383), [#4267](https://github.com/CapSoftware/Cap/pull/4267).

**Why it fits.** Those PRs generate the first-open zooms from clicks and anchor the cursor to hover changes. On a normal Wayland session (`WAYLAND_DISPLAY` set and `XDG_SESSION_TYPE=wayland`, which is also true when XWayland sets `DISPLAY`) the cursor thread returns immediately with empty moves, clicks, and keypresses.

```236:245:crates/recording/src/cursor.rs
fn prefers_wayland_portal_cursor() -> bool {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return false;
    }
    std::env::var_os("DISPLAY").is_none()
        || std::env::var("XDG_SESSION_TYPE")
            .is_ok_and(|session| session.eq_ignore_ascii_case("wayland"))
}
```

```677:692:crates/recording/src/cursor.rs
    if prefers_wayland_portal_cursor() {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(CursorActorResponse {
            cursors: prev_cursors,
            next_cursor_id,
            moves: vec![],
            clicks: vec![],
            keyboard_presses: vec![],
        });
        return CursorActor { /* no thread */ };
    }
```

Studio then turns custom-cursor capture off and asks the portal to burn the cursor into the frames (`studio_recording.rs` 3714–3715 and 3838–3841, `linux.rs` 460–464). The video shows a cursor. The editor’s zoom pass, which only reads clicks (`auto_zoom.rs` 34–62, `editor_window.rs` 6833–6868), gets an empty list and writes no zooms. Hover anchors and the keyboard log are empty for the same reason.

**Change.** Keep the portal cursor embedded, so nothing is drawn twice. Separately, sample pointer position and buttons in the screencast’s own coordinate space (PipeWire `SPA_META_Cursor` on the stream already opened in `linux.rs`) and write the existing cursor move/click files. Leave `custom_cursor_capture` false until a follow-up has a real cursor image and can switch the portal to `CursorMode::Hidden`. Keyboard capture on pure Wayland needs its own source; do not pretend the X11 `device_query` loop works there.

**Value.** Wayland recordings start participating in the auto-zoom and hover work. X11 already records clicks, so this is the platform that currently drops the feature on the floor.

**Verify.** A unit test that a Wayland-shaped environment currently builds an empty `CursorActorResponse`, and that a synthetic cursor-meta sequence produces clicks for which `generate_auto_zoom_segments` is non-empty. On a Wayland desktop: record a short Studio take with a few clicks, open it, and confirm the zoom lane is filled and `cursors/clicks.json` is non-empty.

**Platform.** Linux.

**Risk.** Cursor metadata is in stream space and must survive the crop in `prepare_pipewire_frame`. A wrong origin sends every zoom to the corner. Shipping the overlay cursor in the same PR would double-draw on top of the embedded one.

_Source: shared/Linux scan._

### 3. Decode untagged video as BT.709 in the editor and export

**Builds on:** [#4471](https://github.com/CapSoftware/Cap/pull/4471), [#4268](https://github.com/CapSoftware/Cap/pull/4268), [#4333](https://github.com/CapSoftware/Cap/pull/4333).

**Why it fits.** Today’s GIF fix states the rule: FFmpeg defaults untagged video to BT.601, and browsers play it as BT.709. The media server now forces that for the preview GIF.

```2692:2698:apps/media-server/src/lib/media-video.ts
			// Untagged video defaults to BT.601 in FFmpeg but plays as BT.709 in
			// browsers. These only apply when the input carries no colour tags.
			"-colorspace",
			"bt709",
			"-color_range",
			"tv",
```

The editor still does the opposite. `YuvMatrix::from_color_space` maps everything except an explicit BT.709 tag to BT.601 (`crates/rendering/src/cpu_yuv.rs` 4–20), and the header probe test locks untagged files to BT.601 (`crates/rendering/src/project_recordings.rs` 909–924). An imported untagged file therefore looks different in the editor than in the share-page GIF and the browser.

**Change.** Treat `Space::Unspecified` like BT.709 in `YuvMatrix::from_color_space` and in the swscale path that already has `use_bt709_source` (`crates/rendering/src/decoder/frame_converter.rs`). Keep an explicit SMPTE 170M / BT.601 tag on BT.601. Update the `untagged.mp4` expectation and the comment in `cpu_yuv.rs` that records the old choice.

**Value.** One colour rule for imports, the editor, export, and the share GIF. Cap’s own encodes are already tagged BT.709 (`crates/enc-ffmpeg/src/video/h264.rs`), so they stay put.

**Verify.** `cargo test -p cap-rendering --lib header_video_metadata_reads_stream_matrix_when_container_is_untagged` and the `cpu_yuv` matrix tests. Add an untagged fixture next to the existing colour check in `crates/verify/tests/export.rs` and compare a red patch against the BT.709 reference already used there.

**Platform.** Shared (editor, export) plus the web GIF path already shipped.

**Risk.** Any untagged file that really was BT.601 and relied on the old default will shift. The PR should say that, and the explicit 601 case in the same test must stay 601.

_Source: shared/Linux scan._

### 4. Tag the Media Foundation encode as BT.709 limited

**Fits.** [#4471](https://github.com/CapSoftware/Cap/pull/4471), [#4329](https://github.com/CapSoftware/Cap/pull/4329), and [#4268](https://github.com/CapSoftware/Cap/pull/4268) exist because a missing or wrong matrix shows up in preview, GIFs, and import. The Windows hardware encoder neither tags the bitstream nor packs the processor color space into the fields the API defines.

**Code.** `H264Encoder` sets subtype, bitrate, size, and rate on the output type and only subtype, size, and rate on the NV12 input (`crates/enc-mediafoundation/src/video/h264.rs`, the `SetGUID` / `SetUINT32` block). HEVC is the same (`crates/enc-mediafoundation/src/video/hevc.rs`). The processor then does this (`crates/enc-mediafoundation/src/video/video_processor.rs`):

```98:105:crates/enc-mediafoundation/src/video/video_processor.rs
        let mut color_space = D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
            _bitfield: 1 | D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_0_255.0 as u32,
        };
        unsafe { video_context.VideoProcessorSetOutputColorSpace(&video_processor, &color_space) };
        color_space._bitfield = 1 | D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_16_235.0 as u32;
        unsafe {
            video_context.VideoProcessorSetStreamColorSpace(&video_processor, 0, &color_space)
        };
```

In windows 0.60 those constants are `0_255 = 2` and `16_235 = 1`. The SDK layout of that `_bitfield` is Usage (bit 0), RGB_Range (bit 1), YCbCr_Matrix (bit 2), then Nominal_Range in bits 4–5. So the output word is `1 | 2 = 3`: Usage set, RGB_Range = studio, matrix left at BT.601, Nominal_Range left at 0. The stream word is `1 | 1 = 1`: Usage set, full RGB range, still BT.601. Capture frames are full-range BGRA. The `1` does not select BT.709. That bit is value 4.

**Change.** Build the color-space word with the matrix in bit 2 and the nominal range shifted to bits 4–5. Set input and output media types with BT.709 primaries, transfer, matrix, and limited nominal range so the file matches the FFmpeg path Richie already treats as the reference.

**Value.** Color match, not frame rate. [#3223](https://github.com/CapSoftware/Cap/pull/3223) measured the untagged AMD case: source RGB (125, 62, 93) decoded as (127, 68, 91) before the tag and (122, 60, 90) after, the same as the FFmpeg fallback. Repeat that chart on the live Intel/NVIDIA path. Method: one synthetic frame through `H264Encoder`, ffprobe `color_space` and `color_range` must be `bt709` and `tv`, and the decoded pixel must match `bt709_luma` / `bt709_chroma` in `crates/rendering/src/cpu_yuv.rs` within the same tolerance the existing `export_shader_encodes_bt709_limited_range` test uses.

**Platform.** Windows, any GPU that uses this encoder (Intel and NVIDIA on current main).

**Risk.** An MFT that rejects the color attributes and falls back to software. Set them, and on rejection keep today’s types and log it.

**Overlap.** [#3223](https://github.com/CapSoftware/Cap/pull/3223) does this tagging as part of turning AMD onto this encoder. This proposal is the packing fix on the path that already ships. Reconcile the constants with that branch. No overlap with the other five.

**Local validation.** Yes, on DESKTOP-MQ99RT1, once AMD is allowed onto this encoder. Intel/NVIDIA can be checked on any Windows GPU.

_Source: Windows/GPU scan._

### 5. Route the Linux camera converter through `convert_or_scale` and set BT.709

**Builds on:** [#4471](https://github.com/CapSoftware/Cap/pull/4471), [#4329](https://github.com/CapSoftware/Cap/pull/4329), merged AVX2 RGBA→NV12 ([#3287](https://github.com/CapSoftware/Cap/pull/3287)).

Linux camera encode never takes the BT.709 path the encoders and the new GIF use. `AsyncCamera` builds the H.264 encoder with `with_external_conversion()` and hands every frame to `cap-frame-converter`, whose swscale context is created with default coefficients and no call to `use_bt709_for_rgb_input` (`crates/recording/src/output_pipeline/async_camera.rs` 69–94, `crates/frame-converter/src/swscale.rs` 41–57). That is the Linux camera track: D3D11 and VideoToolbox are absent, so this pool is the conversion. It also skips the AVX2 RGBA→NV12 path inside `convert_or_scale`.

Route the pool through `cap_enc_ffmpeg::convert_or_scale`, and set BT.709 on the swscale fallback the same way `H264Encoder` does. Check it with a limited-range red patch through `AsyncConverterPool` into the colour assertion already in `crates/enc-ffmpeg/src/video/h264.rs`, plus `cargo test -p cap-frame-converter`. Risk: the pool’s output must stay limited-range BT.709 or the tagged stream and the pictures diverge. This extends proposal 2 and [#4471](https://github.com/CapSoftware/Cap/pull/4471). No overlap with the six open PRs.

**Platform.** Linux (camera track).

**Overlap.** None of the six open kyzer1023 PRs.

_Source: shared/Linux scan follow-up._

### 6. Keep recording when Media Foundation dies mid-take

**Fits.** [#4358](https://github.com/CapSoftware/Cap/pull/4358) replaces a dead VideoToolbox encoder with libx264 and keeps the take. The Windows fragmented FFmpeg path already emits `EncoderRebuilt`. The native MF loop does not.

**Code.** On `EncoderUnhealthy`, `WindowsMuxer` calls `force_software_only()` and returns the error (`crates/recording/src/output_pipeline/win.rs`, the `match result` after `encoder.run`). `should_fallback` is true for every unhealthy status (`crates/enc-mediafoundation/src/video/h264.rs`). The flag only changes the next setup. This setup ends the pipeline, and the studio test expects that failure to mark the project Failed (`stop_after_a_screen_pipeline_failure_confirms_capture_stopped` in `crates/recording/src/studio_recording.rs`). The fragmented muxer already logs the replacement and continues (`crates/recording/src/output_pipeline/win_fragmented_m4s.rs`, `report_encode_result`).

**Change.** On a fallback-worthy MF failure, finish the current fragment, open the next segment with the FFmpeg encoder the fallback closure already builds, and emit `PipelineHealthEvent::EncoderRebuilt`. Leave a hard Windows error as a failure. Studio already has a segment factory, so this is a new segment rather than a codec change inside one MP4.

**Value.** A dead MFT stops losing the rest of the take. The macOS change is the acceptance test: inject an unhealthy status after N frames, assert a later segment is libx264 (or AMF on AMD), timestamps continue, and the project is not Failed. The existing `hardware_crash` session in `crates/enc-ffmpeg/src/video/hardware_crash.rs` is the crash-disable hook to keep.

**Platform.** Windows, any GPU. On current main this is the non-fragmented Intel/NVIDIA path. It becomes the AMD path if [#3223](https://github.com/CapSoftware/Cap/pull/3223) or proposal 1 lands.

**Risk.** A gap or overlap at the segment boundary, and audio that was aligned to the old timeline. The fragmented audio muxer already cuts on its own clock, so the test has to check both.

**Overlap.** None of the six open PRs rebuild the MF encoder. [#3223](https://github.com/CapSoftware/Cap/pull/3223) changes how AMD stops the MFT (drain instead of flush) and should land first so this fallback does not throw away that tail.

**Local validation.** Yes, on DESKTOP-MQ99RT1, if the replacement encoder is AMF. A forced `EncoderUnhealthy` against libx264 can be tested on any Windows machine.

_Source: Windows/GPU scan._

### 7. Never give ScreenCaptureKit its own surface back

**Fits.** [#4403](https://github.com/CapSoftware/Cap/pull/4403) copies each screen frame into a pool this process owns, because AVAssetWriter holds frames and ScreenCaptureKit delivers nothing once its few surfaces are all retained. The failure path of that copy puts the capture surface back into the writer.

**Code.** `FrameCopier::copy` uses the scaler’s sample, and otherwise returns the original (`crates/recording/src/output_pipeline/macos.rs`). `FrameScaler::scale_frame` returns `None` when `pool.pixel_buf()` or the transfer fails (`crates/recording/src/sources/screen_capture/macos.rs`). The pool’s minimum is 4 (`FRAME_COPY_POOL_MIN_BUFFERS`), and the pixel format is `420v`. The writer lag that motivated the fix was described as up to a second.

**Change.** If the copy fails, repeat the last owned copy or drop the frame. Do not append the ScreenCaptureKit buffer. Count those failures next to the existing “Could not set up the screen frame copy” warning.

**Value.** Removes the remaining way the freeze returns. It is the tail of a bug that already shipped, so the win is the stalls that come back under pool pressure, not a steady-state speedup. Verify with the studio camera recording from [#4403](https://github.com/CapSoftware/Cap/pull/4403): hold the writer (or force `pixel_buf()` to fail), confirm the capture surface’s retain count drops, and confirm new frames still arrive. The zero-copy VideoToolbox log line in `crates/verify/tests/crash.rs` (“Selected hardware H264 encoder with zero-copy VideoToolbox input”) should still appear, because the copier target is already `420v`.

**Platform.** macOS.

**Risk.** A repeated frame during a real pool failure. That is the same trade the Windows pacer already makes with `last_texture`.

**Overlap.** None.

**Local validation.** No AMD machine. Needs a Mac.

_Source: Windows/GPU scan._

### 8. Stop a Linux Audio Only recording when system audio dies

**Builds on:** [#4394](https://github.com/CapSoftware/Cap/pull/4394), specifically `c8627a369` (“end a macOS audio-only recording when its system audio stream ends”).

**Why it fits.** Audio Only’s picture is a stand-in; the recording is the audio. On macOS a system stop of that stream ends the take (`macos.rs` 1479–1484). On Linux the same mode calls `create_system_audio_source_config` (`studio_recording.rs` 4139–4152), which builds the error channel and drops the receiver:

```1898:1900:crates/recording/src/sources/screen_capture/linux.rs
pub async fn create_system_audio_source_config() -> anyhow::Result<SystemAudioSourceConfig> {
    let (error_tx, _error_rx) = flume::bounded(16);
    let feed = MicrophoneFeed::spawn(MicrophoneFeed::new_system_audio(error_tx));
```

A PipeWire or Pulse monitor that disappears mid-take leaves the stand-in clock running and the file growing in silence until the user presses Stop.

**Change.** Keep that receiver on the audio-only path and, on a source-gone or stream-stopped error, mark the recording user-stopped and cancel the pipeline the way the macOS `standalone` branch does. Leave ordinary screen recordings alone: there the picture continues and a dropped monitor is a missing track, which is a different decision.

**Value.** Audio Only on Linux fails closed instead of producing a long silent project.

**Verify.** A studio-recording test that injects the system-audio error and asserts the actor reaches the stopped state with a non-empty error on the segment. On a machine with PipeWire: start Audio Only with system audio, unload the monitor source, and confirm the recording ends within a couple of seconds.

**Platform.** Linux.

**Risk.** The feed already uses that channel for connect failures during setup. The stop handler has to ignore errors that happen before the pipeline is running, or a flaky first connect will abort a take that the existing retry loop (`retry_system_audio_connection`) would have saved.

_Source: shared/Linux scan._

### 9. Measure Full editor preview on Windows and Linux before changing the default

Two scans proposed the same measurement for their platform; both versions follow.

#### Windows

Measure Full preview on Windows, then default to it only where it is cheap

**Fits.** This is the sentence left open in [#4392](https://github.com/CapSoftware/Cap/pull/4392): Full on Apple Silicon, Half elsewhere until Full is measured. The paused preview is already rendered at the canvas device-pixel size ([#4382](https://github.com/CapSoftware/Cap/pull/4382)). Windows users still preview at scale 0.65.

**Code.** `EditorPreviewQuality::for_this_machine` returns Full only for `macos` + `aarch64` when the adapter is hardware (`apps/desktop-gpui/src/store.rs`). A software adapter stays on Half. The probe is `apps/desktop-gpui/src/editor_latency_bench.rs`, wired from `app_windows.rs` ([#4328](https://github.com/CapSoftware/Cap/pull/4328)).

**Change.** Run that bench at Half and Full on the real editor device, including a paused redraw at canvas size. Record p95 frame time. If Full stays inside the Apple Silicon budget on that adapter class, return Full from `for_this_machine` for that class. Keep the software-adapter Half path. Do not flip the default from a guess.

**Value.** A sharper preview on the machines where the GPU already draws it. The number does not exist yet. The bench is the measurement: same project, Half then Full, paused and playing, p95 of the latency probe. The 780M is the interesting case because the iGPU is also encoding.

**Platform.** Windows, any GPU. The policy change is per adapter class.

**Risk.** Full preview stealing the iGPU from a 60 fps recording. Gate on the bench, and leave a stored `half` with the chosen-key as the user override that already exists.

**Overlap.** None of the six PRs. This is the editor GPU policy, not the desktop UI around it.

**Local validation.** Yes, on DESKTOP-MQ99RT1, before any default change. That machine is the one where Full may not be free.

#### Linux

Measure Full editor preview on Linux and default it when the GPU can take it

**Builds on:** [#4392](https://github.com/CapSoftware/Cap/pull/4392), [#4382](https://github.com/CapSoftware/Cap/pull/4382), and `d33f5deae` (“keep Windows on Half until Full is measured there”).

**Why it fits.** `EditorPreviewQuality::for_this_machine` is Full only on Apple Silicon with a hardware adapter. The comment names the follow-up: Half everywhere else until Full is measured, and a software adapter stays on Half because it readbacks every frame (`apps/desktop-gpui/src/store.rs` 931–942). Linux Vulkan is in that “not measured” bucket. Windows is the sibling’s.

**Change.** Run the preview latency bench at Half (`scale` 0.65) and Full (`scale` 1.0) on a discrete or integrated Vulkan GPU, with the software-adapter case left on Half. If pause and playback stay in the same band Apple Silicon was accepted for, return `Full` from `for_this_machine` when `target_os = "linux"` and the adapter is hardware. Keep the stored-Half-means-old-default logic already in `from_general`.

**Value.** The paused-preview and Full-default work actually reaches Linux desktops. Full is about 2.4× the pixels of Half (`1 / 0.65²`); whether that is free on Vulkan is unmeasured. No number from this scan.

**Verify.** The bench already in `apps/desktop-gpui/src/editor_latency_bench.rs`:

```
CAP_GPUI_AUTO_EDITOR=<project.cap> CAP_GPUI_AUTO_LATENCY_BENCH=1 \
  CAP_GPUI_AUTO_LATENCY_SCENARIOS=playback,pause \
  CAP_GPUI_NO_MENUS=1 CAP_GPUI_NO_TRAY=1 CAP_GPUI_MUTE_AUDIO=1 \
  cargo run -p cap-desktop-gpui --release
```

Run it twice, with `editorPreviewQuality` set to `half` and `full`, on a hardware Vulkan adapter and again with the software adapter forced. Compare Demand→Painted. This VM has no desktop GPU, so that run is still to do.

**Platform.** Linux desktop.

**Risk.** Defaulting Full before the bench, or on llvmpipe, fights the comment in `display_enhancement` that a software adapter already costs too much (`crates/rendering/src/lib.rs` 3957–3960). The settings migration must keep treating an untouched stored `half` as “not a choice,” which `from_general` already does.

### 10. Give the X11 cursor the same cheap shape check macOS just got

**Builds on:** [#4452](https://github.com/CapSoftware/Cap/pull/4452), [#4267](https://github.com/CapSoftware/Cap/pull/4267).

**Why it fits.** [#4452](https://github.com/CapSoftware/Cap/pull/4452) slows still-pointer shape checks after two seconds, and that interval is shared. The fast path that skips the image encode is macOS-only. On every other platform `sample_cursor` ignores the fingerprint and always reads the image (`cursor.rs` 1166–1171). On X11 that read is an `xfixes_get_cursor_image` round trip, and a matching bitmap still clones the cached PNG (`cursor.rs` 1421–1427) so the caller can hash it. `shape` is hard-coded `None` (`cursor.rs` 1391–1398), so the editor’s macOS bitmap matcher (`recorded_image_shape` in `crates/rendering/src/layers/cursor.rs`) never classifies a Linux cursor.

**Change.** Fingerprint the XFixes bitmap (width, height, hotspot, and a hash of the pixels) before encoding PNG, and return `CursorSample::Unchanged` when it matches, the way `macos_cursor_fingerprint` does. Keep PNG encoding for a bitmap that is actually new.

**Value.** Small next to proposal 1. It completes the Linux half of a perf change that just merged, and it stops a PNG clone plus a SHA-256 on every moving-pointer tick. I did not time it. The macOS comment in that file puts the window-server query around 0.2 ms; the X11 round trip is the same kind of cost, paid on every move tick today.

**Verify.** Extend `a_resting_pointer_slows_its_shape_checks_after_settling` with an X11-cursor test that two identical `GetCursorImage` replies produce one PNG encode, using the existing `x11_cursor_tests` module. A moving-pointer microbench can wrap `X11CursorSource::cursor_data` over a cached reply and print encode counts.

**Platform.** Linux (X11). Wayland does not reach this function until proposal 1 lands.

**Risk.** A fingerprint that ignores the hotspot would miss a theme that only moves the hot spot. Include it in the key.

_Source: shared/Linux scan._

### 11. Reuse the Media Foundation input sample

**Fits.** Small, and it sits on the encoder path proposals 1–3 make the default. Per-frame COM allocation is on the list of things that show up once the readback is gone.

**Code.** Every `METransformNeedInput` calls `MFCreateDXGISurfaceBuffer` and `MFCreateSample` against the same processor output texture (`crates/enc-mediafoundation/src/video/h264.rs`, inside `run`). [#3279](https://github.com/CapSoftware/Cap/pull/3279) recycled the playback readback buffers. The encode input sample was not part of that.

**Change.** Cache one sample and one DXGI surface buffer for the current output texture. Call `SetSampleTime` each frame. Recreate them when the texture identity changes.

**Value.** Tens to a few hundred microseconds per frame, not the millisecond readback. Worth doing after proposal 1, not instead of it. Time it with the same 300-frame release harness as proposal 1, texture path only, before and after, median and p95 of the `ProcessInput` section.

**Platform.** Windows, any GPU that uses `H264Encoder::run`.

**Risk.** A transform that retains the sample. If the next `ProcessInput` returns `MF_E_NOTACCEPTING` or the sample is still busy, allocate a second one and keep two, matching the playback pool’s depth.

**Overlap.** [#3134](https://github.com/CapSoftware/Cap/pull/3134) reuses the camera upload texture and the media buffer before this encoder. The sample created here is downstream of that. [#3223](https://github.com/CapSoftware/Cap/pull/3223) does not cache the sample. No overlap with the Linux or CLI PRs.

**Local validation.** Yes, on DESKTOP-MQ99RT1, because the AMD MFT is the one that has already shown different input lifetime (it holds queued samples across stop, which is why [#3223](https://github.com/CapSoftware/Cap/pull/3223) drains).

_Source: Windows/GPU scan._

## Closed or not proposed

- Muxer unlock is not a win. Scan mux record playback (cloud sub-scan) read the lock scopes: `write_interleaved` runs on the writer thread, and the encoder only holds the mutex long enough to push the queue.
- Banded swscale is still unused. Scan conversion and encode (cloud sub-scan) found no `sws_setThreads` anywhere. Leave it until a 4K profile says the single thread is the limiter.

Scan Linux capture hot paths (cloud sub-scan) confirmed the PipeWire row copy and the X11 GetImage alloc are still on this SHA. Those are [#3317](https://github.com/CapSoftware/Cap/pull/3317) and [#3145](https://github.com/CapSoftware/Cap/pull/3145), so they are not new proposals. The Linux preview allocating a fresh BGRA buffer every frame (`apps/desktop-gpui/src/feeds.rs` around 1580) is the same idea as [#3134](https://github.com/CapSoftware/Cap/pull/3134) on another platform. It is real and smaller than the five proposals already ranked.

## Overlap with open kyzer1023 PRs

[#3134](https://github.com/CapSoftware/Cap/pull/3134), [#3145](https://github.com/CapSoftware/Cap/pull/3145), [#3223](https://github.com/CapSoftware/Cap/pull/3223), [#3300](https://github.com/CapSoftware/Cap/pull/3300), [#3317](https://github.com/CapSoftware/Cap/pull/3317), [#3810](https://github.com/CapSoftware/Cap/pull/3810). Overlap is noted per proposal and is not a blocker.
