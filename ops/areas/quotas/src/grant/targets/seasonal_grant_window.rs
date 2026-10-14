use super::*;

use acme_kit::text;

/// Where a seasonal grant window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalGrantWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalGrantWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SeasonalGrantWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal grant window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindow {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub checksum: String,
    pub duration_minutes: i64,
    pub category: String,
    pub segment: String,
    pub batch_size: i64,
    pub issuer: String,
    pub ratio: f64,
    pub status: SeasonalGrantWindowStatus,
}

impl SeasonalGrantWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalGrantWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalGrantWindowStatus) -> bool {
        match self.status {
            SeasonalGrantWindowStatus::New => {
                return next == SeasonalGrantWindowStatus::Done || next == SeasonalGrantWindowStatus::Failed;
            }
            SeasonalGrantWindowStatus::Queued => {
                match next {
                    SeasonalGrantWindowStatus::Running | SeasonalGrantWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalGrantWindowStatus::Running => {
                match next {
                    SeasonalGrantWindowStatus::Done | SeasonalGrantWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalGrantWindowStatus::Done => {
                return next == SeasonalGrantWindowStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalGrantWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal grant window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal grant window", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalGrantWindow {
        SeasonalGrantWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            category: self.category.clone(),
            segment: self.segment.clone(),
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal grant window in status new.
pub fn new_seasonal_grant_window(code: &str, title: &str) -> SeasonalGrantWindow {
    SeasonalGrantWindow {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        ttl_seconds: 0,
        checksum: "".to_string(),
        duration_minutes: 0,
        category: "".to_string(),
        segment: "".to_string(),
        batch_size: 0,
        issuer: "".to_string(),
        ratio: 0.0,
        status: SeasonalGrantWindowStatus::New,
    }
}
