use super::*;

use acme_kit::text;

/// Where a federation batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationBatchStatus {
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

impl std::fmt::Display for FederationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationBatch {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub batch_size: i64,
    pub country: String,
    pub currency: String,
    pub status: FederationBatchStatus,
}

impl FederationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationBatchStatus) -> bool {
        match self.status {
            FederationBatchStatus::Open => {
                match next {
                    FederationBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationBatchStatus::InReview => {
                match next {
                    FederationBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationBatchStatus::Resolved => {
                return next == FederationBatchStatus::Archived;
            }
            FederationBatchStatus::Reopened => {
                match next {
                    FederationBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: FederationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> FederationBatch {
        FederationBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            batch_size: self.batch_size,
            country: self.country.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a federation batch in status open.
pub fn new_federation_batch(code: &str, name: &str) -> FederationBatch {
    FederationBatch {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        batch_size: 0,
        country: "".to_string(),
        currency: "".to_string(),
        status: FederationBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_batch("code-129", "name-263");
        assert_eq!(item.status, FederationBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_federation_batch("code-129", "name-263");
        if item.move_to(FederationBatchStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationBatchStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_batch("code-129", "name-263");
        item.status = FederationBatchStatus::Archived;
        assert!(
            item.move_to(FederationBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
