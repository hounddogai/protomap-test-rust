use super::*;

use acme_kit::text;

/// Where a seasonal usage summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalUsageSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalUsageSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SeasonalUsageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal usage summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageSummary {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub segment: String,
    pub category: String,
    pub ttl_seconds: i64,
    pub status: SeasonalUsageSummaryStatus,
}

impl SeasonalUsageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalUsageSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalUsageSummaryStatus) -> bool {
        match self.status {
            SeasonalUsageSummaryStatus::Draft => {
                match next {
                    SeasonalUsageSummaryStatus::Suspended | SeasonalUsageSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalUsageSummaryStatus::Pending => {
                match next {
                    SeasonalUsageSummaryStatus::Suspended | SeasonalUsageSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalUsageSummaryStatus::Active => {
                match next {
                    SeasonalUsageSummaryStatus::Suspended | SeasonalUsageSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalUsageSummaryStatus::Suspended => {
                match next {
                    SeasonalUsageSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: SeasonalUsageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal usage summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> SeasonalUsageSummary {
        SeasonalUsageSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            segment: self.segment.clone(),
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal usage summary in status draft.
pub fn new_seasonal_usage_summary(code: &str, title: &str) -> SeasonalUsageSummary {
    SeasonalUsageSummary {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        segment: "".to_string(),
        category: "".to_string(),
        ttl_seconds: 0,
        status: SeasonalUsageSummaryStatus::Draft,
    }
}
