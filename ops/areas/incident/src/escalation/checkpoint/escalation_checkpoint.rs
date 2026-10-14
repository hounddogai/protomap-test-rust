use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a escalation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EscalationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EscalationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCheckpoint {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub batch_size: i64,
    pub ttl_seconds: i64,
    pub attributes: BTreeMap<String, String>,
    pub weight: f64,
    pub status: EscalationCheckpointStatus,
}

impl EscalationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationCheckpointStatus) -> bool {
        match self.status {
            EscalationCheckpointStatus::Requested => {
                match next {
                    EscalationCheckpointStatus::Revoked | EscalationCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationCheckpointStatus::Granted => {
                return next == EscalationCheckpointStatus::Expired;
            }
            EscalationCheckpointStatus::Revoked => {
                return next == EscalationCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation checkpoint", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_load(&self) -> f64 {
        (self.weight * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationCheckpoint {
        EscalationCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            batch_size: self.batch_size,
            ttl_seconds: self.ttl_seconds,
            attributes: self.attributes.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation checkpoint in status requested.
pub fn new_escalation_checkpoint(code: &str, label: &str) -> EscalationCheckpoint {
    EscalationCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        batch_size: 0,
        ttl_seconds: 0,
        attributes: BTreeMap::new(),
        weight: 0.0,
        status: EscalationCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_escalation_checkpoint("code-739", "label-665");
        assert_eq!(item.status, EscalationCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_escalation_checkpoint("code-739", "label-665");
        if item.move_to(EscalationCheckpointStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationCheckpointStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_escalation_checkpoint("code-739", "label-665");
        item.status = EscalationCheckpointStatus::Expired;
        assert!(
            item.move_to(EscalationCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
