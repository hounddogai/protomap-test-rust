use super::*;

use acme_kit::text;

/// Where a lenient plan quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPlanQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientPlanQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LenientPlanQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient plan quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanQuota {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub port: i64,
    pub unit: String,
    pub status: LenientPlanQuotaStatus,
}

impl LenientPlanQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPlanQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPlanQuotaStatus) -> bool {
        match self.status {
            LenientPlanQuotaStatus::New => {
                return next == LenientPlanQuotaStatus::Done || next == LenientPlanQuotaStatus::Failed;
            }
            LenientPlanQuotaStatus::Queued => {
                match next {
                    LenientPlanQuotaStatus::Running | LenientPlanQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanQuotaStatus::Running => {
                match next {
                    LenientPlanQuotaStatus::Done | LenientPlanQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanQuotaStatus::Done => {
                return next == LenientPlanQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPlanQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient plan quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> LenientPlanQuota {
        LenientPlanQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            port: self.port,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient plan quota in status new.
pub fn new_lenient_plan_quota(code: &str, name: &str) -> LenientPlanQuota {
    LenientPlanQuota {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        port: 0,
        unit: "".to_string(),
        status: LenientPlanQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_plan_quota("code-113", "name-176");
        assert_eq!(item.status, LenientPlanQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_lenient_plan_quota("code-113", "name-176");
        if item.move_to(LenientPlanQuotaStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPlanQuotaStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_plan_quota("code-113", "name-176");
        item.status = LenientPlanQuotaStatus::Failed;
        assert!(
            item.move_to(LenientPlanQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
