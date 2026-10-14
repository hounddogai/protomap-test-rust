use super::*;

use acme_kit::text;

/// Where a probe summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSummary {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub category: String,
    pub floor_value: f64,
    pub batch_size: i64,
    pub window_days: i64,
    pub checksum: String,
    pub priority: i64,
    pub weight: f64,
    pub currency: String,
    pub status: ProbeSummaryStatus,
}

impl ProbeSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeSummaryStatus) -> bool {
        match self.status {
            ProbeSummaryStatus::Created => {
                return next == ProbeSummaryStatus::Validated || next == ProbeSummaryStatus::Retired;
            }
            ProbeSummaryStatus::Validated => {
                match next {
                    ProbeSummaryStatus::Published | ProbeSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeSummaryStatus::Published => {
                return next == ProbeSummaryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.priority as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeSummary {
        ProbeSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            floor_value: self.floor_value,
            batch_size: self.batch_size,
            window_days: self.window_days,
            checksum: self.checksum.clone(),
            priority: self.priority,
            weight: self.weight,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a probe summary in status created.
pub fn new_probe_summary(code: &str, title: &str) -> ProbeSummary {
    ProbeSummary {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        category: "".to_string(),
        floor_value: 0.0,
        batch_size: 0,
        window_days: 0,
        checksum: "".to_string(),
        priority: 0,
        weight: 0.0,
        currency: "".to_string(),
        status: ProbeSummaryStatus::Created,
    }
}
