use super::*;

use acme_kit::text;

/// Where a tenant proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TenantProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TenantProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant proposal as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposal {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub unit: String,
    pub ends_at: i64,
    pub archived: bool,
    pub external_ref: String,
    pub discount: f64,
    pub title: String,
    pub tenant_id: String,
    pub status: TenantProposalStatus,
}

impl TenantProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantProposalStatus) -> bool {
        match self.status {
            TenantProposalStatus::Planned => {
                match next {
                    TenantProposalStatus::Scheduled | TenantProposalStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantProposalStatus::Scheduled => {
                match next {
                    TenantProposalStatus::InProgress | TenantProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantProposalStatus::InProgress => {
                match next {
                    TenantProposalStatus::Completed | TenantProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantProposalStatus::Completed => {
                return next == TenantProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantProposal {
        TenantProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            unit: self.unit.clone(),
            ends_at: self.ends_at,
            archived: self.archived,
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tenant proposal in status planned.
pub fn new_tenant_proposal(code: &str, label: &str) -> TenantProposal {
    TenantProposal {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        unit: "".to_string(),
        ends_at: 0,
        archived: false,
        external_ref: "".to_string(),
        discount: 0.0,
        title: "".to_string(),
        tenant_id: "".to_string(),
        status: TenantProposalStatus::Planned,
    }
}
