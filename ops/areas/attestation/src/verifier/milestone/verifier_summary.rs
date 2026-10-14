use super::*;

use acme_kit::text;

/// Where a verifier summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl VerifierSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for VerifierSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSummary {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub name: String,
    pub version: i64,
    pub status: VerifierSummaryStatus,
}

impl VerifierSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierSummaryStatus) -> bool {
        match self.status {
            VerifierSummaryStatus::Proposed => {
                match next {
                    VerifierSummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierSummaryStatus::Approved => {
                return next == VerifierSummaryStatus::Rejected || next == VerifierSummaryStatus::Withdrawn;
            }
            VerifierSummaryStatus::Rejected => {
                return next == VerifierSummaryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier summary {} cannot move from {} to {}",
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.priority as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierSummary {
        VerifierSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            name: self.name.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a verifier summary in status proposed.
pub fn new_verifier_summary(code: &str, title: &str) -> VerifierSummary {
    VerifierSummary {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        name: "".to_string(),
        version: 0,
        status: VerifierSummaryStatus::Proposed,
    }
}
