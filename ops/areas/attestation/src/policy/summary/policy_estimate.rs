use super::*;

use acme_kit::text;

/// Where a policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PolicyEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEstimate {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub phone: String,
    pub reviewer: String,
    pub notes: String,
    pub issuer: String,
    pub rotation_days: i64,
    pub host: String,
    pub external_ref: String,
    pub status: PolicyEstimateStatus,
}

impl PolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyEstimateStatus) -> bool {
        match self.status {
            PolicyEstimateStatus::Proposed => {
                return next == PolicyEstimateStatus::Approved || next == PolicyEstimateStatus::Withdrawn;
            }
            PolicyEstimateStatus::Approved => {
                return next == PolicyEstimateStatus::Rejected || next == PolicyEstimateStatus::Withdrawn;
            }
            PolicyEstimateStatus::Rejected => {
                match next {
                    PolicyEstimateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy estimate", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> PolicyEstimate {
        PolicyEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            phone: self.phone.clone(),
            reviewer: self.reviewer.clone(),
            notes: self.notes.clone(),
            issuer: self.issuer.clone(),
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a policy estimate in status proposed.
pub fn new_policy_estimate(code: &str, label: &str) -> PolicyEstimate {
    PolicyEstimate {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        phone: "".to_string(),
        reviewer: "".to_string(),
        notes: "".to_string(),
        issuer: "".to_string(),
        rotation_days: 0,
        host: "".to_string(),
        external_ref: "".to_string(),
        status: PolicyEstimateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_policy_estimate("code-911", "label-139");
        assert_eq!(item.status, PolicyEstimateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_policy_estimate("code-911", "label-139");
        if item.move_to(PolicyEstimateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyEstimateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_policy_estimate("code-911", "label-139");
        item.status = PolicyEstimateStatus::Withdrawn;
        assert!(
            item.move_to(PolicyEstimateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
