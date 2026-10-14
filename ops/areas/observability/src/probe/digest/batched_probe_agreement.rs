use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched probe agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedProbeAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedProbeAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedProbeAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched probe agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeAgreement {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub batch_size: i64,
    pub archived: bool,
    pub attributes: BTreeMap<String, String>,
    pub fingerprint: String,
    pub approved: bool,
    pub status: BatchedProbeAgreementStatus,
}

impl BatchedProbeAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedProbeAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedProbeAgreementStatus) -> bool {
        match self.status {
            BatchedProbeAgreementStatus::Created => {
                return next == BatchedProbeAgreementStatus::Published || next == BatchedProbeAgreementStatus::Retired;
            }
            BatchedProbeAgreementStatus::Validated => {
                match next {
                    BatchedProbeAgreementStatus::Published | BatchedProbeAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedProbeAgreementStatus::Published => {
                return next == BatchedProbeAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedProbeAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched probe agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched probe agreement", self.code)
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.batch_size as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedProbeAgreement {
        BatchedProbeAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            batch_size: self.batch_size,
            archived: self.archived,
            attributes: self.attributes.clone(),
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched probe agreement in status created.
pub fn new_batched_probe_agreement(code: &str, title: &str) -> BatchedProbeAgreement {
    BatchedProbeAgreement {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        batch_size: 0,
        archived: false,
        attributes: BTreeMap::new(),
        fingerprint: "".to_string(),
        approved: false,
        status: BatchedProbeAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_probe_agreement("code-657", "title-100");
        assert_eq!(item.status, BatchedProbeAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_batched_probe_agreement("code-657", "title-100");
        if item.move_to(BatchedProbeAgreementStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedProbeAgreementStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_probe_agreement("code-657", "title-100");
        item.status = BatchedProbeAgreementStatus::Retired;
        assert!(
            item.move_to(BatchedProbeAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
