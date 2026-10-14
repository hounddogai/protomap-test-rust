use super::*;

use acme_kit::text;

/// Where a manual token proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTokenProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualTokenProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualTokenProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual token proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenProposal {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub ceiling: f64,
    pub cluster: String,
    pub phone: String,
    pub rotation_days: i64,
    pub burst: i64,
    pub quantity: i64,
    pub version: i64,
    pub status: ManualTokenProposalStatus,
}

impl ManualTokenProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTokenProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTokenProposalStatus) -> bool {
        match self.status {
            ManualTokenProposalStatus::Created => {
                return next == ManualTokenProposalStatus::Published || next == ManualTokenProposalStatus::Retired;
            }
            ManualTokenProposalStatus::Validated => {
                return next == ManualTokenProposalStatus::Retired;
            }
            ManualTokenProposalStatus::Published => {
                match next {
                    ManualTokenProposalStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualTokenProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual token proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.version as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTokenProposal {
        ManualTokenProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            rotation_days: self.rotation_days,
            burst: self.burst,
            quantity: self.quantity,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual token proposal in status created.
pub fn new_manual_token_proposal(code: &str, name: &str) -> ManualTokenProposal {
    ManualTokenProposal {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        ceiling: 0.0,
        cluster: "".to_string(),
        phone: "".to_string(),
        rotation_days: 0,
        burst: 0,
        quantity: 0,
        version: 0,
        status: ManualTokenProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_token_proposal("code-778", "name-515");
        assert_eq!(item.status, ManualTokenProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_manual_token_proposal("code-778", "name-515");
        if item.move_to(ManualTokenProposalStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTokenProposalStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_token_proposal("code-778", "name-515");
        item.status = ManualTokenProposalStatus::Retired;
        assert!(
            item.move_to(ManualTokenProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
