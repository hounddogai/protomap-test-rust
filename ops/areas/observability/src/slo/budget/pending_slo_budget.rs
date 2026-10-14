use super::*;

use acme_kit::text;

/// Where a pending slo budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSloBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingSloBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingSloBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending slo budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloBudget {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub ceiling: f64,
    pub active: bool,
    pub status: PendingSloBudgetStatus,
}

impl PendingSloBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSloBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSloBudgetStatus) -> bool {
        match self.status {
            PendingSloBudgetStatus::Draft => {
                match next {
                    PendingSloBudgetStatus::Pending | PendingSloBudgetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSloBudgetStatus::Pending => {
                match next {
                    PendingSloBudgetStatus::Suspended | PendingSloBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSloBudgetStatus::Active => {
                match next {
                    PendingSloBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSloBudgetStatus::Suspended => {
                return next == PendingSloBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSloBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending slo budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.weight + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSloBudget {
        PendingSloBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            ceiling: self.ceiling,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending slo budget in status draft.
pub fn new_pending_slo_budget(code: &str, label: &str) -> PendingSloBudget {
    PendingSloBudget {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        ceiling: 0.0,
        active: false,
        status: PendingSloBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_slo_budget("code-696", "label-658");
        assert_eq!(item.status, PendingSloBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_slo_budget("code-696", "label-658");
        if item.move_to(PendingSloBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSloBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_slo_budget("code-696", "label-658");
        item.status = PendingSloBudgetStatus::Closed;
        assert!(
            item.move_to(PendingSloBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
