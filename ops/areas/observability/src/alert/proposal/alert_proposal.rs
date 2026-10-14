use super::*;

use acme_kit::text;

/// Where a alert proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertProposal {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub title: String,
    pub threshold: f64,
    pub rate: f64,
    pub updated_at: i64,
    pub status: AlertProposalStatus,
}

impl AlertProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertProposalStatus) -> bool {
        match self.status {
            AlertProposalStatus::Created => {
                return next == AlertProposalStatus::Validated || next == AlertProposalStatus::Published;
            }
            AlertProposalStatus::Validated => {
                match next {
                    AlertProposalStatus::Published | AlertProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertProposalStatus::Published => {
                return next == AlertProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert proposal {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert proposal", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        (self.rate * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertProposal {
        AlertProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            rate: self.rate,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert proposal in status created.
pub fn new_alert_proposal(code: &str, label: &str) -> AlertProposal {
    AlertProposal {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        title: "".to_string(),
        threshold: 0.0,
        rate: 0.0,
        updated_at: 0,
        status: AlertProposalStatus::Created,
    }
}
