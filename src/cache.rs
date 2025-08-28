use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCache {
    pub bash_outputs: HashMap<String, CachedBashOutput>,
    pub perl_outputs: HashMap<String, CachedPerlOutput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedBashOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub last_modified: u64, // Unix timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedPerlOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub perl_code_hash: String, // SHA256 hash of the generated Perl code
    pub last_modified: u64, // Unix timestamp
}

impl CommandCache {
    pub fn new() -> Self {
        Self {
            bash_outputs: HashMap::new(),
            perl_outputs: HashMap::new(),
        }
    }

    fn compute_sha256(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn load() -> Self {
        let cache_file = "command_cache.json";
        match fs::read_to_string(cache_file) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(cache) => cache,
                    Err(_) => Self::new(),
                }
            }
            Err(_) => Self::new(),
        }
    }

    pub fn save(&self) {
        let cache_file = "command_cache.json";
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(cache_file, json);
        }
    }

    // Bash output caching methods
    pub fn get_cached_bash_output(&self, filename: &str) -> Option<&CachedBashOutput> {
        self.bash_outputs.get(filename)
    }

    pub fn is_bash_cache_valid(&self, filename: &str) -> bool {
        if let Some(cached) = self.bash_outputs.get(filename) {
            if let Ok(metadata) = fs::metadata(filename) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(modified_timestamp) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                        return modified_timestamp.as_secs() <= cached.last_modified;
                    }
                }
            }
        }
        false
    }

    pub fn update_bash_cache(&mut self, filename: &str, stdout: String, stderr: String, exit_code: i32) {
        let last_modified = if let Ok(metadata) = fs::metadata(filename) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(modified_timestamp) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                    modified_timestamp.as_secs()
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        self.bash_outputs.insert(filename.to_string(), CachedBashOutput {
            stdout,
            stderr,
            exit_code,
            last_modified,
        });
    }

    pub fn invalidate_bash_cache(&mut self, filename: &str) {
        self.bash_outputs.remove(filename);
        self.save();
    }

    // Perl output caching methods
    pub fn get_cached_perl_output(&self, filename: &str) -> Option<&CachedPerlOutput> {
        self.perl_outputs.get(filename)
    }

    pub fn is_perl_cache_valid(&self, filename: &str, perl_code: &str) -> bool {
        if let Some(cached) = self.perl_outputs.get(filename) {
            // Check if the Perl code hash matches (indicating the generated code hasn't changed)
            let current_hash = Self::compute_sha256(perl_code);
            if current_hash == cached.perl_code_hash {
                return true;
            }
        }
        false
    }

    pub fn update_perl_cache(&mut self, filename: &str, stdout: String, stderr: String, exit_code: i32, perl_code: &str) {
        let last_modified = if let Ok(metadata) = fs::metadata(filename) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(modified_timestamp) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                    modified_timestamp.as_secs()
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        let perl_code_hash = Self::compute_sha256(perl_code);

        self.perl_outputs.insert(filename.to_string(), CachedPerlOutput {
            stdout,
            stderr,
            exit_code,
            perl_code_hash,
            last_modified,
        });
    }
}
