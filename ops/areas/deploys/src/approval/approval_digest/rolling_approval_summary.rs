use super::*;

use acme_kit::text;

/// Where a rolling approval summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingApprovalSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingApprovalSummaryStatus {
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

impl std::fmt::Display for RollingApprovalSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling approval summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummary {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub unit: String,
    pub checksum: String,
    pub channel: String,
    pub summary: String,
    pub sequence: i64,
    pub key_id: String,
    pub updated_at: i64,
    pub status: RollingApprovalSummaryStatus,
}

impl RollingApprovalSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingApprovalSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingApprovalSummaryStatus) -> bool {
        match self.status {
            RollingApprovalSummaryStatus::Open => {
                match next {
                    RollingApprovalSummaryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingApprovalSummaryStatus::InReview => {
                return next == RollingApprovalSummaryStatus::Resolved
                    || next == RollingApprovalSummaryStatus::Archived;
            }
            RollingApprovalSummaryStatus::Resolved => {
                match next {
                    RollingApprovalSummaryStatus::Reopened | RollingApprovalSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingApprovalSummaryStatus::Reopened => {
                match next {
                    RollingApprovalSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingApprovalSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling approval summary {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.sequence as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingApprovalSummary {
        RollingApprovalSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            summary: self.summary.clone(),
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling approval summary in status open.
pub fn new_rolling_approval_summary(code: &str, name: &str) -> RollingApprovalSummary {
    RollingApprovalSummary {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        unit: "".to_string(),
        checksum: "".to_string(),
        channel: "".to_string(),
        summary: "".to_string(),
        sequence: 0,
        key_id: "".to_string(),
        updated_at: 0,
        status: RollingApprovalSummaryStatus::Open,
    }
}
