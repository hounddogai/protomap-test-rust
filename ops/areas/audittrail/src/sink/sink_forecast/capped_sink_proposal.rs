use super::*;

use acme_kit::text;

/// Where a capped sink proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSinkProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedSinkProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedSinkProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped sink proposal as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkProposal {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub ends_at: i64,
    pub namespace: String,
    pub pinned: bool,
    pub starts_at: i64,
    pub status: CappedSinkProposalStatus,
}

impl CappedSinkProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSinkProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSinkProposalStatus) -> bool {
        match self.status {
            CappedSinkProposalStatus::Created => {
                return next == CappedSinkProposalStatus::Retired;
            }
            CappedSinkProposalStatus::Validated => {
                match next {
                    CappedSinkProposalStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSinkProposalStatus::Published => {
                return next == CappedSinkProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSinkProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped sink proposal {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSinkProposal {
        CappedSinkProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            ends_at: self.ends_at,
            namespace: self.namespace.clone(),
            pinned: self.pinned,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped sink proposal in status created.
pub fn new_capped_sink_proposal(code: &str, label: &str) -> CappedSinkProposal {
    CappedSinkProposal {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        ends_at: 0,
        namespace: "".to_string(),
        pinned: false,
        starts_at: 0,
        status: CappedSinkProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_sink_proposal("code-853", "label-42");
        assert_eq!(item.status, CappedSinkProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_capped_sink_proposal("code-853", "label-42");
        if item.move_to(CappedSinkProposalStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSinkProposalStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_sink_proposal("code-853", "label-42");
        item.status = CappedSinkProposalStatus::Retired;
        assert!(
            item.move_to(CappedSinkProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
