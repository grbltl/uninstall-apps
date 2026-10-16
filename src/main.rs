use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;
use copypasta::{ClipboardContext, ClipboardProvider};

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
            .unwrap()
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let start_time = Instant::now();
    let search_lower = search_str.trim().to_lowercase();

    // 1. Safe Parallel Traversal Engine with live-ticking feedback
    let all_matches_collected: Vec<(String, PathBuf)> = WalkDir::new(root_dir.trim())
        .into_iter()
        .filter_entry(|e| e.metadata().is_ok()) // Skip restricted directories instantly
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>()
        .into_par_iter() // Pivot to multi-threaded threadpool processing
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let is_match = name.contains(&search_lower);

            if entry.file_type().is_dir() {
                let current_dirs = dir_counter.fetch_add(1, Ordering::Relaxed) + 1;
                let current_files = file_counter.load(Ordering::Relaxed);
                
                // Keep progress bar updated smoothly without sacrificing performance
                if current_dirs % 1000 == 0 {
                    pb.set_message(format!("🔍 {} dirs | {} files scanned", current_dirs, current_files));
                }
                
                if is_match && !path.is_symlink() {
                    return Some(("[DIR] ".to_string(), path));
                }
            } else {
                let current_files = file_counter.fetch_add(1, Ordering::Relaxed) + 1;
                let current_dirs = dir_counter.load(Ordering::Relaxed);
                
                if current_files % 5000 == 0 {
                    pb.set_message(format!("🔍 {} dirs | {} files scanned", current_dirs, current_files));
                }

                if is_match {
                    if entry.file_type().is_symlink() {
                        return Some(("[LINK]".to_string(), path));
                    } else {
                        return Some(("[FILE]".to_string(), path));
                    }
                }
            }
            None
        })
        .collect();

    pb.finish_and_clear(); // Erase spinner lines once calculation ends

    let elapsed = start_time.elapsed();
    println!("📈 Scan completed in {:.6} seconds!", elapsed.as_secs_f64());
    println!("Total Directories Checked = {}", dir_counter.load(Ordering::Relaxed));
    println!("Total Files Checked = {}\n", file_counter.load(Ordering::Relaxed));

    if all_matches_collected.is_empty() {
        println!("✨ No matches found for '{}'.", search_str.trim());
        return;
    }

    // 2. Interactive Selection Menu Layout with explicit exit options
    println!("💡 Selection Guide:");
    println!("   • Use [Spacebar] to tick multiple paths");
    println!("   • Choose option 0 or press [Enter] immediately to abort safely\n");

    // Construct options menu vector with an explicit escape option at index 0
    let mut menu_options = vec!["❌ [ EXIT / ABORT OPERATION ]".to_string()];
    for (label, path) in &all_matches_collected {
        menu_options.push(format!("{} {:?}", label, path));
    }

    let chosen_indices = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select targets to modify")
        .items(&menu_options)
        .interact();

    match chosen_indices {
        Ok(selections) => {
            // Safety Check: If user explicitly checked the Exit item, or left it completely blank, abort!
            if selections.is_empty() || selections.contains(&0) {
                println!("👋 Operation aborted successfully. No changes made.");
                return;
            }

            // Remap adjusted index values because of the extra option injected at position zero
            let target_selections: Vec<usize> = selections.iter().map(|idx| idx - 1).collect();

            // 3. Action Workflow Step Selection Options Layout
            let actions = vec![
                "📋 Copy selected paths to Clipboard",
                "🗑️  Permanently DELETE selected entries",
                "❌ Cancel and exit safely"
            ];
            
            println!("\n⚡ Targeted matches confirmed: {} selected. Choose action:", target_selections.len());
            let action_select = Select::with_theme(&ColorfulTheme::default())
                .items(&actions)
                .default(0)
                .interact()
                .unwrap_or(2);

            match action_select {
                0 => {
                    let mut copied_text = String::new();
                    for idx in &target_selections {
                        let (_, path) = &all_matches_collected[*idx];
                        copied_text.push_str(&format!("{}\n", path.to_string_lossy()));
                    }
                    
                    if let Ok(mut ctx) = ClipboardContext::new() {
                        if ctx.set_contents(copied_text).is_ok() {
                            println!("\n✅ Successfully copied {} file path(s) to Mac clipboard!", target_selections.len());
                        } else {
                            eprintln!("❌ Failed to write to system clipboard.");
                        }
                    }
                }
                1 => {
                    println!("\n⚠️  WARNING: You are executing a raw binary command deletion.");
                    let confirm = prompt_input("Are you absolutely sure? Type 'yes' to proceed: ");
                    
                    if confirm.trim().eq_ignore_ascii_case("yes") {
                        for idx in target_selections {
                            let (label, path) = &all_matches_collected[idx];
                            if label.contains("DIR") {
                                let _ = fs::remove_dir_all(path);
                                println!("🗑️  Deleted Directory: {:?}", path);
                            } else {
                                let _ = fs::remove_file(path);
                                println!("🗑️  Deleted File/Link: {:?}", path);
                            }
                        }
                        println!("✅ Targets successfully cleared.");
                    } else {
                        println!("❌ Action canceled safely.");
                    }
                }
                _ => println!("❌ Operation canceled safely."),
            }
        }
        Err(_) => println!("Error rendering interactive menu frames."),
    }
}

fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

