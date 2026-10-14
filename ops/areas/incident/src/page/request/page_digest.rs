use super::*;

use acme_kit::text;

/// Where a page digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageDigestStatus {
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

impl std::fmt::Display for PageDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageDigest {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub retries: i64,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub status: PageDigestStatus,
}

impl PageDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageDigestStatus) -> bool {
        match self.status {
            PageDigestStatus::Idle => {
                return next == PageDigestStatus::Ready || next == PageDigestStatus::Draining;
            }
            PageDigestStatus::Warming => {
                return next == PageDigestStatus::Draining || next == PageDigestStatus::Stopped;
            }
            PageDigestStatus::Ready => {
                match next {
                    PageDigestStatus::Draining | PageDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageDigestStatus::Draining => {
                match next {
                    PageDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "page digest", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageDigest {
        PageDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            retries: self.retries,
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page digest in status idle.
pub fn new_page_digest(code: &str, name: &str) -> PageDigest {
    PageDigest {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        retries: 0,
        tags: Vec::new(),
        grace_days: 0,
        status: PageDigestStatus::Idle,
    }
}
