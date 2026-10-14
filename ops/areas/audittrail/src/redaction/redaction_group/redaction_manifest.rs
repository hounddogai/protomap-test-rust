use super::*;

use acme_kit::text;

/// Where a redaction manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RedactionManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RedactionManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction manifest as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifest {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub ttl_seconds: i64,
    pub category: String,
    pub status: RedactionManifestStatus,
}

impl RedactionManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionManifestStatus) -> bool {
        match self.status {
            RedactionManifestStatus::Idle => {
                match next {
                    RedactionManifestStatus::Draining | RedactionManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionManifestStatus::Warming => {
                return next == RedactionManifestStatus::Ready || next == RedactionManifestStatus::Stopped;
            }
            RedactionManifestStatus::Ready => {
                match next {
                    RedactionManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionManifestStatus::Draining => {
                match next {
                    RedactionManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionManifest {
        RedactionManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a redaction manifest in status idle.
pub fn new_redaction_manifest(code: &str, title: &str) -> RedactionManifest {
    RedactionManifest {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        ttl_seconds: 0,
        category: "".to_string(),
        status: RedactionManifestStatus::Idle,
    }
}
