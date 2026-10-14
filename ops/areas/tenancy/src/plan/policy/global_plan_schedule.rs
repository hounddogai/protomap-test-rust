use super::*;

use acme_kit::text;

/// Where a global plan schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPlanScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalPlanScheduleStatus {
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

impl std::fmt::Display for GlobalPlanScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global plan schedule as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanSchedule {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub expires_at: i64,
    pub tenant_id: String,
    pub status: GlobalPlanScheduleStatus,
}

impl GlobalPlanSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPlanScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPlanScheduleStatus) -> bool {
        match self.status {
            GlobalPlanScheduleStatus::Draft => {
                match next {
                    GlobalPlanScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanScheduleStatus::Pending => {
                return next == GlobalPlanScheduleStatus::Suspended || next == GlobalPlanScheduleStatus::Closed;
            }
            GlobalPlanScheduleStatus::Active => {
                return next == GlobalPlanScheduleStatus::Suspended;
            }
            GlobalPlanScheduleStatus::Suspended => {
                return next == GlobalPlanScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPlanScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global plan schedule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.expires_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPlanSchedule {
        GlobalPlanSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global plan schedule in status draft.
pub fn new_global_plan_schedule(code: &str, label: &str) -> GlobalPlanSchedule {
    GlobalPlanSchedule {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        starts_at: 0,
        expires_at: 0,
        tenant_id: "".to_string(),
        status: GlobalPlanScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_plan_schedule("code-108", "label-927");
        assert_eq!(item.status, GlobalPlanScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_global_plan_schedule("code-108", "label-927");
        if item.move_to(GlobalPlanScheduleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPlanScheduleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_plan_schedule("code-108", "label-927");
        item.status = GlobalPlanScheduleStatus::Closed;
        assert!(
            item.move_to(GlobalPlanScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
