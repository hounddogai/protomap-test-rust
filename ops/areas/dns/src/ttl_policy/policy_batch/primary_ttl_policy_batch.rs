use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary ttl policy batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTtlPolicyBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryTtlPolicyBatchStatus {
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

impl std::fmt::Display for PrimaryTtlPolicyBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary ttl policy batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatch {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub cluster: String,
    pub name: String,
    pub capacity: i64,
    pub archived: bool,
    pub sequence: i64,
    pub summary: String,
    pub unit: String,
    pub active: bool,
    pub status: PrimaryTtlPolicyBatchStatus,
}

impl PrimaryTtlPolicyBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTtlPolicyBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTtlPolicyBatchStatus) -> bool {
        match self.status {
            PrimaryTtlPolicyBatchStatus::Draft => {
                match next {
                    PrimaryTtlPolicyBatchStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyBatchStatus::Pending => {
                return next == PrimaryTtlPolicyBatchStatus::Closed;
            }
            PrimaryTtlPolicyBatchStatus::Active => {
                return next == PrimaryTtlPolicyBatchStatus::Suspended || next == PrimaryTtlPolicyBatchStatus::Closed;
            }
            PrimaryTtlPolicyBatchStatus::Suspended => {
                return next == PrimaryTtlPolicyBatchStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTtlPolicyBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary ttl policy batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary ttl policy batch", self.code)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.sequence as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTtlPolicyBatch {
        PrimaryTtlPolicyBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            cluster: self.cluster.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            archived: self.archived,
            sequence: self.sequence,
            summary: self.summary.clone(),
            unit: self.unit.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary ttl policy batch in status draft.
pub fn new_primary_ttl_policy_batch(code: &str, title: &str) -> PrimaryTtlPolicyBatch {
    PrimaryTtlPolicyBatch {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        cluster: "".to_string(),
        name: "".to_string(),
        capacity: 0,
        archived: false,
        sequence: 0,
        summary: "".to_string(),
        unit: "".to_string(),
        active: false,
        status: PrimaryTtlPolicyBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_ttl_policy_batch("code-536", "title-276");
        assert_eq!(item.status, PrimaryTtlPolicyBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_primary_ttl_policy_batch("code-536", "title-276");
        if item.move_to(PrimaryTtlPolicyBatchStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTtlPolicyBatchStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_ttl_policy_batch("code-536", "title-276");
        item.status = PrimaryTtlPolicyBatchStatus::Closed;
        assert!(
            item.move_to(PrimaryTtlPolicyBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
