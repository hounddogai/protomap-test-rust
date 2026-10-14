use super::*;

use acme_kit::text;

/// Where a delegation batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DelegationBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DelegationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatch {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub unit: String,
    pub batch_size: i64,
    pub status: DelegationBatchStatus,
}

impl DelegationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationBatchStatus) -> bool {
        match self.status {
            DelegationBatchStatus::Created => {
                return next == DelegationBatchStatus::Published || next == DelegationBatchStatus::Retired;
            }
            DelegationBatchStatus::Validated => {
                return next == DelegationBatchStatus::Published || next == DelegationBatchStatus::Retired;
            }
            DelegationBatchStatus::Published => {
                match next {
                    DelegationBatchStatus::Retired => {
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

    pub fn move_to(&mut self, next: DelegationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.batch_size as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationBatch {
        DelegationBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a delegation batch in status created.
pub fn new_delegation_batch(code: &str, title: &str) -> DelegationBatch {
    DelegationBatch {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        unit: "".to_string(),
        batch_size: 0,
        status: DelegationBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_delegation_batch("code-204", "title-938");
        assert_eq!(item.status, DelegationBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_delegation_batch("code-204", "title-938");
        if item.move_to(DelegationBatchStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationBatchStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_delegation_batch("code-204", "title-938");
        item.status = DelegationBatchStatus::Retired;
        assert!(
            item.move_to(DelegationBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
