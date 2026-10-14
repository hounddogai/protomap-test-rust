use super::*;

use acme_kit::text;

/// Where a tiered token bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTokenBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredTokenBundleStatus {
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

impl std::fmt::Display for TieredTokenBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered token bundle as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundle {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub updated_at: i64,
    pub capacity: i64,
    pub tags: Vec<String>,
    pub batch_size: i64,
    pub status: TieredTokenBundleStatus,
}

impl TieredTokenBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTokenBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTokenBundleStatus) -> bool {
        match self.status {
            TieredTokenBundleStatus::Idle => {
                return next == TieredTokenBundleStatus::Warming;
            }
            TieredTokenBundleStatus::Warming => {
                return next == TieredTokenBundleStatus::Draining;
            }
            TieredTokenBundleStatus::Ready => {
                return next == TieredTokenBundleStatus::Stopped;
            }
            TieredTokenBundleStatus::Draining => {
                return next == TieredTokenBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTokenBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered token bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered token bundle", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.batch_size as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTokenBundle {
        TieredTokenBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            capacity: self.capacity,
            tags: self.tags.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered token bundle in status idle.
pub fn new_tiered_token_bundle(code: &str, name: &str) -> TieredTokenBundle {
    TieredTokenBundle {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        updated_at: 0,
        capacity: 0,
        tags: Vec::new(),
        batch_size: 0,
        status: TieredTokenBundleStatus::Idle,
    }
}
