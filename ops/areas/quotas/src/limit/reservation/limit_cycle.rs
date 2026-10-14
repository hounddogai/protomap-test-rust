use super::*;

use acme_kit::text;

/// Where a limit cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LimitCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LimitCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitCycle {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub discount: f64,
    pub reviewer: String,
    pub status: LimitCycleStatus,
}

impl LimitCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitCycleStatus) -> bool {
        match self.status {
            LimitCycleStatus::Proposed => {
                return next == LimitCycleStatus::Approved;
            }
            LimitCycleStatus::Approved => {
                return next == LimitCycleStatus::Withdrawn;
            }
            LimitCycleStatus::Rejected => {
                return next == LimitCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> LimitCycle {
        LimitCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit cycle in status proposed.
pub fn new_limit_cycle(code: &str, label: &str) -> LimitCycle {
    LimitCycle {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        discount: 0.0,
        reviewer: "".to_string(),
        status: LimitCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_limit_cycle("code-356", "label-626");
        assert_eq!(item.status, LimitCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_limit_cycle("code-356", "label-626");
        if item.move_to(LimitCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_limit_cycle("code-356", "label-626");
        item.status = LimitCycleStatus::Withdrawn;
        assert!(
            item.move_to(LimitCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
