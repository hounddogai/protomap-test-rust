use super::*;

use acme_kit::text;

/// Where a batched release entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReleaseEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedReleaseEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedReleaseEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched release entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseEntry {
    pub code: String,
    pub name: String,
    pub description: String,
    pub quantity: i64,
    pub locale: String,
    pub status: BatchedReleaseEntryStatus,
}

impl BatchedReleaseEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReleaseEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReleaseEntryStatus) -> bool {
        match self.status {
            BatchedReleaseEntryStatus::Created => {
                match next {
                    BatchedReleaseEntryStatus::Validated | BatchedReleaseEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReleaseEntryStatus::Validated => {
                match next {
                    BatchedReleaseEntryStatus::Published | BatchedReleaseEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReleaseEntryStatus::Published => {
                return next == BatchedReleaseEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedReleaseEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched release entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> BatchedReleaseEntry {
        BatchedReleaseEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            quantity: self.quantity,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched release entry in status created.
pub fn new_batched_release_entry(code: &str, name: &str) -> BatchedReleaseEntry {
    BatchedReleaseEntry {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        quantity: 0,
        locale: "".to_string(),
        status: BatchedReleaseEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_release_entry("code-995", "name-366");
        assert_eq!(item.status, BatchedReleaseEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_release_entry("code-995", "name-366");
        if item.move_to(BatchedReleaseEntryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedReleaseEntryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_release_entry("code-995", "name-366");
        item.status = BatchedReleaseEntryStatus::Retired;
        assert!(
            item.move_to(BatchedReleaseEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
