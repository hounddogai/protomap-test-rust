use super::*;

use acme_kit::text;

/// Where a rolling silence score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSilenceScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingSilenceScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RollingSilenceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling silence score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScore {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub ends_at: i64,
    pub currency: String,
    pub status: RollingSilenceScoreStatus,
}

impl RollingSilenceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSilenceScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSilenceScoreStatus) -> bool {
        match self.status {
            RollingSilenceScoreStatus::Open => {
                return next == RollingSilenceScoreStatus::Resolved;
            }
            RollingSilenceScoreStatus::InReview => {
                return next == RollingSilenceScoreStatus::Archived;
            }
            RollingSilenceScoreStatus::Resolved => {
                return next == RollingSilenceScoreStatus::Reopened || next == RollingSilenceScoreStatus::Archived;
            }
            RollingSilenceScoreStatus::Reopened => {
                match next {
                    RollingSilenceScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingSilenceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling silence score {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.window_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSilenceScore {
        RollingSilenceScore {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            ends_at: self.ends_at,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling silence score in status open.
pub fn new_rolling_silence_score(code: &str, name: &str) -> RollingSilenceScore {
    RollingSilenceScore {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        tags: Vec::new(),
        quota_units: 0,
        ends_at: 0,
        currency: "".to_string(),
        status: RollingSilenceScoreStatus::Open,
    }
}
