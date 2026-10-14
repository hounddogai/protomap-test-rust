use super::*;

use acme_kit::text;

/// Where a flow log checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FlowLogCheckpointStatus {
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

impl std::fmt::Display for FlowLogCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpoint {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub unit: String,
    pub quantity: i64,
    pub status: FlowLogCheckpointStatus,
}

impl FlowLogCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogCheckpointStatus) -> bool {
        match self.status {
            FlowLogCheckpointStatus::New => {
                match next {
                    FlowLogCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogCheckpointStatus::Queued => {
                match next {
                    FlowLogCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogCheckpointStatus::Running => {
                return next == FlowLogCheckpointStatus::Done || next == FlowLogCheckpointStatus::Failed;
            }
            FlowLogCheckpointStatus::Done => {
                match next {
                    FlowLogCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: FlowLogCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quantity as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogCheckpoint {
        FlowLogCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            unit: self.unit.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a flow log checkpoint in status new.
pub fn new_flow_log_checkpoint(code: &str, name: &str) -> FlowLogCheckpoint {
    FlowLogCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        unit: "".to_string(),
        quantity: 0,
        status: FlowLogCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_flow_log_checkpoint("code-209", "name-731");
        assert_eq!(item.status, FlowLogCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_flow_log_checkpoint("code-209", "name-731");
        if item.move_to(FlowLogCheckpointStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogCheckpointStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_flow_log_checkpoint("code-209", "name-731");
        item.status = FlowLogCheckpointStatus::Failed;
        assert!(
            item.move_to(FlowLogCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
