use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use cap_project::XY;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{
    CursorActorResponse, CursorData, CursorSample, X11CursorSource, cursor_id_for_image,
    sample_cursor,
};

const RUNS: usize = 7;
const SIZES: [(u16, u16); 3] = [(24, 24), (48, 48), (64, 64)];

pub fn x11_cursor_cache_bench_report(
    iterations: u32,
    display: Option<&str>,
    hold_so: Option<&str>,
) -> String {
    let iterations = iterations.max(1);
    let mut sizes = Vec::with_capacity(SIZES.len());
    for (width, height) in SIZES {
        sizes.push(measure_size(width, height, iterations));
    }
    let x11 = match display {
        Some(display) => measure_x11(display, 200),
        None => json!({ "ran": false }),
    };
    let live = match (display, hold_so) {
        (Some(display), Some(hold_so)) => measure_live_cursor(display, hold_so, iterations),
        _ => json!({ "ran": false }),
    };
    let report = json!({
        "debugAssertions": cfg!(debug_assertions),
        "iterations": iterations,
        "runs": RUNS,
        "timed": "Live section: sample_cursor plus cursor_id_for_image on an xsetroot cursor under Xvfb, after xdotool moves the pointer. Synthetic sizes: png cache hit plus the id byte match, without GetCursorImage.",
        "sizes": sizes,
        "x11": x11,
        "live": live,
    });
    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

fn pattern(width: u16, height: u16) -> Vec<u32> {
    let count = usize::from(width) * usize::from(height);
    let mut pixels = Vec::with_capacity(count);
    let mut state = 0x1234_5678u32;
    for _ in 0..count {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        pixels.push(0xff00_0000 | (state & 0x00ff_ffff));
    }
    pixels
}

fn empty_response() -> CursorActorResponse {
    CursorActorResponse {
        cursors: Default::default(),
        next_cursor_id: 0,
        moves: Vec::new(),
        clicks: Vec::new(),
        keyboard_presses: Vec::new(),
    }
}

fn per_call_ns(started: Instant, iterations: u32) -> u64 {
    let nanos = started.elapsed().as_nanos();
    u64::try_from(nanos / u128::from(iterations)).unwrap_or(u64::MAX)
}

fn summary(mut samples: Vec<u64>) -> Value {
    samples.sort_unstable();
    json!({
        "medianNs": samples[samples.len() / 2],
        "minNs": samples[0],
        "maxNs": samples[samples.len() - 1],
        "runsNs": samples,
    })
}

struct Prepared {
    source: X11CursorSource,
    width: u16,
    height: u16,
    pixels: Vec<u32>,
    png: Vec<u8>,
    hotspot: XY<f64>,
    dir: PathBuf,
    response: CursorActorResponse,
    last: Option<(Vec<u8>, [u64; 2], String)>,
}

fn prepare(width: u16, height: u16) -> Result<Prepared, String> {
    let pixels = pattern(width, height);
    let hotspot = XY::new(1.0 / f64::from(width), 2.0 / f64::from(height));
    let mut source = X11CursorSource::default();
    let png = source
        .png(width, height, pixels.clone())
        .ok_or_else(|| "png encode failed".to_string())?;
    let dir = std::env::temp_dir().join(format!(
        "cap-x11-cursor-bench-{}-{width}-{height}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let mut response = empty_response();
    let mut last = None;
    cursor_id_for_image(
        CursorData {
            image: png.clone(),
            hotspot,
            shape: None,
        },
        &mut response,
        &dir,
        None,
        &mut last,
    );
    if response.cursors.len() != 1 || last.is_none() {
        return Err("warm-up did not register one cursor".to_string());
    }
    Ok(Prepared {
        source,
        width,
        height,
        pixels,
        png,
        hotspot,
        dir,
        response,
        last,
    })
}

impl Drop for Prepared {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn pixel_bank(pixels: &[u32], iterations: u32) -> Vec<Vec<u32>> {
    let mut bank = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        bank.push(pixels.to_vec());
    }
    bank
}

fn time_cached_poll(prepared: &mut Prepared, iterations: u32) -> Result<u64, String> {
    let bank = pixel_bank(&prepared.pixels, iterations);
    let width = prepared.width;
    let height = prepared.height;
    let hotspot = prepared.hotspot;
    let started = Instant::now();
    for pixels in bank {
        let image = prepared
            .source
            .png(width, height, black_box(pixels))
            .ok_or_else(|| "png cache miss returned None".to_string())?;
        let id = cursor_id_for_image(
            CursorData {
                image: black_box(image),
                hotspot,
                shape: None,
            },
            &mut prepared.response,
            &prepared.dir,
            None,
            &mut prepared.last,
        );
        black_box(id);
    }
    Ok(per_call_ns(started, iterations))
}

fn time_png_hit(prepared: &mut Prepared, iterations: u32) -> Result<u64, String> {
    let bank = pixel_bank(&prepared.pixels, iterations);
    let width = prepared.width;
    let height = prepared.height;
    let started = Instant::now();
    for pixels in bank {
        let image = prepared
            .source
            .png(width, height, black_box(pixels))
            .ok_or_else(|| "png cache miss returned None".to_string())?;
        black_box(image);
    }
    Ok(per_call_ns(started, iterations))
}

fn time_id_match(prepared: &mut Prepared, iterations: u32) -> u64 {
    let mut images = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        images.push(prepared.png.clone());
    }
    let hotspot = prepared.hotspot;
    let started = Instant::now();
    for image in images {
        let id = cursor_id_for_image(
            CursorData {
                image: black_box(image),
                hotspot,
                shape: None,
            },
            &mut prepared.response,
            &prepared.dir,
            None,
            &mut prepared.last,
        );
        black_box(id);
    }
    per_call_ns(started, iterations)
}

fn time_pixel_compare(pixels: &[u32], iterations: u32) -> u64 {
    let copy = pixels.to_vec();
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(black_box(pixels) == black_box(copy.as_slice()));
    }
    per_call_ns(started, iterations)
}

fn time_sha256(png: &[u8], iterations: u32) -> u64 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(Sha256::digest(black_box(png)));
    }
    per_call_ns(started, iterations)
}

