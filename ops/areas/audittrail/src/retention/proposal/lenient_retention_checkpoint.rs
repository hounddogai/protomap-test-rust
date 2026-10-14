use super::*;

use acme_kit::text;

/// Where a lenient retention checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRetentionCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientRetentionCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientRetentionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient retention checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionCheckpoint {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub updated_at: i64,
    pub status: LenientRetentionCheckpointStatus,
}

impl LenientRetentionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRetentionCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRetentionCheckpointStatus) -> bool {
        match self.status {
            LenientRetentionCheckpointStatus::Created => {
                match next {
                    LenientRetentionCheckpointStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRetentionCheckpointStatus::Validated => {
                match next {
                    LenientRetentionCheckpointStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRetentionCheckpointStatus::Published => {
                return next == LenientRetentionCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRetentionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient retention checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.updated_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRetentionCheckpoint {
        LenientRetentionCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient retention checkpoint in status created.
pub fn new_lenient_retention_checkpoint(code: &str, name: &str) -> LenientRetentionCheckpoint {
    LenientRetentionCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        duration_minutes: 0,
        updated_at: 0,
        status: LenientRetentionCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_retention_checkpoint("code-959", "name-972");
        assert_eq!(item.status, LenientRetentionCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_retention_checkpoint("code-959", "name-972");
        if item.move_to(LenientRetentionCheckpointStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRetentionCheckpointStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_retention_checkpoint("code-959", "name-972");
        item.status = LenientRetentionCheckpointStatus::Retired;
        assert!(
            item.move_to(LenientRetentionCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
