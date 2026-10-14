use super::*;

use acme_kit::text;

/// Where a default failover cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFailoverCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultFailoverCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultFailoverCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default failover cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCycle {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub cost: f64,
    pub tenant_id: String,
    pub pinned: bool,
    pub amount: f64,
    pub discount: f64,
    pub ceiling: f64,
    pub currency: String,
    pub name: String,
    pub status: DefaultFailoverCycleStatus,
}

impl DefaultFailoverCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFailoverCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFailoverCycleStatus) -> bool {
        match self.status {
            DefaultFailoverCycleStatus::Proposed => {
                match next {
                    DefaultFailoverCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFailoverCycleStatus::Approved => {
                match next {
                    DefaultFailoverCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFailoverCycleStatus::Rejected => {
                match next {
                    DefaultFailoverCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultFailoverCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default failover cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.cost - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultFailoverCycle {
        DefaultFailoverCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            amount: self.amount,
            discount: self.discount,
            ceiling: self.ceiling,
            currency: self.currency.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default failover cycle in status proposed.
pub fn new_default_failover_cycle(code: &str, label: &str) -> DefaultFailoverCycle {
    DefaultFailoverCycle {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        cost: 0.0,
        tenant_id: "".to_string(),
        pinned: false,
        amount: 0.0,
        discount: 0.0,
        ceiling: 0.0,
        currency: "".to_string(),
        name: "".to_string(),
        status: DefaultFailoverCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_failover_cycle("code-634", "label-352");
        assert_eq!(item.status, DefaultFailoverCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_failover_cycle("code-634", "label-352");
        if item.move_to(DefaultFailoverCycleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFailoverCycleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_failover_cycle("code-634", "label-352");
        item.status = DefaultFailoverCycleStatus::Withdrawn;
        assert!(
            item.move_to(DefaultFailoverCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
