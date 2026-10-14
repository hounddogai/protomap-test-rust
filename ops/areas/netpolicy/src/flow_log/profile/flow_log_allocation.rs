use super::*;

use acme_kit::text;

/// Where a flow log allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAllocation {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub priority: i64,
    pub country: String,
    pub reviewer: String,
    pub cipher: String,
    pub updated_at: i64,
    pub status: FlowLogAllocationStatus,
}

impl FlowLogAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogAllocationStatus) -> bool {
        match self.status {
            FlowLogAllocationStatus::Proposed => {
                return next == FlowLogAllocationStatus::Approved;
            }
            FlowLogAllocationStatus::Approved => {
                return next == FlowLogAllocationStatus::Withdrawn;
            }
            FlowLogAllocationStatus::Rejected => {
                return next == FlowLogAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.updated_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogAllocation {
        FlowLogAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            priority: self.priority,
            country: self.country.clone(),
            reviewer: self.reviewer.clone(),
            cipher: self.cipher.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a flow log allocation in status proposed.
pub fn new_flow_log_allocation(code: &str, title: &str) -> FlowLogAllocation {
    FlowLogAllocation {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        priority: 0,
        country: "".to_string(),
        reviewer: "".to_string(),
        cipher: "".to_string(),
        updated_at: 0,
        status: FlowLogAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_allocation("code-320", "title-430");
        assert_eq!(item.status, FlowLogAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_flow_log_allocation("code-320", "title-430");
        if item.move_to(FlowLogAllocationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogAllocationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_allocation("code-320", "title-430");
        item.status = FlowLogAllocationStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