fn samples_of(mut time_one: impl FnMut() -> Result<u64, String>) -> Result<Vec<u64>, String> {
    let mut samples = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        samples.push(time_one()?);
    }
    Ok(samples)
}

fn measure_size(width: u16, height: u16, iterations: u32) -> Value {
    let prepared = match prepare(width, height) {
        Ok(prepared) => prepared,
        Err(error) => {
            return json!({
                "width": width,
                "height": height,
                "error": error,
            });
        }
    };
    let mut prepared = prepared;
    let cached = samples_of(|| time_cached_poll(&mut prepared, iterations));
    let png_hit = samples_of(|| time_png_hit(&mut prepared, iterations));
    let id_match = samples_of(|| Ok(time_id_match(&mut prepared, iterations)));
    let pixel_compare = samples_of(|| Ok(time_pixel_compare(&prepared.pixels, iterations)));
    let sha256 = samples_of(|| Ok(time_sha256(&prepared.png, iterations)));
    let registrations = prepared.response.cursors.len();
    let id_stable = registrations == 1;
    json!({
        "width": width,
        "height": height,
        "pixelBytes": prepared.pixels.len() * 4,
        "pngBytes": prepared.png.len(),
        "registrations": registrations,
        "idStable": id_stable,
        "cachedPollNs": metric(cached),
        "pngHitNs": metric(png_hit),
        "idMatchNs": metric(id_match),
        "pixelCompareNs": metric(pixel_compare),
        "pngSha256Ns": metric(sha256),
    })
}

fn metric(result: Result<Vec<u64>, String>) -> Value {
    match result {
        Ok(samples) => summary(samples),
        Err(error) => json!({ "error": error }),
    }
}

