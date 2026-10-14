use super::*;

use acme_kit::text;

/// Where a failover plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FailoverPlanStatus {
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

impl std::fmt::Display for FailoverPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlan {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub contact_email: String,
    pub ceiling: f64,
    pub status: FailoverPlanStatus,
}

impl FailoverPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverPlanStatus) -> bool {
        match self.status {
            FailoverPlanStatus::Planned => {
                return next == FailoverPlanStatus::Completed || next == FailoverPlanStatus::Cancelled;
            }
            FailoverPlanStatus::Scheduled => {
                match next {
                    FailoverPlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverPlanStatus::InProgress => {
                return next == FailoverPlanStatus::Cancelled;
            }
            FailoverPlanStatus::Completed => {
                return next == FailoverPlanStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "failover plan", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.discount * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverPlan {
        FailoverPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a failover plan in status planned.
pub fn new_failover_plan(code: &str, name: &str) -> FailoverPlan {
    FailoverPlan {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        contact_email: "".to_string(),
        ceiling: 0.0,
        status: FailoverPlanStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_failover_plan("code-204", "name-629");
        assert_eq!(item.status, FailoverPlanStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_failover_plan("code-204", "name-629");
        if item.move_to(FailoverPlanStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverPlanStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_failover_plan("code-204", "name-629");
        item.status = FailoverPlanStatus::Cancelled;
        assert!(
            item.move_to(FailoverPlanStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
