use super::*;

use acme_kit::text;

/// Where a default rollout batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRolloutBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultRolloutBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultRolloutBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default rollout batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatch {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub checksum: String,
    pub quantity: i64,
    pub status: DefaultRolloutBatchStatus,
}

impl DefaultRolloutBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRolloutBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRolloutBatchStatus) -> bool {
        match self.status {
            DefaultRolloutBatchStatus::Requested => {
                return next == DefaultRolloutBatchStatus::Revoked;
            }
            DefaultRolloutBatchStatus::Granted => {
                match next {
                    DefaultRolloutBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRolloutBatchStatus::Revoked => {
                match next {
                    DefaultRolloutBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultRolloutBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollout batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> DefaultRolloutBatch {
        DefaultRolloutBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            checksum: self.checksum.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default rollout batch in status requested.
pub fn new_default_rollout_batch(code: &str, name: &str) -> DefaultRolloutBatch {
    DefaultRolloutBatch {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        checksum: "".to_string(),
        quantity: 0,
        status: DefaultRolloutBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_rollout_batch("code-547", "name-681");
        assert_eq!(item.status, DefaultRolloutBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_rollout_batch("code-547", "name-681");
        if item.move_to(DefaultRolloutBatchStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRolloutBatchStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_rollout_batch("code-547", "name-681");
        item.status = DefaultRolloutBatchStatus::Expired;
        assert!(
            item.move_to(DefaultRolloutBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
