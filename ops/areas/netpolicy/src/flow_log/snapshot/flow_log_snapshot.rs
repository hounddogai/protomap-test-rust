use super::*;

use acme_kit::text;

/// Where a flow log snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshot {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub segment: String,
    pub name: String,
    pub approved: bool,
    pub fingerprint: String,
    pub status: FlowLogSnapshotStatus,
}

impl FlowLogSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogSnapshotStatus) -> bool {
        match self.status {
            FlowLogSnapshotStatus::Proposed => {
                match next {
                    FlowLogSnapshotStatus::Rejected | FlowLogSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogSnapshotStatus::Approved => {
                return next == FlowLogSnapshotStatus::Rejected || next == FlowLogSnapshotStatus::Withdrawn;
            }
            FlowLogSnapshotStatus::Rejected => {
                match next {
                    FlowLogSnapshotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FlowLogSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> FlowLogSnapshot {
        FlowLogSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            segment: self.segment.clone(),
            name: self.name.clone(),
            approved: self.approved,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a flow log snapshot in status proposed.
pub fn new_flow_log_snapshot(code: &str, title: &str) -> FlowLogSnapshot {
    FlowLogSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        segment: "".to_string(),
        name: "".to_string(),
        approved: false,
        fingerprint: "".to_string(),
        status: FlowLogSnapshotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_snapshot("code-687", "title-309");
        assert_eq!(item.status, FlowLogSnapshotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_flow_log_snapshot("code-687", "title-309");
        if item.move_to(FlowLogSnapshotStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogSnapshotStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_snapshot("code-687", "title-309");
        item.status = FlowLogSnapshotStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogSnapshotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
