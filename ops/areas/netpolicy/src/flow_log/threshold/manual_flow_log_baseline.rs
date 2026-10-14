use super::*;

use acme_kit::text;

/// Where a manual flow log baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFlowLogBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualFlowLogBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualFlowLogBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual flow log baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogBaseline {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub retries: i64,
    pub source: String,
    pub score: f64,
    pub margin: f64,
    pub reviewer: String,
    pub audience: String,
    pub port: i64,
    pub window_days: i64,
    pub status: ManualFlowLogBaselineStatus,
}

impl ManualFlowLogBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFlowLogBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFlowLogBaselineStatus) -> bool {
        match self.status {
            ManualFlowLogBaselineStatus::Proposed => {
                return next == ManualFlowLogBaselineStatus::Withdrawn;
            }
            ManualFlowLogBaselineStatus::Approved => {
                match next {
                    ManualFlowLogBaselineStatus::Rejected | ManualFlowLogBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFlowLogBaselineStatus::Rejected => {
                return next == ManualFlowLogBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualFlowLogBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual flow log baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        (self.margin * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFlowLogBaseline {
        ManualFlowLogBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            retries: self.retries,
            source: self.source.clone(),
            score: self.score,
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            port: self.port,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual flow log baseline in status proposed.
pub fn new_manual_flow_log_baseline(code: &str, name: &str) -> ManualFlowLogBaseline {
    ManualFlowLogBaseline {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        retries: 0,
        source: "".to_string(),
        score: 0.0,
        margin: 0.0,
        reviewer: "".to_string(),
        audience: "".to_string(),
        port: 0,
        window_days: 0,
        status: ManualFlowLogBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_flow_log_baseline("code-815", "name-806");
        assert_eq!(item.status, ManualFlowLogBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_manual_flow_log_baseline("code-815", "name-806");
        if item.move_to(ManualFlowLogBaselineStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFlowLogBaselineStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_flow_log_baseline("code-815", "name-806");
        item.status = ManualFlowLogBaselineStatus::Withdrawn;
        assert!(
            item.move_to(ManualFlowLogBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
