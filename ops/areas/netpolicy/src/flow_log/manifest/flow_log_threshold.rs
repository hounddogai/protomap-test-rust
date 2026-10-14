use super::*;

use acme_kit::text;

/// Where a flow log threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThreshold {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub reviewer: String,
    pub key_id: String,
    pub ttl_seconds: i64,
    pub locale: String,
    pub status: FlowLogThresholdStatus,
}

impl FlowLogThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogThresholdStatus) -> bool {
        match self.status {
            FlowLogThresholdStatus::Proposed => {
                match next {
                    FlowLogThresholdStatus::Rejected | FlowLogThresholdStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogThresholdStatus::Approved => {
                return next == FlowLogThresholdStatus::Withdrawn;
            }
            FlowLogThresholdStatus::Rejected => {
                match next {
                    FlowLogThresholdStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FlowLogThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> FlowLogThreshold {
        FlowLogThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            reviewer: self.reviewer.clone(),
            key_id: self.key_id.clone(),
            ttl_seconds: self.ttl_seconds,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a flow log threshold in status proposed.
pub fn new_flow_log_threshold(code: &str, title: &str) -> FlowLogThreshold {
    FlowLogThreshold {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        reviewer: "".to_string(),
        key_id: "".to_string(),
        ttl_seconds: 0,
        locale: "".to_string(),
        status: FlowLogThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_threshold("code-963", "title-3");
        assert_eq!(item.status, FlowLogThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_flow_log_threshold("code-963", "title-3");
        if item.move_to(FlowLogThresholdStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogThresholdStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_threshold("code-963", "title-3");
        item.status = FlowLogThresholdStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
