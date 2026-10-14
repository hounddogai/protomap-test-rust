use super::*;

use acme_kit::text;

/// Where a retention ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RetentionLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RetentionLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention ledger as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedger {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub tags: Vec<String>,
    pub summary: String,
    pub reviewer: String,
    pub revision: i64,
    pub owner_id: String,
    pub maximum: f64,
    pub status: RetentionLedgerStatus,
}

impl RetentionLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionLedgerStatus) -> bool {
        match self.status {
            RetentionLedgerStatus::Created => {
                match next {
                    RetentionLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionLedgerStatus::Validated => {
                return next == RetentionLedgerStatus::Retired;
            }
            RetentionLedgerStatus::Published => {
                match next {
                    RetentionLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: RetentionLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention ledger {} cannot move from {} to {}",
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

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.version as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionLedger {
        RetentionLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            tags: self.tags.clone(),
            summary: self.summary.clone(),
            reviewer: self.reviewer.clone(),
            revision: self.revision,
            owner_id: self.owner_id.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention ledger in status created.
pub fn new_retention_ledger(code: &str, label: &str) -> RetentionLedger {
    RetentionLedger {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        tags: Vec::new(),
        summary: "".to_string(),
        reviewer: "".to_string(),
        revision: 0,
        owner_id: "".to_string(),
        maximum: 0.0,
        status: RetentionLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_retention_ledger("code-36", "label-184");
        assert_eq!(item.status, RetentionLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_retention_ledger("code-36", "label-184");
        if item.move_to(RetentionLedgerStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionLedgerStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_retention_ledger("code-36", "label-184");
        item.status = RetentionLedgerStatus::Retired;
        assert!(
            item.move_to(RetentionLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
