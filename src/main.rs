use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

fn main() {
    let root_dir = prompt_input("What is the name of your directory: ");
    let search_str = prompt_input("What word are you trying to find? ");

    println!("\n🚀 Initializing fast parallel scan...");

    // Set up real-time atomic counters that can be read safely across multiple threads
    let dir_counter = AtomicUsize::new(0);
    let file_counter = AtomicUsize::new(0);

    // Create a high-performance progress spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Processing... {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let start_time = Instant::now();
    let search_lower = search_str.trim().to_lowercase();

    // 1. Safe Parallel Traversal Engine with live-ticking feedback
    let mut matches: Vec<MatchEntry> = WalkDir::new(root_dir.trim())
        .into_iter()
        .filter_entry(|e| e.metadata().is_ok()) // Skip restricted directories instantly
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>()
        .into_par_iter() // Pivot to multi-threaded threadpool processing
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            let is_match = name.contains(&search_lower);

            if entry.file_type().is_dir() {
                let current_dirs = dir_counter.fetch_add(1, Ordering::Relaxed) + 1;
                let current_files = file_counter.load(Ordering::Relaxed);

                // Keep progress bar updated smoothly without sacrificing performance
                if current_dirs % 1000 == 0 {
                    pb.set_message(format!(
                        "🔍 {} dirs | {} files scanned",
                        current_dirs, current_files
                    ));
                }

                if is_match && !path.is_symlink() {
                    return Some(MatchEntry::new(MatchKind::Dir, path));
                }
            } else {
                let current_files = file_counter.fetch_add(1, Ordering::Relaxed) + 1;
                let current_dirs = dir_counter.load(Ordering::Relaxed);

                if current_files % 5000 == 0 {
                    pb.set_message(format!(
                        "🔍 {} dirs | {} files scanned",
                        current_dirs, current_files
                    ));
                }

                if is_match {
                    if entry.file_type().is_symlink() {
                        return Some(MatchEntry::new(MatchKind::Link, path));
                    } else {
                        return Some(MatchEntry::new(MatchKind::File, path));
                    }
                }
            }
            None
        })
        .collect();

    pb.finish_and_clear(); // Erase spinner lines once calculation ends

    let elapsed = start_time.elapsed();
    println!("📈 Scan completed in {:.6} seconds!", elapsed.as_secs_f64());
    println!(
        "Total Directories Checked = {}",
        dir_counter.load(Ordering::Relaxed)
    );
    println!(
        "Total Files Checked = {}\n",
        file_counter.load(Ordering::Relaxed)
    );

    if matches.is_empty() {
        println!("✨ No matches found for '{}'.", search_str.trim());
        return;
    }

    let confirmed = match run_picker(&mut matches) {
        Ok(confirmed) => confirmed,
        Err(err) => {
            eprintln!("Error rendering interactive menu: {err}");
            return;
        }
    };

    if !confirmed {
        println!("👋 Operation aborted successfully. No changes made.");
        return;
    }

    let mut selected: Vec<usize> = matches
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.checked)
        .map(|(idx, _)| idx)
        .collect();

    if selected.is_empty() {
        println!("👋 Operation aborted successfully. No changes made.");
        return;
    }

    let actions = vec![
        "🗑️  Permanently DELETE selected entries",
        "❌ Cancel and exit safely",
    ];

    println!(
        "\n⚡ Targeted matches confirmed: {} selected. Choose action:",
        selected.len()
    );
    let action_select = Select::with_theme(&ColorfulTheme::default())
        .items(&actions)
        .default(1)
        .interact()
        .unwrap_or(1);

    if action_select != 0 {
        println!("❌ Operation canceled safely.");
        return;
    }

    println!("\n⚠️  WARNING: You are executing a raw binary command deletion.");
    let confirm = prompt_input("Are you absolutely sure? Type 'yes' to proceed: ");

    if !confirm.trim().eq_ignore_ascii_case("yes") {
        println!("❌ Action canceled safely.");
        return;
    }

    selected.sort_by(|a, b| {
        matches[*b]
            .path
            .components()
            .count()
            .cmp(&matches[*a].path.components().count())
    });

    let mut deleted = 0usize;
    let mut failed = 0usize;
    for idx in selected {
        let entry = &matches[idx];
        if entry.path == Path::new("/") {
            eprintln!("❌ Refusing to delete /");
            failed += 1;
            continue;
        }

        let result = match entry.kind {
            MatchKind::Dir => fs::remove_dir_all(&entry.path),
            MatchKind::File | MatchKind::Link => fs::remove_file(&entry.path),
        };
        match result {
            Ok(()) => {
                deleted += 1;
                println!("🗑️  Deleted: {}", entry.path.display());
            }
            Err(err) => {
                failed += 1;
                eprintln!("❌ Failed to delete {}: {err}", entry.path.display());
            }
        }
    }

    if failed == 0 {
        println!("✅ Deleted {deleted}.");
    } else {
        println!("Finished with {deleted} deleted and {failed} failed.");
    }
}

fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

enum MatchKind {
    Dir,
    File,
    Link,
}

struct MatchEntry {
    kind: MatchKind,
    path: PathBuf,
    checked: bool,
}

impl MatchEntry {
    fn new(kind: MatchKind, path: PathBuf) -> Self {
        Self {
            kind,
            path,
            checked: false,
        }
    }

