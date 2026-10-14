use super::*;

use acme_kit::text;

/// Where a egress proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EgressProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EgressProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress proposal as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressProposal {
    pub code: String,
    pub name: String,
    pub title: String,
    pub floor_value: f64,
    pub audience: String,
    pub starts_at: i64,
    pub port: i64,
    pub ratio: f64,
    pub visible: bool,
    pub tenant_id: String,
    pub window_days: i64,
    pub status: EgressProposalStatus,
}

impl EgressProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressProposalStatus) -> bool {
        match self.status {
            EgressProposalStatus::Proposed => {
                return next == EgressProposalStatus::Withdrawn;
            }
            EgressProposalStatus::Approved => {
                return next == EgressProposalStatus::Withdrawn;
            }
            EgressProposalStatus::Rejected => {
                match next {
                    EgressProposalStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EgressProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress proposal {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.port as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressProposal {
        EgressProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            starts_at: self.starts_at,
            port: self.port,
            ratio: self.ratio,
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a egress proposal in status proposed.
pub fn new_egress_proposal(code: &str, name: &str) -> EgressProposal {
    EgressProposal {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        floor_value: 0.0,
        audience: "".to_string(),
        starts_at: 0,
        port: 0,
        ratio: 0.0,
        visible: false,
        tenant_id: "".to_string(),
        window_days: 0,
        status: EgressProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_egress_proposal("code-750", "name-283");
        assert_eq!(item.status, EgressProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_egress_proposal("code-750", "name-283");
        if item.move_to(EgressProposalStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressProposalStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_egress_proposal("code-750", "name-283");
        item.status = EgressProposalStatus::Withdrawn;
        assert!(
            item.move_to(EgressProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
