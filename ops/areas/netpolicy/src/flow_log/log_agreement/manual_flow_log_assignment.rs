use super::*;

use acme_kit::text;

/// Where a manual flow log assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFlowLogAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualFlowLogAssignmentStatus {
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

impl std::fmt::Display for ManualFlowLogAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual flow log assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignment {
    pub code: String,
    pub title: String,
    pub host: String,
    pub burst: i64,
    pub label: String,
    pub duration_minutes: i64,
    pub ceiling: f64,
    pub status: ManualFlowLogAssignmentStatus,
}

impl ManualFlowLogAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFlowLogAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFlowLogAssignmentStatus) -> bool {
        match self.status {
            ManualFlowLogAssignmentStatus::New => {
                return next == ManualFlowLogAssignmentStatus::Queued || next == ManualFlowLogAssignmentStatus::Failed;
            }
            ManualFlowLogAssignmentStatus::Queued => {
                match next {
                    ManualFlowLogAssignmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFlowLogAssignmentStatus::Running => {
                return next == ManualFlowLogAssignmentStatus::Failed;
            }
            ManualFlowLogAssignmentStatus::Done => {
                match next {
                    ManualFlowLogAssignmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualFlowLogAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual flow log assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.burst as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFlowLogAssignment {
        ManualFlowLogAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            burst: self.burst,
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual flow log assignment in status new.
pub fn new_manual_flow_log_assignment(code: &str, title: &str) -> ManualFlowLogAssignment {
    ManualFlowLogAssignment {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        burst: 0,
        label: "".to_string(),
        duration_minutes: 0,
        ceiling: 0.0,
        status: ManualFlowLogAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_flow_log_assignment("code-4", "title-196");
        assert_eq!(item.status, ManualFlowLogAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_flow_log_assignment("code-4", "title-196");
        if item.move_to(ManualFlowLogAssignmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFlowLogAssignmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_flow_log_assignment("code-4", "title-196");
        item.status = ManualFlowLogAssignmentStatus::Failed;
        assert!(
            item.move_to(ManualFlowLogAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
