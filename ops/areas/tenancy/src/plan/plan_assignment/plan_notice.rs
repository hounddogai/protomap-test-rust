use super::*;

use acme_kit::text;

/// Where a plan notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanNoticeStatus {
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

impl std::fmt::Display for PlanNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanNotice {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub name: String,
    pub key_id: String,
    pub status: PlanNoticeStatus,
}

impl PlanNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanNoticeStatus) -> bool {
        match self.status {
            PlanNoticeStatus::Planned => {
                match next {
                    PlanNoticeStatus::Scheduled | PlanNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanNoticeStatus::Scheduled => {
                return next == PlanNoticeStatus::InProgress || next == PlanNoticeStatus::Completed;
            }
            PlanNoticeStatus::InProgress => {
                return next == PlanNoticeStatus::Completed;
            }
            PlanNoticeStatus::Completed => {
                match next {
                    PlanNoticeStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PlanNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> PlanNotice {
        PlanNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan notice in status planned.
pub fn new_plan_notice(code: &str, title: &str) -> PlanNotice {
    PlanNotice {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        name: "".to_string(),
        key_id: "".to_string(),
        status: PlanNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_notice("code-548", "title-510");
        assert_eq!(item.status, PlanNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_plan_notice("code-548", "title-510");
        if item.move_to(PlanNoticeStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanNoticeStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_notice("code-548", "title-510");
        item.status = PlanNoticeStatus::Cancelled;
        assert!(
            item.move_to(PlanNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
