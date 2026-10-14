use super::*;

use acme_kit::text;

/// Where a silence policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilencePolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilencePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilencePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence policy as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilencePolicy {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub key_id: String,
    pub sequence: i64,
    pub floor_value: f64,
    pub scope: String,
    pub status: SilencePolicyStatus,
}

impl SilencePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilencePolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilencePolicyStatus) -> bool {
        match self.status {
            SilencePolicyStatus::Requested => {
                match next {
                    SilencePolicyStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilencePolicyStatus::Granted => {
                return next == SilencePolicyStatus::Revoked || next == SilencePolicyStatus::Expired;
            }
            SilencePolicyStatus::Revoked => {
                return next == SilencePolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilencePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence policy", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.floor_value - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilencePolicy {
        SilencePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            floor_value: self.floor_value,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a silence policy in status requested.
pub fn new_silence_policy(code: &str, name: &str) -> SilencePolicy {
    SilencePolicy {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        key_id: "".to_string(),
        sequence: 0,
        floor_value: 0.0,
        scope: "".to_string(),
        status: SilencePolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_silence_policy("code-756", "name-88");
        assert_eq!(item.status, SilencePolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_silence_policy("code-756", "name-88");
        if item.move_to(SilencePolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilencePolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_silence_policy("code-756", "name-88");
        item.status = SilencePolicyStatus::Expired;
        assert!(
            item.move_to(SilencePolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
