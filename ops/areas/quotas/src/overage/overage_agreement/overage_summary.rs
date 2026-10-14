use super::*;

use acme_kit::text;

/// Where a overage summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageSummaryStatus {
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

impl std::fmt::Display for OverageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummary {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub margin: f64,
    pub ratio: f64,
    pub starts_at: i64,
    pub visible: bool,
    pub checksum: String,
    pub status: OverageSummaryStatus,
}

impl OverageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageSummaryStatus) -> bool {
        match self.status {
            OverageSummaryStatus::Open => {
                match next {
                    OverageSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageSummaryStatus::InReview => {
                return next == OverageSummaryStatus::Resolved;
            }
            OverageSummaryStatus::Resolved => {
                match next {
                    OverageSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageSummaryStatus::Reopened => {
                match next {
                    OverageSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: OverageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage summary {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageSummary {
        OverageSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            margin: self.margin,
            ratio: self.ratio,
            starts_at: self.starts_at,
            visible: self.visible,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a overage summary in status open.
pub fn new_overage_summary(code: &str, title: &str) -> OverageSummary {
    OverageSummary {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        margin: 0.0,
        ratio: 0.0,
        starts_at: 0,
        visible: false,
        checksum: "".to_string(),
        status: OverageSummaryStatus::Open,
    }
}
