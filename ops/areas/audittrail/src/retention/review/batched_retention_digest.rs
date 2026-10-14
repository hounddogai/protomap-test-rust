use super::*;

use acme_kit::text;

/// Where a batched retention digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRetentionDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedRetentionDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedRetentionDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched retention digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionDigest {
    pub code: String,
    pub name: String,
    pub host: String,
    pub ceiling: f64,
    pub active: bool,
    pub status: BatchedRetentionDigestStatus,
}

impl BatchedRetentionDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRetentionDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRetentionDigestStatus) -> bool {
        match self.status {
            BatchedRetentionDigestStatus::Draft => {
                match next {
                    BatchedRetentionDigestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionDigestStatus::Pending => {
                return next == BatchedRetentionDigestStatus::Active || next == BatchedRetentionDigestStatus::Suspended;
            }
            BatchedRetentionDigestStatus::Active => {
                match next {
                    BatchedRetentionDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionDigestStatus::Suspended => {
                return next == BatchedRetentionDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRetentionDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched retention digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched retention digest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> BatchedRetentionDigest {
        BatchedRetentionDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            ceiling: self.ceiling,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched retention digest in status draft.
pub fn new_batched_retention_digest(code: &str, name: &str) -> BatchedRetentionDigest {
    BatchedRetentionDigest {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        ceiling: 0.0,
        active: false,
        status: BatchedRetentionDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_retention_digest("code-305", "name-931");
        assert_eq!(item.status, BatchedRetentionDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_batched_retention_digest("code-305", "name-931");
        if item.move_to(BatchedRetentionDigestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRetentionDigestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_retention_digest("code-305", "name-931");
        item.status = BatchedRetentionDigestStatus::Closed;
        assert!(
            item.move_to(BatchedRetentionDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
