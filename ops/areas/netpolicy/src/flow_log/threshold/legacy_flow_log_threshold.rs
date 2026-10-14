use super::*;

use acme_kit::text;

/// Where a legacy flow log threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFlowLogThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyFlowLogThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyFlowLogThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy flow log threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThreshold {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub key_id: String,
    pub rotation_days: i64,
    pub score: f64,
    pub notes: String,
    pub status: LegacyFlowLogThresholdStatus,
}

impl LegacyFlowLogThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFlowLogThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFlowLogThresholdStatus) -> bool {
        match self.status {
            LegacyFlowLogThresholdStatus::Requested => {
                return next == LegacyFlowLogThresholdStatus::Revoked;
            }
            LegacyFlowLogThresholdStatus::Granted => {
                return next == LegacyFlowLogThresholdStatus::Expired;
            }
            LegacyFlowLogThresholdStatus::Revoked => {
                match next {
                    LegacyFlowLogThresholdStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyFlowLogThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy flow log threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy flow log threshold", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.score + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFlowLogThreshold {
        LegacyFlowLogThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            key_id: self.key_id.clone(),
            rotation_days: self.rotation_days,
            score: self.score,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy flow log threshold in status requested.
pub fn new_legacy_flow_log_threshold(code: &str, label: &str) -> LegacyFlowLogThreshold {
    LegacyFlowLogThreshold {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        key_id: "".to_string(),
        rotation_days: 0,
        score: 0.0,
        notes: "".to_string(),
        status: LegacyFlowLogThresholdStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_flow_log_threshold("code-179", "label-165");
        assert_eq!(item.status, LegacyFlowLogThresholdStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_flow_log_threshold("code-179", "label-165");
        if item.move_to(LegacyFlowLogThresholdStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFlowLogThresholdStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_flow_log_threshold("code-179", "label-165");
        item.status = LegacyFlowLogThresholdStatus::Expired;
        assert!(
            item.move_to(LegacyFlowLogThresholdStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
