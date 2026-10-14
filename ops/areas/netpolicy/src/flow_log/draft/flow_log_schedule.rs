use super::*;

use acme_kit::text;

/// Where a flow log schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSchedule {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub cipher: String,
    pub source: String,
    pub status: FlowLogScheduleStatus,
}

impl FlowLogSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogScheduleStatus) -> bool {
        match self.status {
            FlowLogScheduleStatus::Proposed => {
                return next == FlowLogScheduleStatus::Rejected;
            }
            FlowLogScheduleStatus::Approved => {
                match next {
                    FlowLogScheduleStatus::Rejected | FlowLogScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogScheduleStatus::Rejected => {
                return next == FlowLogScheduleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> FlowLogSchedule {
        FlowLogSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            cipher: self.cipher.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a flow log schedule in status proposed.
pub fn new_flow_log_schedule(code: &str, name: &str) -> FlowLogSchedule {
    FlowLogSchedule {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        cipher: "".to_string(),
        source: "".to_string(),
        status: FlowLogScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_schedule("code-847", "name-834");
        assert_eq!(item.status, FlowLogScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_flow_log_schedule("code-847", "name-834");
        if item.move_to(FlowLogScheduleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogScheduleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_schedule("code-847", "name-834");
        item.status = FlowLogScheduleStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
