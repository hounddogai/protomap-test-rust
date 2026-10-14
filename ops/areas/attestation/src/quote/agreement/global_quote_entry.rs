use super::*;

use acme_kit::text;

/// Where a global quote entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalQuoteEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalQuoteEntryStatus {
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

impl std::fmt::Display for GlobalQuoteEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global quote entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteEntry {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub starts_at: i64,
    pub principal: String,
    pub status: GlobalQuoteEntryStatus,
}

impl GlobalQuoteEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalQuoteEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalQuoteEntryStatus) -> bool {
        match self.status {
            GlobalQuoteEntryStatus::Idle => {
                return next == GlobalQuoteEntryStatus::Warming || next == GlobalQuoteEntryStatus::Stopped;
            }
            GlobalQuoteEntryStatus::Warming => {
                return next == GlobalQuoteEntryStatus::Ready;
            }
            GlobalQuoteEntryStatus::Ready => {
                match next {
                    GlobalQuoteEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalQuoteEntryStatus::Draining => {
                return next == GlobalQuoteEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalQuoteEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global quote entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global quote entry", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.starts_at as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalQuoteEntry {
        GlobalQuoteEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            starts_at: self.starts_at,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global quote entry in status idle.
pub fn new_global_quote_entry(code: &str, label: &str) -> GlobalQuoteEntry {
    GlobalQuoteEntry {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        starts_at: 0,
        principal: "".to_string(),
        status: GlobalQuoteEntryStatus::Idle,
    }
}
