use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal postmortem rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPostmortemRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalPostmortemRollupStatus {
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

impl std::fmt::Display for SeasonalPostmortemRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal postmortem rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollup {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub label: String,
    pub rate: f64,
    pub active: bool,
    pub attributes: BTreeMap<String, String>,
    pub status: SeasonalPostmortemRollupStatus,
}

impl SeasonalPostmortemRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPostmortemRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPostmortemRollupStatus) -> bool {
        match self.status {
            SeasonalPostmortemRollupStatus::Open => {
                return next == SeasonalPostmortemRollupStatus::Resolved;
            }
            SeasonalPostmortemRollupStatus::InReview => {
                return next == SeasonalPostmortemRollupStatus::Reopened;
            }
            SeasonalPostmortemRollupStatus::Resolved => {
                match next {
                    SeasonalPostmortemRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPostmortemRollupStatus::Reopened => {
                match next {
                    SeasonalPostmortemRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalPostmortemRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal postmortem rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.rate * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPostmortemRollup {
        SeasonalPostmortemRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            label: self.label.clone(),
            rate: self.rate,
            active: self.active,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal postmortem rollup in status open.
pub fn new_seasonal_postmortem_rollup(code: &str, title: &str) -> SeasonalPostmortemRollup {
    SeasonalPostmortemRollup {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        label: "".to_string(),
        rate: 0.0,
        active: false,
        attributes: BTreeMap::new(),
        status: SeasonalPostmortemRollupStatus::Open,
    }
}
