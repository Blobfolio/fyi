"""Exercise fyi_msg's Windows policies using hidden, real console sessions."""

import argparse
import ctypes
from ctypes import wintypes
import json
import shutil
from pathlib import Path
import subprocess
import sys
import time


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def until(predicate, message, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.005)
    raise AssertionError(message)


def read(path):
    return path.read_bytes() if path.exists() else b""


def main():
    require(sys.platform == "win32", "This test requires Windows")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target")
    parser.add_argument("--isolate-artifacts", action="store_true",
                        help="Exercise Cargo layouts with one library directory per artifact")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    command = ["cargo", "build", "-p", "fyi_msg", "--examples", "--all-features",
               "--locked", "--message-format=json"]
    if args.target:
        command += ["--target", args.target]
    built = subprocess.run(command, cwd=root, capture_output=True, text=True)
    if built.returncode:
        sys.stderr.write(built.stderr)
        raise SystemExit(built.returncode)
    artifacts = [json.loads(line) for line in built.stdout.splitlines() if line.startswith("{")]
    artifacts = [a for a in artifacts if a.get("reason") == "compiler-artifact"]
    examples = {a["target"]["name"]: a["executable"] for a in artifacts
                if "example" in a["target"]["kind"]}
    libraries = {a["target"]["name"]: f for a in artifacts for f in a["filenames"]
                 if f.endswith(".rlib")}
    output = root / "target/windows-console-tests"
    output.mkdir(parents=True, exist_ok=True)
    library_paths = {f for a in artifacts for f in a["filenames"] if f.endswith(".rlib")}
    if args.isolate_artifacts:
        # Reproduce Cargo versions that place libraries in separate output
        # directories instead of a shared debug/deps directory.
        relocated = {}
        for index, filename in enumerate(sorted(library_paths)):
            destination = output / "artifacts" / str(index) / Path(filename).name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(filename, destination)
            relocated[filename] = str(destination)
        library_paths = set(relocated.values())
        libraries = {name: relocated[filename] for name, filename in libraries.items()}
    dependency_dirs = sorted({str(Path(filename).parent) for filename in library_paths})

    # Build a small consumer to check native console flags and policy selection.
    # Each policy runs in its own process because registration is process-wide.
    source = output / "consumer.rs"
    source.write_text(r'''
use std::{path::PathBuf, sync::{Arc, atomic::Ordering::SeqCst}, time::Duration};
use windows_sys::Win32::System::Console::{
    GetConsoleMode, SetConsoleMode, GetStdHandle,
    STD_OUTPUT_HANDLE, STD_ERROR_HANDLE, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = &args[1];
    let ready = PathBuf::from(&args[2]);
    if mode == "ignored-parent" {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        unsafe extern "system" fn ignore_events(_: u32) -> i32 { 1 }
        // A parent can ignore Ctrl+C even though its child explicitly opts
        // into fyi_msg's signal handling. Keep the parent alive for Ctrl+Break.
        unsafe {
            assert_ne!(SetConsoleCtrlHandler(Some(ignore_events), 1), 0);
            assert_ne!(SetConsoleCtrlHandler(None, 1), 0);
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("keepalive-first").arg(&ready).status().unwrap();
        std::process::exit(status.code().unwrap_or(1));
    }
    let panic_log = ready.with_extension("panic");
    std::panic::set_hook(Box::new(move |info| { let _ = std::fs::write(&panic_log, info.to_string()); }));
    if mode == "ansi" {
        let mut expected = Vec::new();
        for stream in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            unsafe {
                let handle = GetStdHandle(stream);
                let mut flags = 0;
                assert_ne!(GetConsoleMode(handle, &raw mut flags), 0);
                // Start with ANSI disabled and verify other flags survive.
                assert_ne!(SetConsoleMode(handle, flags & !ENABLE_VIRTUAL_TERMINAL_PROCESSING), 0);
                expected.push((handle, flags | ENABLE_VIRTUAL_TERMINAL_PROCESSING));
            }
        }
        fyi_msg::enable_console_ansi();
        fyi_msg::enable_console_ansi();
        for (handle, flags) in expected {
            let mut actual = 0;
            unsafe { assert_ne!(GetConsoleMode(handle, &raw mut actual), 0); }
            assert_eq!(actual, flags);
        }
    } else if mode == "redirected" {
        for stream in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            let mut flags = 0;
            unsafe { assert_eq!(GetConsoleMode(GetStdHandle(stream), &raw mut flags), 0); }
        }
        fyi_msg::enable_console_ansi();
        print!("stdout unchanged");
        eprint!("stderr unchanged");
    } else {
        use fyi_msg::Progless;
        let (first, second) = if mode == "keepalive-first" {
            (Progless::sigint_keepalive(), Progless::sigint_two_strike())
        } else {
            (Progless::sigint_two_strike(), Progless::sigint_keepalive())
        };
        assert!(Arc::ptr_eq(first, second));
        // A forced exit must not wait for Rust's stderr lock.
        let _stderr_lock = (mode == "locked-stderr").then(|| std::io::stderr().lock());
        let progress = Progless::try_from(100).unwrap();
        std::fs::write(&ready, "ready").unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !first.load(SeqCst) {
            assert!(std::time::Instant::now() < deadline, "No interrupt received");
            std::thread::sleep(Duration::from_millis(5));
        }
        std::fs::write(&ready, "interrupted").unwrap();
        while !ready.with_extension("done").exists() {
            assert!(std::time::Instant::now() < deadline, "Test did not finish");
            std::thread::sleep(Duration::from_millis(5));
        }
        progress.finish();
    }
    std::fs::write(ready, "done").unwrap();
}
''', encoding="utf-8")
    consumer = output / "consumer.exe"
    compile_command = ["rustc", "--edition=2024", str(source), "-o", str(consumer),
                       "--extern", "fyi_msg=" + libraries["fyi_msg"],
                       "--extern", "windows_sys=" + libraries["windows_sys"]]
    # Direct --extern paths do not tell rustc where to find the libraries'
    # transitive dependencies. Use all paths reported by Cargo, not just the
    # directory containing fyi_msg.
    for directory in dependency_dirs:
        compile_command += ["-L", "dependency=" + directory]
    if args.target:
        compile_command += ["--target", args.target]
    subprocess.run(compile_command, cwd=root, check=True)

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    handler_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.DWORD)
    handler = handler_type(lambda event: True)
    kernel.SetConsoleCtrlHandler.argtypes = [handler_type, wintypes.BOOL]
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE

    def start(name, argv, console_output=False):
        ready = output / (name + ".ready")
        done = ready.with_suffix(".done")
        for path in [ready, done]:
            path.unlink(missing_ok=True)
        stdout = (output / (name + ".stdout")).open("wb")
        stderr = (output / (name + ".stderr")).open("wb")
        process = subprocess.Popen(
            argv, cwd=root, startupinfo=startup, creationflags=subprocess.CREATE_NEW_CONSOLE,
            **({} if console_output else {"stdin": subprocess.PIPE, "stdout": stdout, "stderr": stderr}),
        )
        if process.stdin:
            process.stdin.write(b"n\n")
            process.stdin.flush()
        return process, ready, stdout, stderr

    def attach(process):
        kernel.FreeConsole()
        require(kernel.AttachConsole(process.pid), "AttachConsole failed")
        require(kernel.SetConsoleCtrlHandler(handler, True), "Handler registration failed")

    def signal(event):
        require(kernel.GenerateConsoleCtrlEvent(event, 0), "GenerateConsoleCtrlEvent failed")

    def cleanup(process, stdout, stderr):
        kernel.FreeConsole()
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        if process.stdin:
            process.stdin.close()
        stdout.close()
        stderr.close()

    for name, executable, arguments, expected in [
        ("default", "progless-serial", [], 1),
        ("two-strike", "progless-sigint", [], 1),
        ("keepalive", "progless-sigint", ["--keepalive"], 0),
    ]:
        process, ready, stdout, stderr = start(name, [examples[executable], *arguments])
        path = Path(stderr.name)
        try:
            until(lambda: b"\x1b[?25l" in read(path), name + " did not register/hide the cursor")
            attach(process)
            signal(0)  # Ctrl+C.
            until(lambda: b"\x1b[?25h" in read(path), name + " did not restore the cursor")
            if name != "default":
                require(process.poll() is None, name + " exited on its first interrupt")
                signal(1)  # Ctrl+Break must share the selected policy.
                if name == "keepalive":
                    signal(0)
                    require(process.poll() is None, "Keepalive exited after repeated interrupts")
            require(process.wait(timeout=10) == expected, name + " returned the wrong exit code")
            if name == "keepalive":
                require(b"terminated naturally" in read(path), "Keepalive did not finish normally")
            print("PASS", name, "console events and cursor restoration")
        finally:
            cleanup(process, stdout, stderr)

    for name in ["ansi", "redirected", "keepalive-first", "two-strike-first", "locked-stderr", "ignored-parent"]:
        marker = output / (name + ".ready")
        process, ready, stdout, stderr = start(name, [str(consumer), name, str(marker)], name == "ansi")
        try:
            if name.endswith("-first") or name in ("locked-stderr", "ignored-parent"):
                until(lambda: read(ready) == b"ready", name + " did not initialize")
                attach(process)
                signal(0)
                until(lambda: read(ready) == b"interrupted", name + " did not see Ctrl+C")
                signal(1)
                if name in ("keepalive-first", "ignored-parent"):
                    time.sleep(0.05)
                    require(process.poll() is None, "Second registration replaced keepalive")
                    ready.with_suffix(".done").write_text("finish", encoding="utf-8")
                expected = 0 if name in ("keepalive-first", "ignored-parent") else 1
            else:
                expected = 0
            require(process.wait(timeout=10) == expected, name + " failed")
            if name == "redirected":
                require(read(Path(stdout.name)) == b"stdout unchanged", "stdout changed")
                require(read(Path(stderr.name)) == b"stderr unchanged", "stderr changed")
            print("PASS", name)
        finally:
            cleanup(process, stdout, stderr)


if __name__ == "__main__":
    main()
