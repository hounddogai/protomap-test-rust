use super::*;

use acme_kit::text;

/// Where a regional health check proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalHealthCheckProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalHealthCheckProposalStatus {
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

impl std::fmt::Display for RegionalHealthCheckProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional health check proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalHealthCheckProposal {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub ttl_seconds: i64,
    pub checksum: String,
    pub timezone: String,
    pub sequence: i64,
    pub status: RegionalHealthCheckProposalStatus,
}

impl RegionalHealthCheckProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalHealthCheckProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalHealthCheckProposalStatus) -> bool {
        match self.status {
            RegionalHealthCheckProposalStatus::Open => {
                match next {
                    RegionalHealthCheckProposalStatus::Resolved | RegionalHealthCheckProposalStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalHealthCheckProposalStatus::InReview => {
                return next == RegionalHealthCheckProposalStatus::Reopened
                    || next == RegionalHealthCheckProposalStatus::Archived;
            }
            RegionalHealthCheckProposalStatus::Resolved => {
                return next == RegionalHealthCheckProposalStatus::Reopened
                    || next == RegionalHealthCheckProposalStatus::Archived;
            }
            RegionalHealthCheckProposalStatus::Reopened => {
                return next == RegionalHealthCheckProposalStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalHealthCheckProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional health check proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalHealthCheckProposal {
        RegionalHealthCheckProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            ttl_seconds: self.ttl_seconds,
            checksum: self.checksum.clone(),
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional health check proposal in status open.
pub fn new_regional_health_check_proposal(code: &str, label: &str) -> RegionalHealthCheckProposal {
    RegionalHealthCheckProposal {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        ttl_seconds: 0,
        checksum: "".to_string(),
        timezone: "".to_string(),
        sequence: 0,
        status: RegionalHealthCheckProposalStatus::Open,
    }
}
