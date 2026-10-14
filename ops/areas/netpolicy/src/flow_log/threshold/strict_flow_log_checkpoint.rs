use super::*;

use acme_kit::text;

/// Where a strict flow log checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFlowLogCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictFlowLogCheckpointStatus {
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

impl std::fmt::Display for StrictFlowLogCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict flow log checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogCheckpoint {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub category: String,
    pub discount: f64,
    pub status: StrictFlowLogCheckpointStatus,
}

impl StrictFlowLogCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFlowLogCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFlowLogCheckpointStatus) -> bool {
        match self.status {
            StrictFlowLogCheckpointStatus::Draft => {
                return next == StrictFlowLogCheckpointStatus::Closed;
            }
            StrictFlowLogCheckpointStatus::Pending => {
                match next {
                    StrictFlowLogCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFlowLogCheckpointStatus::Active => {
                return next == StrictFlowLogCheckpointStatus::Suspended
                    || next == StrictFlowLogCheckpointStatus::Closed;
            }
            StrictFlowLogCheckpointStatus::Suspended => {
                match next {
                    StrictFlowLogCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictFlowLogCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict flow log checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict flow log checkpoint", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.discount - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFlowLogCheckpoint {
        StrictFlowLogCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            category: self.category.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict flow log checkpoint in status draft.
pub fn new_strict_flow_log_checkpoint(code: &str, title: &str) -> StrictFlowLogCheckpoint {
    StrictFlowLogCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        category: "".to_string(),
        discount: 0.0,
        status: StrictFlowLogCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_flow_log_checkpoint("code-270", "title-178");
        assert_eq!(item.status, StrictFlowLogCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_strict_flow_log_checkpoint("code-270", "title-178");
        if item.move_to(StrictFlowLogCheckpointStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFlowLogCheckpointStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_flow_log_checkpoint("code-270", "title-178");
        item.status = StrictFlowLogCheckpointStatus::Closed;
        assert!(
            item.move_to(StrictFlowLogCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