fn measure_x11(display: &str, iterations: u32) -> Value {
    use x11rb::protocol::xfixes::ConnectionExt as _;

    let connect = match x11rb::connect(Some(display)) {
        Ok(connect) => connect,
        Err(error) => return json!({ "ran": false, "error": error.to_string() }),
    };
    let (connection, _) = connect;
    let version = match connection.xfixes_query_version(5, 0) {
        Ok(cookie) => cookie,
        Err(error) => {
            return json!({ "ran": false, "display": display, "error": error.to_string() });
        }
    };
    if let Err(error) = version.reply() {
        return json!({ "ran": false, "display": display, "error": error.to_string() });
    }
    let mut source = X11CursorSource {
        connection: Some(connection),
        last_image: None,
    };
    let Some(first) = source.cursor_image() else {
        return json!({ "ran": false, "display": display, "error": "GetCursorImage returned no reply" });
    };
    let serial = first.cursor_serial;
    let width = first.width;
    let height = first.height;
    let xhot = first.xhot;
    let yhot = first.yhot;
    let pixel_count = first.cursor_image.len();
    drop(first);
    if source.cursor_image().is_none() {
        return json!({ "ran": false, "display": display, "error": "warm-up GetCursorImage failed" });
    }

    let mut samples = Vec::with_capacity(RUNS);
    let mut errors = 0u32;
    let mut serial_changed = false;
    for _ in 0..RUNS {
        let started = Instant::now();
        for _ in 0..iterations {
            match source.cursor_image() {
                Some(reply) => {
                    if reply.cursor_serial != serial {
                        serial_changed = true;
                    }
                    black_box(reply.cursor_image.len());
                }
                None => errors += 1,
            }
        }
        samples.push(per_call_ns(started, iterations));
    }
    json!({
        "ran": true,
        "display": display,
        "iterations": iterations,
        "width": width,
        "height": height,
        "xhot": xhot,
        "yhot": yhot,
        "pixels": pixel_count,
        "cursorSerial": serial,
        "serialChanged": serial_changed,
        "errors": errors,
        "getCursorImageNs": summary(samples),
    })
}

const LIVE_SIZE: u16 = 64;
const LIVE_HOT_X: u16 = 4;
const LIVE_HOT_Y: u16 = 6;
const LIVE_HOT_X_MOVED: u16 = 20;
const LIVE_HOT_Y_MOVED: u16 = 8;

struct HeldXsetroot {
    child: Child,
}

