use super::*;

use acme_kit::text;

/// Where a verifier checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier checkpoint as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCheckpoint {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub approved: bool,
    pub active: bool,
    pub minimum: f64,
    pub name: String,
    pub namespace: String,
    pub status: VerifierCheckpointStatus,
}

impl VerifierCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierCheckpointStatus) -> bool {
        match self.status {
            VerifierCheckpointStatus::Requested => {
                match next {
                    VerifierCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierCheckpointStatus::Granted => {
                match next {
                    VerifierCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierCheckpointStatus::Revoked => {
                return next == VerifierCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "verifier checkpoint", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.minimum * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierCheckpoint {
        VerifierCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            approved: self.approved,
            active: self.active,
            minimum: self.minimum,
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a verifier checkpoint in status requested.
pub fn new_verifier_checkpoint(code: &str, label: &str) -> VerifierCheckpoint {
    VerifierCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        reviewer: "".to_string(),
        duration_minutes: 0,
        approved: false,
        active: false,
        minimum: 0.0,
        name: "".to_string(),
        namespace: "".to_string(),
        status: VerifierCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_checkpoint("code-113", "label-567");
        assert_eq!(item.status, VerifierCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_verifier_checkpoint("code-113", "label-567");
        if item.move_to(VerifierCheckpointStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierCheckpointStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_checkpoint("code-113", "label-567");
        item.status = VerifierCheckpointStatus::Expired;
        assert!(
            item.move_to(VerifierCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
