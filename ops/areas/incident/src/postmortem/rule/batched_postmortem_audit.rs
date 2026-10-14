use super::*;

use acme_kit::text;

/// Where a batched postmortem audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPostmortemAuditStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedPostmortemAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedPostmortemAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched postmortem audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemAudit {
    pub code: String,
    pub name: String,
    pub category: String,
    pub rotation_days: i64,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub ceiling: f64,
    pub status: BatchedPostmortemAuditStatus,
}

impl BatchedPostmortemAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPostmortemAuditStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPostmortemAuditStatus) -> bool {
        match self.status {
            BatchedPostmortemAuditStatus::Created => {
                return next == BatchedPostmortemAuditStatus::Published;
            }
            BatchedPostmortemAuditStatus::Validated => {
                match next {
                    BatchedPostmortemAuditStatus::Published | BatchedPostmortemAuditStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPostmortemAuditStatus::Published => {
                return next == BatchedPostmortemAuditStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPostmortemAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched postmortem audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPostmortemAudit {
        BatchedPostmortemAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched postmortem audit in status created.
pub fn new_batched_postmortem_audit(code: &str, name: &str) -> BatchedPostmortemAudit {
    BatchedPostmortemAudit {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        rotation_days: 0,
        ttl_seconds: 0,
        owner_id: "".to_string(),
        ceiling: 0.0,
        status: BatchedPostmortemAuditStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_postmortem_audit("code-9", "name-167");
        assert_eq!(item.status, BatchedPostmortemAuditStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_batched_postmortem_audit("code-9", "name-167");
        if item.move_to(BatchedPostmortemAuditStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPostmortemAuditStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_postmortem_audit("code-9", "name-167");
        item.status = BatchedPostmortemAuditStatus::Retired;
        assert!(
            item.move_to(BatchedPostmortemAuditStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
