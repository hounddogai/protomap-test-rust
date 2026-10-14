use super::*;

use acme_kit::text;

/// Where a legacy flow log override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFlowLogOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyFlowLogOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyFlowLogOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy flow log override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogOverride {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub region: String,
    pub cost: f64,
    pub fingerprint: String,
    pub ratio: f64,
    pub namespace: String,
    pub unit: String,
    pub port: i64,
    pub status: LegacyFlowLogOverrideStatus,
}

impl LegacyFlowLogOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFlowLogOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFlowLogOverrideStatus) -> bool {
        match self.status {
            LegacyFlowLogOverrideStatus::Proposed => {
                return next == LegacyFlowLogOverrideStatus::Approved || next == LegacyFlowLogOverrideStatus::Withdrawn;
            }
            LegacyFlowLogOverrideStatus::Approved => {
                match next {
                    LegacyFlowLogOverrideStatus::Rejected | LegacyFlowLogOverrideStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFlowLogOverrideStatus::Rejected => {
                return next == LegacyFlowLogOverrideStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFlowLogOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy flow log override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        (self.cost * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFlowLogOverride {
        LegacyFlowLogOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            region: self.region.clone(),
            cost: self.cost,
            fingerprint: self.fingerprint.clone(),
            ratio: self.ratio,
            namespace: self.namespace.clone(),
            unit: self.unit.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy flow log override in status proposed.
pub fn new_legacy_flow_log_override(code: &str, title: &str) -> LegacyFlowLogOverride {
    LegacyFlowLogOverride {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        region: "".to_string(),
        cost: 0.0,
        fingerprint: "".to_string(),
        ratio: 0.0,
        namespace: "".to_string(),
        unit: "".to_string(),
        port: 0,
        status: LegacyFlowLogOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_flow_log_override("code-236", "title-328");
        assert_eq!(item.status, LegacyFlowLogOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_legacy_flow_log_override("code-236", "title-328");
        if item.move_to(LegacyFlowLogOverrideStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFlowLogOverrideStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_flow_log_override("code-236", "title-328");
        item.status = LegacyFlowLogOverrideStatus::Withdrawn;
        assert!(
            item.move_to(LegacyFlowLogOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