    fn label(&self) -> &'static str {
        match self.kind {
            MatchKind::Dir => "[DIR] ",
            MatchKind::File => "[FILE]",
            MatchKind::Link => "[LINK]",
        }
    }

    /// Path written to the system clipboard when this row is checked.
    /// A directory copies itself. A file or link copies its parent directory.
    fn clipboard_text(&self) -> String {
        match self.kind {
            MatchKind::Dir => self.path.display().to_string(),
            MatchKind::File | MatchKind::Link => self
                .path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(self.path.as_path())
                .display()
                .to_string(),
        }
    }
}

struct TermGuard;

impl TermGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        if let Err(err) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
            let _ = terminal::disable_raw_mode();
            return Err(err);
        }
        Ok(Self)
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn run_picker(matches: &mut [MatchEntry]) -> io::Result<bool> {
    let _guard = TermGuard::enter()?;
    let mut cursor = 0usize;
    let mut scroll = 0usize;
    let mut clipboard_error = false;

    loop {
        draw_picker(matches, cursor, &mut scroll, clipboard_error)?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up if !matches.is_empty() => {
                cursor = if cursor == 0 {
                    matches.len() - 1
                } else {
                    cursor - 1
                };
            }
            KeyCode::Down if !matches.is_empty() => {
                cursor = if cursor + 1 >= matches.len() {
                    0
                } else {
                    cursor + 1
                };
            }
            KeyCode::Char(' ') => {
                if let Some(entry) = matches.get_mut(cursor) {
                    entry.checked = !entry.checked;
                    let text = if entry.checked {
                        entry.clipboard_text()
                    } else {
                        String::new()
                    };
                    clipboard_error = !write_clipboard(&text);
                }
            }
            KeyCode::Enter => return Ok(true),
            KeyCode::Esc | KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            _ => {}
        }
    }
}

fn draw_picker(
    matches: &[MatchEntry],
    cursor: usize,
    scroll: &mut usize,
    clipboard_error: bool,
) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let width = cols as usize;
    let header_rows = 3u16;
    let footer_rows = 1u16;
    let visible = rows.saturating_sub(header_rows + footer_rows).max(1) as usize;

    if cursor < *scroll {
        *scroll = cursor;
    } else if cursor >= *scroll + visible {
        *scroll = cursor + 1 - visible;
    }
    let max_scroll = matches.len().saturating_sub(visible);
    if *scroll > max_scroll {
        *scroll = max_scroll;
    }

    let mut out = io::stdout();
    queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;
    queue!(
        out,
        Print(fit(
            "Space checks or unchecks. Up/Down moves. Enter continues. q or Esc aborts.",
            width
        )),
        MoveTo(0, 1),
        Print(fit(
            "A file copies its parent folder. A folder copies itself. Uncheck clears it.",
            width
        )),
        MoveTo(0, 2)
    )?;

    for row in 0..visible {
        let idx = *scroll + row;
        queue!(out, MoveTo(0, header_rows + row as u16))?;
        if idx >= matches.len() {
            continue;
        }

        let entry = &matches[idx];
        let mark = if entry.checked { "[x]" } else { "[ ]" };
        let line = fit(
            &format!("{} {} {}", mark, entry.label(), entry.path.display()),
            width,
        );
        if idx == cursor {
            queue!(out, SetAttribute(Attribute::Reverse))?;
        }
        queue!(out, Print(line), SetAttribute(Attribute::Reset))?;
    }

    let checked = matches.iter().filter(|entry| entry.checked).count();
    let mut footer = format!("{checked} checked · {}/{}", cursor + 1, matches.len());
    if clipboard_error {
        footer.push_str(" · clipboard update failed");
    }
    queue!(
        out,
        MoveTo(0, rows.saturating_sub(1)),
        Print(fit(&footer, width))
    )?;
    out.flush()
}

fn fit(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut chars = text.chars();
    let mut fitted = String::new();
    for _ in 0..width {
        match chars.next() {
            Some(ch) => fitted.push(ch),
            None => return fitted,
        }
    }
    if chars.next().is_some() {
        fitted.pop();
        fitted.push('…');
    }
    fitted
}

fn write_clipboard(text: &str) -> bool {
    if text.is_empty() {
        // Empty pbcopy can report success and leave the previous value in place.
        // Reading the pasteboard afterward publishes the clear to other apps.
        let cleared = Command::new("osascript")
            .args(["-e", "set the clipboard to \"\""])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        let _ = Command::new("pbpaste").output();
        return cleared;
    }

    let mut child = match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(_) => return false,
    };
    let wrote = child
        .stdin
        .take()
        .is_some_and(|mut stdin| stdin.write_all(text.as_bytes()).is_ok());
    let exited = matches!(child.wait(), Ok(status) if status.success());
    wrote && exited
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_copies_its_parent_directory() {
        let entry = MatchEntry::new(MatchKind::File, PathBuf::from("/tmp/apps/word.txt"));
        assert_eq!(entry.clipboard_text(), "/tmp/apps");
    }

    #[test]
    fn directory_copies_itself() {
        let entry = MatchEntry::new(MatchKind::Dir, PathBuf::from("/tmp/apps/word.app"));
        assert_eq!(entry.clipboard_text(), "/tmp/apps/word.app");
    }

    #[test]
    fn link_copies_its_parent_directory() {
        let entry = MatchEntry::new(MatchKind::Link, PathBuf::from("/tmp/apps/word.lnk"));
        assert_eq!(entry.clipboard_text(), "/tmp/apps");
    }
}
