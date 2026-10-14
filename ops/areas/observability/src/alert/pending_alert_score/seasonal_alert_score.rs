use super::*;

use acme_kit::text;

/// Where a seasonal alert score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAlertScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalAlertScoreStatus {
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

impl std::fmt::Display for SeasonalAlertScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal alert score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScore {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub version: i64,
    pub unit: String,
    pub status: SeasonalAlertScoreStatus,
}

impl SeasonalAlertScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAlertScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAlertScoreStatus) -> bool {
        match self.status {
            SeasonalAlertScoreStatus::Open => {
                match next {
                    SeasonalAlertScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAlertScoreStatus::InReview => {
                match next {
                    SeasonalAlertScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAlertScoreStatus::Resolved => {
                match next {
                    SeasonalAlertScoreStatus::Reopened | SeasonalAlertScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAlertScoreStatus::Reopened => {
                match next {
                    SeasonalAlertScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalAlertScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal alert score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.version as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAlertScore {
        SeasonalAlertScore {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            version: self.version,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal alert score in status open.
pub fn new_seasonal_alert_score(code: &str, title: &str) -> SeasonalAlertScore {
    SeasonalAlertScore {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        version: 0,
        unit: "".to_string(),
        status: SeasonalAlertScoreStatus::Open,
    }
}
