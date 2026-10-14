use super::*;

use acme_kit::text;

/// Where a default tenant proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTenantProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultTenantProposalStatus {
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

impl std::fmt::Display for DefaultTenantProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default tenant proposal as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposal {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub batch_size: i64,
    pub rate: f64,
    pub issuer: String,
    pub host: String,
    pub pinned: bool,
    pub window_days: i64,
    pub timezone: String,
    pub status: DefaultTenantProposalStatus,
}

impl DefaultTenantProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTenantProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTenantProposalStatus) -> bool {
        match self.status {
            DefaultTenantProposalStatus::Planned => {
                match next {
                    DefaultTenantProposalStatus::Completed | DefaultTenantProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTenantProposalStatus::Scheduled => {
                return next == DefaultTenantProposalStatus::InProgress
                    || next == DefaultTenantProposalStatus::Cancelled;
            }
            DefaultTenantProposalStatus::InProgress => {
                return next == DefaultTenantProposalStatus::Completed
                    || next == DefaultTenantProposalStatus::Cancelled;
            }
            DefaultTenantProposalStatus::Completed => {
                return next == DefaultTenantProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultTenantProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default tenant proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default tenant proposal", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rate + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTenantProposal {
        DefaultTenantProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            issuer: self.issuer.clone(),
            host: self.host.clone(),
            pinned: self.pinned,
            window_days: self.window_days,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default tenant proposal in status planned.
pub fn new_default_tenant_proposal(code: &str, name: &str) -> DefaultTenantProposal {
    DefaultTenantProposal {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        batch_size: 0,
        rate: 0.0,
        issuer: "".to_string(),
        host: "".to_string(),
        pinned: false,
        window_days: 0,
        timezone: "".to_string(),
        status: DefaultTenantProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_tenant_proposal("code-289", "name-290");
        assert_eq!(item.status, DefaultTenantProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_default_tenant_proposal("code-289", "name-290");
        if item.move_to(DefaultTenantProposalStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTenantProposalStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_tenant_proposal("code-289", "name-290");
        item.status = DefaultTenantProposalStatus::Cancelled;
        assert!(
            item.move_to(DefaultTenantProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
