use super::*;

use acme_kit::text;

/// Where a batched credential mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedCredentialMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedCredentialMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BatchedCredentialMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched credential mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialMapping {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub archived: bool,
    pub version: i64,
    pub ttl_seconds: i64,
    pub minimum: f64,
    pub namespace: String,
    pub description: String,
    pub summary: String,
    pub status: BatchedCredentialMappingStatus,
}

impl BatchedCredentialMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedCredentialMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedCredentialMappingStatus) -> bool {
        match self.status {
            BatchedCredentialMappingStatus::Open => {
                return next == BatchedCredentialMappingStatus::Resolved;
            }
            BatchedCredentialMappingStatus::InReview => {
                match next {
                    BatchedCredentialMappingStatus::Reopened | BatchedCredentialMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCredentialMappingStatus::Resolved => {
                return next == BatchedCredentialMappingStatus::Archived;
            }
            BatchedCredentialMappingStatus::Reopened => {
                match next {
                    BatchedCredentialMappingStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedCredentialMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched credential mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedCredentialMapping {
        BatchedCredentialMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            archived: self.archived,
            version: self.version,
            ttl_seconds: self.ttl_seconds,
            minimum: self.minimum,
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched credential mapping in status open.
pub fn new_batched_credential_mapping(code: &str, label: &str) -> BatchedCredentialMapping {
    BatchedCredentialMapping {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        archived: false,
        version: 0,
        ttl_seconds: 0,
        minimum: 0.0,
        namespace: "".to_string(),
        description: "".to_string(),
        summary: "".to_string(),
        status: BatchedCredentialMappingStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_credential_mapping("code-441", "label-584");
        assert_eq!(item.status, BatchedCredentialMappingStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_batched_credential_mapping("code-441", "label-584");
        if item.move_to(BatchedCredentialMappingStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedCredentialMappingStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_credential_mapping("code-441", "label-584");
        item.status = BatchedCredentialMappingStatus::Archived;
        assert!(
            item.move_to(BatchedCredentialMappingStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
