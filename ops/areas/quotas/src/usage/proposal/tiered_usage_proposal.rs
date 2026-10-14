use super::*;

use acme_kit::text;

/// Where a tiered usage proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredUsageProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredUsageProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TieredUsageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered usage proposal as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposal {
    pub code: String,
    pub label: String,
    pub category: String,
    pub key_id: String,
    pub tenant_id: String,
    pub amount: f64,
    pub status: TieredUsageProposalStatus,
}

impl TieredUsageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredUsageProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredUsageProposalStatus) -> bool {
        match self.status {
            TieredUsageProposalStatus::Idle => {
                match next {
                    TieredUsageProposalStatus::Ready | TieredUsageProposalStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredUsageProposalStatus::Warming => {
                return next == TieredUsageProposalStatus::Ready || next == TieredUsageProposalStatus::Stopped;
            }
            TieredUsageProposalStatus::Ready => {
                return next == TieredUsageProposalStatus::Draining || next == TieredUsageProposalStatus::Stopped;
            }
            TieredUsageProposalStatus::Draining => {
                return next == TieredUsageProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredUsageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered usage proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered usage proposal", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TieredUsageProposal {
        TieredUsageProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            tenant_id: self.tenant_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered usage proposal in status idle.
pub fn new_tiered_usage_proposal(code: &str, label: &str) -> TieredUsageProposal {
    TieredUsageProposal {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        key_id: "".to_string(),
        tenant_id: "".to_string(),
        amount: 0.0,
        status: TieredUsageProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_usage_proposal("code-137", "label-439");
        assert_eq!(item.status, TieredUsageProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tiered_usage_proposal("code-137", "label-439");
        if item.move_to(TieredUsageProposalStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredUsageProposalStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_usage_proposal("code-137", "label-439");
        item.status = TieredUsageProposalStatus::Stopped;
        assert!(
            item.move_to(TieredUsageProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
