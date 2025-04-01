use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::utils::ExclusionManager;
use crate::utils::ProgressTracker;
use globwalk;

pub fn find_sql_files<T: ProgressTracker>(
    dir: &Path,
    recursive: bool,
    exclusion_manager: &ExclusionManager,
    mut progress: Option<&mut T>,
) -> Result<Vec<PathBuf>> {
    println!("\n🔍 Searching for SQL files in: {}", dir.display());
    println!("   Recursive search: {}", recursive);
    
    let mut sql_files = Vec::new();
    
    if !dir.exists() {
        println!("⚠️  Directory does not exist: {}", dir.display());
        return Ok(sql_files);
    }
    
    // Use globwalk with appropriate pattern
    let pattern = if recursive { "**/*.sql" } else { "*.sql" };
    
    match globwalk::GlobWalkerBuilder::from_patterns(dir, &[pattern])
        .follow_links(true)
        .build() {
        Ok(walker) => {
            for entry in walker {
                match entry {
                    Ok(file) => {
                        let path = file.path();
                        println!("   Found SQL file: {}", path.display());
                        
                        // Check exclusions
                        let file_content = std::fs::read_to_string(path)?;
                        let (should_exclude_by_pattern, pattern) = exclusion_manager.should_exclude_file(path, dir);
                        let (should_exclude_by_tags, tag) = exclusion_manager.should_exclude_by_tags(&file_content);
                        
                        if should_exclude_by_pattern {
                            let pattern_str = pattern.unwrap_or_else(|| "unknown".to_string());
                            println!("   ⚠️  Excluded by pattern: {} (pattern: {})", path.display(), pattern_str);
                            if let Some(ref mut tracker) = progress {
                                tracker.log_excluded_file(
                                    &path.to_string_lossy(),
                                    &pattern_str,
                                );
                            }
                            continue;
                        }
                        
                        if should_exclude_by_tags {
                            let tag_str = tag.unwrap_or_else(|| "unknown".to_string());
                            println!("   ⚠️  Excluded by tag: {} (tag: {})", path.display(), tag_str);
                            if let Some(ref mut tracker) = progress {
                                tracker.log_excluded_tag(
                                    &path.to_string_lossy(),
                                    &tag_str,
                                );
                            }
                            continue;
                        }
                        
                        println!("   ✓ Adding file: {}", path.display());
                        sql_files.push(path.to_path_buf());
                    }
                    Err(e) => println!("   ⚠️  Error processing entry: {}", e),
                }
            }
        }
        Err(e) => println!("   ⚠️  Error creating glob walker: {}", e),
    }
    
    println!("\n✓ Found {} SQL files", sql_files.len());
    Ok(sql_files)
} 