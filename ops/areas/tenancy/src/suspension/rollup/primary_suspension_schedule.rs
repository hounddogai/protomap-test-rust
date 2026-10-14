use super::*;

use acme_kit::text;

/// Where a primary suspension schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySuspensionScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimarySuspensionScheduleStatus {
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

impl std::fmt::Display for PrimarySuspensionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary suspension schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionSchedule {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub archived: bool,
    pub limit: i64,
    pub threshold: f64,
    pub summary: String,
    pub tenant_id: String,
    pub status: PrimarySuspensionScheduleStatus,
}

impl PrimarySuspensionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySuspensionScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySuspensionScheduleStatus) -> bool {
        match self.status {
            PrimarySuspensionScheduleStatus::New => {
                return next == PrimarySuspensionScheduleStatus::Running
                    || next == PrimarySuspensionScheduleStatus::Done;
            }
            PrimarySuspensionScheduleStatus::Queued => {
                match next {
                    PrimarySuspensionScheduleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySuspensionScheduleStatus::Running => {
                match next {
                    PrimarySuspensionScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySuspensionScheduleStatus::Done => {
                match next {
                    PrimarySuspensionScheduleStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimarySuspensionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary suspension schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.limit as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySuspensionSchedule {
        PrimarySuspensionSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            archived: self.archived,
            limit: self.limit,
            threshold: self.threshold,
            summary: self.summary.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary suspension schedule in status new.
pub fn new_primary_suspension_schedule(code: &str, name: &str) -> PrimarySuspensionSchedule {
    PrimarySuspensionSchedule {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        archived: false,
        limit: 0,
        threshold: 0.0,
        summary: "".to_string(),
        tenant_id: "".to_string(),
        status: PrimarySuspensionScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_suspension_schedule("code-724", "name-18");
        assert_eq!(item.status, PrimarySuspensionScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_suspension_schedule("code-724", "name-18");
        if item.move_to(PrimarySuspensionScheduleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySuspensionScheduleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_suspension_schedule("code-724", "name-18");
        item.status = PrimarySuspensionScheduleStatus::Failed;
        assert!(
            item.move_to(PrimarySuspensionScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
