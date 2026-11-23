fn main() {
    run();
}

#[cfg(not(target_os = "linux"))]
fn run() {
    eprintln!("x11 cursor cache benchmark runs on linux");
}

#[cfg(target_os = "linux")]
fn run() {
    if std::env::var_os("CAP_X11_CURSOR_BENCH_CHILD").is_none() {
        parent();
    } else {
        child();
    }
}

#[cfg(target_os = "linux")]
fn parent() {
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};

    let xvfb = Xvfb::start();
    let hold_so = compile_hold_so();
    let mut child = Command::new(std::env::current_exe().expect("current exe"));
    child.args(std::env::args().skip(1));
    child.env("CAP_X11_CURSOR_BENCH_CHILD", "1");
    if let Some(server) = &xvfb {
        child.env("DISPLAY", server.display());
    }
    if let Some(path) = &hold_so {
        child.env("CAP_X11_CURSOR_HOLD_SO", path);
    }
    let status = child.status().expect("bench child");
    drop(xvfb);
    std::process::exit(status.code().unwrap_or(1));

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
            let read = BufReader::new(reader).read_line(&mut line);
            let number = read.ok().and_then(|_| line.trim().parse().ok());
            let Some(number) = number else {
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

    fn compile_hold_so() -> Option<String> {
        let src =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/x11-cursor-hold.c");
        let out =
            std::env::temp_dir().join(format!("cap-x11-cursor-hold-{}.so", std::process::id()));
        let compiled = Command::new("gcc")
            .args(["-shared", "-fPIC", "-o"])
            .arg(&out)
            .arg(&src)
            .args(["-ldl", "-lX11"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        compiled.then(|| out.to_string_lossy().into_owned())
    }
}

#[cfg(target_os = "linux")]
fn child() {
    let args: Vec<String> = std::env::args().collect();
    let iterations = args
        .iter()
        .position(|arg| arg == "--iterations")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(2000u32);
    let display = std::env::var("DISPLAY").ok();
    let hold_so = std::env::var("CAP_X11_CURSOR_HOLD_SO").ok();
    let report = cap_recording::cursor::x11_cursor_cache_bench_report(
        iterations,
        display.as_deref(),
        hold_so.as_deref(),
    );
    println!("{report}");
}
