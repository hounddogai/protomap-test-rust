use super::*;

use acme_kit::text;

/// Where a credential batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CredentialBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatch {
    pub code: String,
    pub name: String,
    pub title: String,
    pub quantity: i64,
    pub rotation_days: i64,
    pub status: CredentialBatchStatus,
}

impl CredentialBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialBatchStatus) -> bool {
        match self.status {
            CredentialBatchStatus::Idle => {
                match next {
                    CredentialBatchStatus::Draining | CredentialBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialBatchStatus::Warming => {
                return next == CredentialBatchStatus::Ready;
            }
            CredentialBatchStatus::Ready => {
                return next == CredentialBatchStatus::Draining;
            }
            CredentialBatchStatus::Draining => {
                return next == CredentialBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rotation_days as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialBatch {
        CredentialBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential batch in status idle.
pub fn new_credential_batch(code: &str, name: &str) -> CredentialBatch {
    CredentialBatch {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        quantity: 0,
        rotation_days: 0,
        status: CredentialBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_credential_batch("code-35", "name-39");
        assert_eq!(item.status, CredentialBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_credential_batch("code-35", "name-39");
        if item.move_to(CredentialBatchStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialBatchStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_credential_batch("code-35", "name-39");
        item.status = CredentialBatchStatus::Stopped;
        assert!(
            item.move_to(CredentialBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
