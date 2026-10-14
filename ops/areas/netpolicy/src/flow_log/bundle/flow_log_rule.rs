use super::*;

use acme_kit::text;

/// Where a flow log rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRule {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub duration_minutes: i64,
    pub issuer: String,
    pub weight: f64,
    pub threshold: f64,
    pub priority: i64,
    pub created_at: i64,
    pub status: FlowLogRuleStatus,
}

impl FlowLogRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogRuleStatus) -> bool {
        match self.status {
            FlowLogRuleStatus::Proposed => {
                return next == FlowLogRuleStatus::Approved || next == FlowLogRuleStatus::Withdrawn;
            }
            FlowLogRuleStatus::Approved => {
                match next {
                    FlowLogRuleStatus::Rejected | FlowLogRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogRuleStatus::Rejected => {
                return next == FlowLogRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogRule {
        FlowLogRule {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            duration_minutes: self.duration_minutes,
            issuer: self.issuer.clone(),
            weight: self.weight,
            threshold: self.threshold,
            priority: self.priority,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a flow log rule in status proposed.
pub fn new_flow_log_rule(code: &str, name: &str) -> FlowLogRule {
    FlowLogRule {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        duration_minutes: 0,
        issuer: "".to_string(),
        weight: 0.0,
        threshold: 0.0,
        priority: 0,
        created_at: 0,
        status: FlowLogRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_rule("code-208", "name-362");
        assert_eq!(item.status, FlowLogRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_flow_log_rule("code-208", "name-362");
        if item.move_to(FlowLogRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_rule("code-208", "name-362");
        item.status = FlowLogRuleStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
