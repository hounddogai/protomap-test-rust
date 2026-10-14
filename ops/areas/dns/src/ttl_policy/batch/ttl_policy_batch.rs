use super::*;

use acme_kit::text;

/// Where a ttl policy batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TtlPolicyBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TtlPolicyBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy batch as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBatch {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub burst: i64,
    pub approved: bool,
    pub budget: f64,
    pub scope: String,
    pub namespace: String,
    pub updated_at: i64,
    pub owner_id: String,
    pub status: TtlPolicyBatchStatus,
}

impl TtlPolicyBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyBatchStatus) -> bool {
        match self.status {
            TtlPolicyBatchStatus::Created => {
                return next == TtlPolicyBatchStatus::Published || next == TtlPolicyBatchStatus::Retired;
            }
            TtlPolicyBatchStatus::Validated => {
                return next == TtlPolicyBatchStatus::Published || next == TtlPolicyBatchStatus::Retired;
            }
            TtlPolicyBatchStatus::Published => {
                return next == TtlPolicyBatchStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy batch {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.burst as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyBatch {
        TtlPolicyBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            burst: self.burst,
            approved: self.approved,
            budget: self.budget,
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            updated_at: self.updated_at,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a ttl policy batch in status created.
pub fn new_ttl_policy_batch(code: &str, name: &str) -> TtlPolicyBatch {
    TtlPolicyBatch {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        burst: 0,
        approved: false,
        budget: 0.0,
        scope: "".to_string(),
        namespace: "".to_string(),
        updated_at: 0,
        owner_id: "".to_string(),
        status: TtlPolicyBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_ttl_policy_batch("code-818", "name-291");
        assert_eq!(item.status, TtlPolicyBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_ttl_policy_batch("code-818", "name-291");
        if item.move_to(TtlPolicyBatchStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyBatchStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_ttl_policy_batch("code-818", "name-291");
        item.status = TtlPolicyBatchStatus::Retired;
        assert!(
            item.move_to(TtlPolicyBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
