use super::*;

use acme_kit::text;

/// Where a global sink policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSinkPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalSinkPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalSinkPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global sink policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkPolicy {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub floor_value: f64,
    pub quota_units: i64,
    pub threshold: f64,
    pub key_id: String,
    pub score: f64,
    pub cost: f64,
    pub fingerprint: String,
    pub namespace: String,
    pub status: GlobalSinkPolicyStatus,
}

impl GlobalSinkPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSinkPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSinkPolicyStatus) -> bool {
        match self.status {
            GlobalSinkPolicyStatus::Proposed => {
                return next == GlobalSinkPolicyStatus::Approved || next == GlobalSinkPolicyStatus::Withdrawn;
            }
            GlobalSinkPolicyStatus::Approved => {
                match next {
                    GlobalSinkPolicyStatus::Rejected | GlobalSinkPolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSinkPolicyStatus::Rejected => {
                return next == GlobalSinkPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSinkPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global sink policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSinkPolicy {
        GlobalSinkPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            floor_value: self.floor_value,
            quota_units: self.quota_units,
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            score: self.score,
            cost: self.cost,
            fingerprint: self.fingerprint.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global sink policy in status proposed.
pub fn new_global_sink_policy(code: &str, title: &str) -> GlobalSinkPolicy {
    GlobalSinkPolicy {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        floor_value: 0.0,
        quota_units: 0,
        threshold: 0.0,
        key_id: "".to_string(),
        score: 0.0,
        cost: 0.0,
        fingerprint: "".to_string(),
        namespace: "".to_string(),
        status: GlobalSinkPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_sink_policy("code-914", "title-284");
        assert_eq!(item.status, GlobalSinkPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_global_sink_policy("code-914", "title-284");
        if item.move_to(GlobalSinkPolicyStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSinkPolicyStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_sink_policy("code-914", "title-284");
        item.status = GlobalSinkPolicyStatus::Withdrawn;
        assert!(
            item.move_to(GlobalSinkPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
