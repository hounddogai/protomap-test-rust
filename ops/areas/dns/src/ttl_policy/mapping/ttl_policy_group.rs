use super::*;

use acme_kit::text;

/// Where a ttl policy group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TtlPolicyGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TtlPolicyGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyGroup {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub segment: String,
    pub revision: i64,
    pub description: String,
    pub margin: f64,
    pub status: TtlPolicyGroupStatus,
}

impl TtlPolicyGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyGroupStatus) -> bool {
        match self.status {
            TtlPolicyGroupStatus::Proposed => {
                return next == TtlPolicyGroupStatus::Rejected || next == TtlPolicyGroupStatus::Withdrawn;
            }
            TtlPolicyGroupStatus::Approved => {
                return next == TtlPolicyGroupStatus::Rejected;
            }
            TtlPolicyGroupStatus::Rejected => {
                return next == TtlPolicyGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyGroup {
        TtlPolicyGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            segment: self.segment.clone(),
            revision: self.revision,
            description: self.description.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a ttl policy group in status proposed.
pub fn new_ttl_policy_group(code: &str, name: &str) -> TtlPolicyGroup {
    TtlPolicyGroup {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        segment: "".to_string(),
        revision: 0,
        description: "".to_string(),
        margin: 0.0,
        status: TtlPolicyGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_ttl_policy_group("code-378", "name-323");
        assert_eq!(item.status, TtlPolicyGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_ttl_policy_group("code-378", "name-323");
        if item.move_to(TtlPolicyGroupStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyGroupStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_ttl_policy_group("code-378", "name-323");
        item.status = TtlPolicyGroupStatus::Withdrawn;
        assert!(
            item.move_to(TtlPolicyGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
