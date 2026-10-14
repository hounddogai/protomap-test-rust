use super::*;

use acme_kit::text;

/// Where a seasonal limit metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalLimitMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalLimitMetricStatus {
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

impl std::fmt::Display for SeasonalLimitMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal limit metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetric {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub duration_minutes: i64,
    pub archived: bool,
    pub threshold: f64,
    pub starts_at: i64,
    pub quantity: i64,
    pub host: String,
    pub checksum: String,
    pub status: SeasonalLimitMetricStatus,
}

impl SeasonalLimitMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalLimitMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalLimitMetricStatus) -> bool {
        match self.status {
            SeasonalLimitMetricStatus::Open => {
                return next == SeasonalLimitMetricStatus::Resolved || next == SeasonalLimitMetricStatus::Reopened;
            }
            SeasonalLimitMetricStatus::InReview => {
                match next {
                    SeasonalLimitMetricStatus::Reopened | SeasonalLimitMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalLimitMetricStatus::Resolved => {
                match next {
                    SeasonalLimitMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalLimitMetricStatus::Reopened => {
                match next {
                    SeasonalLimitMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalLimitMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal limit metric {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.starts_at as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalLimitMetric {
        SeasonalLimitMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            duration_minutes: self.duration_minutes,
            archived: self.archived,
            threshold: self.threshold,
            starts_at: self.starts_at,
            quantity: self.quantity,
            host: self.host.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal limit metric in status open.
pub fn new_seasonal_limit_metric(code: &str, name: &str) -> SeasonalLimitMetric {
    SeasonalLimitMetric {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        duration_minutes: 0,
        archived: false,
        threshold: 0.0,
        starts_at: 0,
        quantity: 0,
        host: "".to_string(),
        checksum: "".to_string(),
        status: SeasonalLimitMetricStatus::Open,
    }
}
