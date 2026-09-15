//! Terminal/REPL backend using `portable-pty`.
//!
//! Each open terminal in the frontend gets a `TerminalSession` identified by a
//! UUID. The frontend writes keystrokes to the session via the `term_write`
//! command and receives output via the `term_output` event.
//!
//! On Windows, the default shell is PowerShell (Core if available, otherwise
//! Windows PowerShell). On Unix it falls back to `$SHELL` or `/bin/bash`.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutput {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnOptions {
    pub cmd: Option<String>,
    pub cwd: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    pub env: Option<HashMap<String, String>>,
}

pub struct TerminalSession {
    pub id: String,
    pub writer: Box<dyn Write + Send>,
    pub master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    pub killer: Arc<Mutex<Option<Box<dyn Fn() + Send>>>>,
}

impl TerminalSession {
    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer.write_all(data).map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn kill(&self) {
        if let Ok(guard) = self.killer.lock() {
            if let Some(f) = guard.as_ref() {
                f();
            }
        }
    }
}

/// Global session registry.
static SESSIONS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, Arc<Mutex<TerminalSession>>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Pick a sensible default shell for the host platform.
///
/// Windows: prefer `pwsh.exe` (PowerShell Core), fall back to
/// `powershell.exe` (Windows PowerShell 5.x), fall back to `cmd.exe`.
/// Unix:    prefer `$SHELL`, fall back to `/bin/bash`, then `/bin/sh`.
pub fn default_shell() -> String {
    if cfg!(target_os = "windows") {
        // Look for PowerShell Core (pwsh.exe) on PATH.
        if let Ok(paths) = std::env::var("PATH") {
            for dir in paths.split(';') {
                if dir.is_empty() {
                    continue;
                }
                let pwsh = std::path::Path::new(dir).join("pwsh.exe");
                if pwsh.exists() {
                    return pwsh.display().to_string();
                }
            }
        }
        // Check the well-known Windows PowerShell location.
        let ps5 = std::path::Path::new(
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        );
        if ps5.exists() {
            return ps5.display().to_string();
        }
        // Last resort: cmd.exe
        return "cmd.exe".to_string();
    }

    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

pub fn spawn_session(app: AppHandle, opts: SpawnOptions) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let pty_system = native_pty_system();

    let cols = opts.cols.unwrap_or(80);
    let rows = opts.rows.unwrap_or(24);
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("openpty: {}", e))?;

    let cmd_str = opts.cmd.clone().unwrap_or_else(default_shell);

    let mut cmd = CommandBuilder::new(&cmd_str);
    if let Some(cwd) = &opts.cwd {
        cmd.cwd(cwd);
    } else if cfg!(target_os = "windows") {
        // Default to the user profile on Windows.
        if let Ok(home) = std::env::var("USERPROFILE") {
            cmd.cwd(home);
        }
    } else if let Ok(home) = std::env::var("HOME") {
        cmd.cwd(home);
    }
    if let Some(env) = &opts.env {
        for (k, v) in env {
            cmd.env(k, v);
        }
    }
    // Make sure the terminal type is set so colored prompts work.
    cmd.env("TERM", "xterm-256color");

    // On Windows, launch PowerShell with `-NoLogo` for a cleaner startup.
    // We detect PowerShell by checking if the executable name contains "powershell" or "pwsh".
    #[cfg(target_os = "windows")]
    {
        let lower = cmd_str.to_lowercase();
        if lower.contains("pwsh.exe") || lower.contains("powershell.exe") || lower.contains("pwsh") || lower.contains("powershell") {
            cmd.arg("-NoLogo");
            cmd.arg("-NoExit");
            // Run an init script that prints a banner.
            cmd.arg("-Command");
            cmd.arg("$Host.UI.RawUI.WindowTitle = 'BookForge REPL'; Write-Host ''; Write-Host 'BookForge PowerShell session.' -ForegroundColor Cyan; Write-Host ''");
        }
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("spawn_command ({}): {}", cmd_str, e))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("take_writer: {}", e))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("try_clone_reader: {}", e))?;

    let master = pair.master;
    let master_box: Box<dyn portable_pty::MasterPty + Send> = master;
    let master_arc = Arc::new(Mutex::new(master_box));

    let session_id = id.clone();
    let app_clone = app.clone();
    let child_arc = Arc::new(Mutex::new(child));
    let child_for_kill = child_arc.clone();

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_clone.emit(
                        "term_output",
                        TerminalOutput {
                            session_id: session_id.clone(),
                            data,
                        },
                    );
                }
                Err(_) => break,
            }
        }
        let _ = app_clone.emit(
            "term_output",
            TerminalOutput {
                session_id: session_id.clone(),
                data: "\r\n[process exited]\r\n".to_string(),
            },
        );
        // Clean up
        if let Ok(mut g) = SESSIONS.lock() {
            g.remove(&session_id);
        }
        let _ = child_for_kill.lock().map(|mut c| c.kill().ok());
    });

    let session = TerminalSession {
        id: id.clone(),
        writer,
        master: master_arc,
        killer: Arc::new(Mutex::new(Some(Box::new(move || {
            let _ = child_arc.lock().map(|mut c| c.kill().ok());
        })))),
    };

    SESSIONS
        .lock()
        .map_err(|e| e.to_string())?
        .insert(id.clone(), Arc::new(Mutex::new(session)));

    Ok(id)
}

pub fn write_session(id: &str, data: &[u8]) -> Result<(), String> {
    let g = SESSIONS.lock().map_err(|e| e.to_string())?;
    let session = g
        .get(id)
        .cloned()
        .ok_or_else(|| format!("Unknown terminal session: {}", id))?;
    drop(g);
    let mut s = session.lock().map_err(|e| e.to_string())?;
    s.write(data)
}

pub fn resize_session(id: &str, cols: u16, rows: u16) -> Result<(), String> {
    let g = SESSIONS.lock().map_err(|e| e.to_string())?;
    let session = g
        .get(id)
        .cloned()
        .ok_or_else(|| format!("Unknown terminal session: {}", id))?;
    drop(g);
    let s = session.lock().map_err(|e| e.to_string())?;
    if let Ok(m) = s.master.lock() {
        let _ = m.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }
    Ok(())
}

pub fn kill_session(id: &str) -> Result<(), String> {
    let g = SESSIONS.lock().map_err(|e| e.to_string())?;
    let session = g.get(id).cloned();
    drop(g);
    if let Some(s) = session {
        let s = s.lock().map_err(|e| e.to_string())?;
        s.kill();
    }
    Ok(())
}
