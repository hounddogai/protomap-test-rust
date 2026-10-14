use super::*;

use acme_kit::text;

/// Where a global usage plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalUsagePlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalUsagePlanStatus {
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

impl std::fmt::Display for GlobalUsagePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global usage plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsagePlan {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub cost: f64,
    pub margin: f64,
    pub title: String,
    pub status: GlobalUsagePlanStatus,
}

impl GlobalUsagePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalUsagePlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalUsagePlanStatus) -> bool {
        match self.status {
            GlobalUsagePlanStatus::Planned => {
                match next {
                    GlobalUsagePlanStatus::Scheduled | GlobalUsagePlanStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalUsagePlanStatus::Scheduled => {
                match next {
                    GlobalUsagePlanStatus::Completed | GlobalUsagePlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalUsagePlanStatus::InProgress => {
                return next == GlobalUsagePlanStatus::Completed || next == GlobalUsagePlanStatus::Cancelled;
            }
            GlobalUsagePlanStatus::Completed => {
                match next {
                    GlobalUsagePlanStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalUsagePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global usage plan {} cannot move from {} to {}",
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
        (self.ceiling * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalUsagePlan {
        GlobalUsagePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            cost: self.cost,
            margin: self.margin,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global usage plan in status planned.
pub fn new_global_usage_plan(code: &str, label: &str) -> GlobalUsagePlan {
    GlobalUsagePlan {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        cost: 0.0,
        margin: 0.0,
        title: "".to_string(),
        status: GlobalUsagePlanStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_usage_plan("code-380", "label-346");
        assert_eq!(item.status, GlobalUsagePlanStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_usage_plan("code-380", "label-346");
        if item.move_to(GlobalUsagePlanStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalUsagePlanStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_usage_plan("code-380", "label-346");
        item.status = GlobalUsagePlanStatus::Cancelled;
        assert!(
            item.move_to(GlobalUsagePlanStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
