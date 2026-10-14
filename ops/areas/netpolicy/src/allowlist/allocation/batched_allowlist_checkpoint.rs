use super::*;

use acme_kit::text;

/// Where a batched allowlist checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedAllowlistCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched allowlist checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistCheckpoint {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub port: i64,
    pub archived: bool,
    pub expires_at: i64,
    pub key_id: String,
    pub discount: f64,
    pub summary: String,
    pub label: String,
    pub status: BatchedAllowlistCheckpointStatus,
}

impl BatchedAllowlistCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistCheckpointStatus) -> bool {
        match self.status {
            BatchedAllowlistCheckpointStatus::Created => {
                match next {
                    BatchedAllowlistCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAllowlistCheckpointStatus::Validated => {
                return next == BatchedAllowlistCheckpointStatus::Published
                    || next == BatchedAllowlistCheckpointStatus::Retired;
            }
            BatchedAllowlistCheckpointStatus::Published => {
                match next {
                    BatchedAllowlistCheckpointStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedAllowlistCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.burst as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistCheckpoint {
        BatchedAllowlistCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            port: self.port,
            archived: self.archived,
            expires_at: self.expires_at,
            key_id: self.key_id.clone(),
            discount: self.discount,
            summary: self.summary.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched allowlist checkpoint in status created.
pub fn new_batched_allowlist_checkpoint(code: &str, name: &str) -> BatchedAllowlistCheckpoint {
    BatchedAllowlistCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        ttl_seconds: 0,
        port: 0,
        archived: false,
        expires_at: 0,
        key_id: "".to_string(),
        discount: 0.0,
        summary: "".to_string(),
        label: "".to_string(),
        status: BatchedAllowlistCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_allowlist_checkpoint("code-376", "name-128");
        assert_eq!(item.status, BatchedAllowlistCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_batched_allowlist_checkpoint("code-376", "name-128");
        if item.move_to(BatchedAllowlistCheckpointStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAllowlistCheckpointStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_allowlist_checkpoint("code-376", "name-128");
        item.status = BatchedAllowlistCheckpointStatus::Retired;
        assert!(
            item.move_to(BatchedAllowlistCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
