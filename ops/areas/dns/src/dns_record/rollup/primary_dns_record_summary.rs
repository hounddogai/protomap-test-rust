use super::*;

use acme_kit::text;

/// Where a primary dns record summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDnsRecordSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryDnsRecordSummaryStatus {
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

impl std::fmt::Display for PrimaryDnsRecordSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary dns record summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordSummary {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub weight: f64,
    pub timezone: String,
    pub budget: f64,
    pub duration_minutes: i64,
    pub status: PrimaryDnsRecordSummaryStatus,
}

impl PrimaryDnsRecordSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDnsRecordSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDnsRecordSummaryStatus) -> bool {
        match self.status {
            PrimaryDnsRecordSummaryStatus::Open => {
                return next == PrimaryDnsRecordSummaryStatus::Archived;
            }
            PrimaryDnsRecordSummaryStatus::InReview => {
                return next == PrimaryDnsRecordSummaryStatus::Reopened
                    || next == PrimaryDnsRecordSummaryStatus::Archived;
            }
            PrimaryDnsRecordSummaryStatus::Resolved => {
                match next {
                    PrimaryDnsRecordSummaryStatus::Reopened | PrimaryDnsRecordSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDnsRecordSummaryStatus::Reopened => {
                return next == PrimaryDnsRecordSummaryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDnsRecordSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary dns record summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.budget + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDnsRecordSummary {
        PrimaryDnsRecordSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            weight: self.weight,
            timezone: self.timezone.clone(),
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary dns record summary in status open.
pub fn new_primary_dns_record_summary(code: &str, label: &str) -> PrimaryDnsRecordSummary {
    PrimaryDnsRecordSummary {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        weight: 0.0,
        timezone: "".to_string(),
        budget: 0.0,
        duration_minutes: 0,
        status: PrimaryDnsRecordSummaryStatus::Open,
    }
}
