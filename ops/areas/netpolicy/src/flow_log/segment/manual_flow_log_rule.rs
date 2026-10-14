use super::*;

use acme_kit::text;

/// Where a manual flow log rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFlowLogRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualFlowLogRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualFlowLogRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual flow log rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogRule {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub timezone: String,
    pub version: i64,
    pub created_at: i64,
    pub port: i64,
    pub limit: i64,
    pub status: ManualFlowLogRuleStatus,
}

impl ManualFlowLogRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFlowLogRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFlowLogRuleStatus) -> bool {
        match self.status {
            ManualFlowLogRuleStatus::Proposed => {
                match next {
                    ManualFlowLogRuleStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFlowLogRuleStatus::Approved => {
                match next {
                    ManualFlowLogRuleStatus::Rejected | ManualFlowLogRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFlowLogRuleStatus::Rejected => {
                match next {
                    ManualFlowLogRuleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualFlowLogRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual flow log rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual flow log rule", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFlowLogRule {
        ManualFlowLogRule {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            timezone: self.timezone.clone(),
            version: self.version,
            created_at: self.created_at,
            port: self.port,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual flow log rule in status proposed.
pub fn new_manual_flow_log_rule(code: &str, name: &str) -> ManualFlowLogRule {
    ManualFlowLogRule {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        timezone: "".to_string(),
        version: 0,
        created_at: 0,
        port: 0,
        limit: 0,
        status: ManualFlowLogRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_flow_log_rule("code-346", "name-452");
        assert_eq!(item.status, ManualFlowLogRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_manual_flow_log_rule("code-346", "name-452");
        if item.move_to(ManualFlowLogRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFlowLogRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_flow_log_rule("code-346", "name-452");
        item.status = ManualFlowLogRuleStatus::Withdrawn;
        assert!(
            item.move_to(ManualFlowLogRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
