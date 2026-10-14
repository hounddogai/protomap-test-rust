use super::*;

use acme_kit::text;

/// Where a flow log batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FlowLogBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FlowLogBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatch {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub country: String,
    pub ends_at: i64,
    pub status: FlowLogBatchStatus,
}

impl FlowLogBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogBatchStatus) -> bool {
        match self.status {
            FlowLogBatchStatus::Open => {
                return next == FlowLogBatchStatus::Resolved;
            }
            FlowLogBatchStatus::InReview => {
                return next == FlowLogBatchStatus::Reopened || next == FlowLogBatchStatus::Archived;
            }
            FlowLogBatchStatus::Resolved => {
                match next {
                    FlowLogBatchStatus::Reopened | FlowLogBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBatchStatus::Reopened => {
                match next {
                    FlowLogBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: FlowLogBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> FlowLogBatch {
        FlowLogBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            country: self.country.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a flow log batch in status open.
pub fn new_flow_log_batch(code: &str, label: &str) -> FlowLogBatch {
    FlowLogBatch {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        country: "".to_string(),
        ends_at: 0,
        status: FlowLogBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_flow_log_batch("code-557", "label-401");
        assert_eq!(item.status, FlowLogBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_flow_log_batch("code-557", "label-401");
        if item.move_to(FlowLogBatchStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogBatchStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_flow_log_batch("code-557", "label-401");
        item.status = FlowLogBatchStatus::Archived;
        assert!(item.move_to(FlowLogBatchStatus::Open).is_err(), "expected move to fail");
    }
}