impl HeldXsetroot {
    fn start(display: &str, hold_so: &str, cursor: &Path, mask: &Path) -> Result<Self, String> {
        let mut child = Command::new("xsetroot")
            .env("DISPLAY", display)
            .env("LD_PRELOAD", hold_so)
            .args(["-display", display, "-cursor"])
            .arg(cursor)
            .arg(mask)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| error.to_string())?;
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        while Instant::now() < deadline {
            if child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_some()
            {
                return Err("xsetroot exited before its cursor could be read".to_string());
            }
            if process_stopped(child.id()) {
                let moved = Command::new("xdotool")
                    .env("DISPLAY", display)
                    .args(["mousemove", "40", "40"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map_err(|error| error.to_string())?;
                if !moved.success() {
                    return Err("xdotool mousemove failed".to_string());
                }
                return Ok(Self { child });
            }
            std::thread::yield_now();
        }
        let _ = child.kill();
        let _ = child.wait();
        Err("xsetroot did not stop with the cursor held".to_string())
    }
}

impl Drop for HeldXsetroot {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn process_stopped(pid: u32) -> bool {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return false;
    };
    let Some(end) = stat.rfind(')') else {
        return false;
    };
    stat[end + 1..].split_whitespace().next() == Some("T")
}

fn write_xbm(
    path: &Path,
    name: &str,
    size: u16,
    hot_x: u16,
    hot_y: u16,
    on: impl Fn(u16, u16) -> bool,
) -> Result<(), String> {
    let row = usize::from(size).div_ceil(8);
    let mut bits = vec![0u8; row * usize::from(size)];
    for y in 0..size {
        for x in 0..size {
            if on(x, y) {
                let index = usize::from(y) * row + usize::from(x) / 8;
                bits[index] |= 1 << (x % 8);
            }
        }
    }
    let mut body = String::new();
    for (index, byte) in bits.iter().enumerate() {
        if index % 12 == 0 {
            body.push_str("\n   ");
        }
        body.push_str(&format!("0x{byte:02x}, "));
    }
    let text = format!(
        "#define {name}_width {size}\n#define {name}_height {size}\n#define {name}_x_hot {hot_x}\n#define {name}_y_hot {hot_y}\nstatic unsigned char {name}_bits[] = {{{body}\n}};\n"
    );
    std::fs::write(path, text).map_err(|error| error.to_string())
}

fn checker(x: u16, y: u16) -> bool {
    (x / 4 + y / 4) % 2 == 0
}

fn hotspot_near(actual: XY<f64>, x_hot: u16, y_hot: u16, size: u16) -> bool {
    let expect_x = f64::from(x_hot) / f64::from(size);
    let expect_y = f64::from(y_hot) / f64::from(size);
    (actual.x - expect_x).abs() < 1e-9 && (actual.y - expect_y).abs() < 1e-9
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PollKind {
    Unchanged,
    Read,
    Missing,
}

struct PollState {
    response: CursorActorResponse,
    last_image: Option<(Vec<u8>, [u64; 2], String)>,
    known: Option<super::CursorFingerprint>,
    dir: PathBuf,
    last_id: Option<String>,
}

struct Observed {
    kind: PollKind,
    hotspot: XY<f64>,
    png_sha: String,
    cursor_id: String,
}

fn poll_timed(state: &mut PollState) -> PollKind {
    match sample_cursor(state.known.as_ref()) {
        CursorSample::Unchanged => {
            black_box(state.last_id.as_deref());
            PollKind::Unchanged
        }
        CursorSample::Read {
            fingerprint,
            data: Some(data),
        } => {
            let id = cursor_id_for_image(
                data,
                &mut state.response,
                &state.dir,
                None,
                &mut state.last_image,
            );
            state.known = state.last_image.as_ref().and(fingerprint);
            state.last_id = Some(id);
            black_box(state.last_id.as_deref());
            PollKind::Read
        }
        CursorSample::Read { data: None, .. } => PollKind::Missing,
    }
}

fn poll_observed(state: &mut PollState) -> Result<Observed, String> {
    match sample_cursor(state.known.as_ref()) {
        CursorSample::Unchanged => {
            let hotspot = state
                .response
                .cursors
                .values()
                .find(|cursor| Some(cursor.id.to_string()) == state.last_id)
                .map(|cursor| cursor.hotspot)
                .ok_or_else(|| "unchanged poll had no registered cursor".to_string())?;
            Ok(Observed {
                kind: PollKind::Unchanged,
                hotspot,
                png_sha: String::new(),
                cursor_id: state.last_id.clone().unwrap_or_default(),
            })
        }
        CursorSample::Read {
            fingerprint,
            data: Some(data),
        } => {
            let hotspot = data.hotspot;
            let png_sha = hex::encode(Sha256::digest(&data.image));
            let cursor_id = cursor_id_for_image(
                data,
                &mut state.response,
                &state.dir,
                None,
                &mut state.last_image,
            );
            state.known = state.last_image.as_ref().and(fingerprint);
            state.last_id = Some(cursor_id.clone());
            Ok(Observed {
                kind: PollKind::Read,
                hotspot,
                png_sha,
                cursor_id,
            })
        }
        CursorSample::Read { data: None, .. } => {
            Err("X11 cursor read returned no image".to_string())
        }
    }
}

fn fresh_poll_state(dir: &Path) -> PollState {
    PollState {
        response: empty_response(),
        last_image: None,
        known: None,
        dir: dir.to_path_buf(),
        last_id: None,
    }
}

fn measure_live_cursor(display: &str, hold_so: &str, iterations: u32) -> Value {
    let dir = std::env::temp_dir().join(format!("cap-x11-live-{}", std::process::id()));
    let outcome = measure_live_cursor_in(display, hold_so, iterations, &dir);
    let _ = std::fs::remove_dir_all(&dir);
    outcome
}

fn measure_live_cursor_in(display: &str, hold_so: &str, iterations: u32, dir: &Path) -> Value {
    if let Err(error) = std::fs::create_dir_all(dir) {
        return json!({ "ran": false, "error": error.to_string() });
    }
    let mask = dir.join("mask.xbm");
    let shape = dir.join("shape.xbm");
    let moved = dir.join("moved-hot.xbm");
    let other = dir.join("other.xbm");
    if let Err(error) = write_xbm(&mask, "mask", LIVE_SIZE, 0, 0, |_, _| true)
        .and_then(|_| write_xbm(&shape, "shape", LIVE_SIZE, LIVE_HOT_X, LIVE_HOT_Y, checker))
        .and_then(|_| {
            write_xbm(
                &moved,
                "moved",
                LIVE_SIZE,
                LIVE_HOT_X_MOVED,
                LIVE_HOT_Y_MOVED,
                checker,
            )
        })
        .and_then(|_| {
            write_xbm(
                &other,
                "other",
                LIVE_SIZE,
                LIVE_HOT_X,
                LIVE_HOT_Y,
                |x, y| !checker(x, y),
            )
        })
    {
        return json!({ "ran": false, "error": error });
    }

    let held = match HeldXsetroot::start(display, hold_so, &shape, &mask) {
        Ok(held) => held,
        Err(error) => return json!({ "ran": false, "error": error }),
    };
    let mut state = fresh_poll_state(dir);
    let first = match poll_observed(&mut state) {
        Ok(first) => first,
        Err(error) => return json!({ "ran": false, "error": error }),
    };
    if first.kind != PollKind::Read
        || !hotspot_near(first.hotspot, LIVE_HOT_X, LIVE_HOT_Y, LIVE_SIZE)
    {
        return json!({
            "ran": false,
            "error": "first xsetroot cursor did not match the bitmap hotspot",
            "hotspot": [first.hotspot.x, first.hotspot.y],
        });
    }
    for _ in 0..3 {
        let _ = poll_timed(&mut state);
    }

    let mut samples = Vec::with_capacity(RUNS);
    let mut reads = 0u32;
    let mut unchanged = 0u32;
    let mut missing = 0u32;
    for _ in 0..RUNS {
        let mut run_reads = 0u32;
        let mut run_unchanged = 0u32;
        let mut run_missing = 0u32;
        let started = Instant::now();
        for _ in 0..iterations {
            match poll_timed(&mut state) {
                PollKind::Read => run_reads += 1,
                PollKind::Unchanged => run_unchanged += 1,
                PollKind::Missing => run_missing += 1,
            }
        }
        samples.push(per_call_ns(started, iterations));
        reads += run_reads;
        unchanged += run_unchanged;
        missing += run_missing;
    }
    drop(held);

    let moved_hold = match HeldXsetroot::start(display, hold_so, &moved, &mask) {
        Ok(held) => held,
        Err(error) => {
            return json!({ "ran": false, "error": error, "unchangedPollNs": summary(samples) });
        }
    };
    let moved_obs = match poll_observed(&mut state) {
        Ok(observed) => observed,
        Err(error) => return json!({ "ran": false, "error": error }),
    };
    drop(moved_hold);

    let other_hold = match HeldXsetroot::start(display, hold_so, &other, &mask) {
        Ok(held) => held,
        Err(error) => return json!({ "ran": false, "error": error }),
    };
    let other_obs = match poll_observed(&mut state) {
        Ok(observed) => observed,
        Err(error) => return json!({ "ran": false, "error": error }),
    };
    drop(other_hold);

    json!({
        "ran": true,
        "display": display,
        "size": LIVE_SIZE,
        "tool": "xsetroot -cursor held until XFreeCursor, then xdotool mousemove",
        "iterations": iterations,
        "reads": reads,
        "unchanged": unchanged,
        "missing": missing,
        "registrations": state.response.cursors.len(),
        "unchangedPollNs": summary(samples),
        "hotspotOnly": {
            "pngEqual": moved_obs.png_sha == first.png_sha,
            "hotspotChanged": moved_obs.hotspot != first.hotspot,
            "hotspotMatchesBitmap": hotspot_near(moved_obs.hotspot, LIVE_HOT_X_MOVED, LIVE_HOT_Y_MOVED, LIVE_SIZE),
            "treatedAsUnchanged": moved_obs.kind == PollKind::Unchanged,
            "cursorIdsDiffer": moved_obs.cursor_id != first.cursor_id,
            "firstHotspot": [first.hotspot.x, first.hotspot.y],
            "secondHotspot": [moved_obs.hotspot.x, moved_obs.hotspot.y],
        },
        "shapeChange": {
            "pngEqual": other_obs.png_sha == first.png_sha,
            "treatedAsUnchanged": other_obs.kind == PollKind::Unchanged,
            "cursorIdsDiffer": other_obs.cursor_id != first.cursor_id,
        },
    })
}

#[cfg(test)]
mod xsetroot_hotspot {
    use super::super::{CursorActorResponse, CursorSample, X11CursorSource, cursor_id_for_image};
    use super::{HeldXsetroot, checker, write_xbm};
    use cap_project::XY;
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};

    struct Xvfb {
        child: Child,
        number: u32,
    }

    impl Xvfb {
        fn start() -> Option<Self> {
            let (reader, writer) = std::io::pipe().ok()?;
            let child = Command::new("Xvfb")
                .args([
                    "-displayfd",
                    "1",
                    "-screen",
                    "0",
                    "640x480x24",
                    "-nolisten",
                    "tcp",
                ])
                .stdin(Stdio::null())
                .stdout(writer)
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let mut line = String::new();
            let Some(number) = BufReader::new(reader)
                .read_line(&mut line)
                .ok()
                .and_then(|_| line.trim().parse().ok())
            else {
                let mut child = child;
                let _ = child.kill();
                let _ = child.wait();
                return None;
            };
            Some(Self { child, number })
        }

        fn display(&self) -> String {
            format!(":{}", self.number)
        }
    }

    impl Drop for Xvfb {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn tool_exists(name: &str) -> bool {
        Command::new("sh")
            .args(["-c", &format!("command -v {name}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    fn hold_library() -> Option<std::path::PathBuf> {
        let src =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/x11-cursor-hold.c");
        let out = std::env::temp_dir().join(format!(
            "cap-x11-cursor-hold-test-{}.so",
            std::process::id()
        ));
        let compiled = Command::new("gcc")
            .args(["-shared", "-fPIC", "-o"])
            .arg(&out)
            .arg(&src)
            .args(["-ldl", "-lX11"])
            .status()
            .is_ok_and(|status| status.success());
        compiled.then_some(out)
    }

    fn connected(display: &str) -> X11CursorSource {
        use x11rb::protocol::xfixes::ConnectionExt as _;
        let (connection, _) = x11rb::connect(Some(display)).unwrap();
        connection
            .xfixes_query_version(5, 0)
            .unwrap()
            .reply()
            .unwrap();
        X11CursorSource {
            connection: Some(connection),
            last_image: None,
        }
    }

    #[test]
    fn an_xsetroot_hotspot_change_keeps_the_new_hotspot() {
        if !tool_exists("Xvfb")
            || !tool_exists("xsetroot")
            || !tool_exists("xdotool")
            || !tool_exists("gcc")
        {
            eprintln!("skipping live X11 cursor test, a required tool is missing");
            return;
        }
        let Some(server) = Xvfb::start() else {
            eprintln!("skipping live X11 cursor test, Xvfb did not start");
            return;
        };
        let Some(hold_so) = hold_library() else {
            eprintln!("skipping live X11 cursor test, gcc could not build the hold library");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let mask = dir.path().join("mask.xbm");
        let shape = dir.path().join("shape.xbm");
        let moved = dir.path().join("moved.xbm");
        write_xbm(&mask, "mask", 16, 0, 0, |_, _| true).unwrap();
        write_xbm(&shape, "shape", 16, 3, 4, checker).unwrap();
        write_xbm(&moved, "moved", 16, 9, 1, checker).unwrap();

        let display = server.display();
        let held =
            HeldXsetroot::start(&display, &hold_so.to_string_lossy(), &shape, &mask).unwrap();
        let mut source = connected(&display);
        let first = source.cursor_data(None).unwrap();
        let CursorSample::Read {
            fingerprint,
            data: Some(data),
        } = first
        else {
            panic!("xsetroot cursor was not read");
        };
        assert_eq!(data.hotspot, XY::new(3.0 / 16.0, 4.0 / 16.0));
        let first_png = data.image.clone();
        let mut response = CursorActorResponse {
            cursors: Default::default(),
            next_cursor_id: 0,
            moves: Vec::new(),
            clicks: Vec::new(),
            keyboard_presses: Vec::new(),
        };
        let mut last = None;
        let first_id = cursor_id_for_image(data, &mut response, dir.path(), None, &mut last);
        let known = last.as_ref().and(fingerprint);
        assert!(known.is_some());
        assert!(matches!(
            source.cursor_data(known.as_ref()).unwrap(),
            CursorSample::Unchanged
        ));
        drop(held);

        let held =
            HeldXsetroot::start(&display, &hold_so.to_string_lossy(), &moved, &mask).unwrap();
        let second = source.cursor_data(known.as_ref()).unwrap();
        let CursorSample::Read {
            data: Some(data), ..
        } = second
        else {
            panic!("same xsetroot pixels with a new hotspot were treated as unchanged");
        };
        assert_eq!(data.hotspot, XY::new(9.0 / 16.0, 1.0 / 16.0));
        assert_eq!(data.image, first_png);
        let second_id = cursor_id_for_image(data, &mut response, dir.path(), None, &mut last);
        assert_ne!(first_id, second_id);
        let recorded = response
            .cursors
            .values()
            .find(|cursor| cursor.id.to_string() == second_id)
            .unwrap();
        assert_eq!(recorded.hotspot, XY::new(9.0 / 16.0, 1.0 / 16.0));
        drop(held);
    }
}
