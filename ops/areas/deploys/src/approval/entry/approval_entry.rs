use super::*;

use acme_kit::text;

/// Where a approval entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEntry {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub window_days: i64,
    pub label: String,
    pub fingerprint: String,
    pub archived: bool,
    pub status: ApprovalEntryStatus,
}

impl ApprovalEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalEntryStatus) -> bool {
        match self.status {
            ApprovalEntryStatus::Created => {
                match next {
                    ApprovalEntryStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalEntryStatus::Validated => {
                match next {
                    ApprovalEntryStatus::Published | ApprovalEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalEntryStatus::Published => {
                match next {
                    ApprovalEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: ApprovalEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval entry", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.window_days as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalEntry {
        ApprovalEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            window_days: self.window_days,
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a approval entry in status created.
pub fn new_approval_entry(code: &str, name: &str) -> ApprovalEntry {
    ApprovalEntry {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        window_days: 0,
        label: "".to_string(),
        fingerprint: "".to_string(),
        archived: false,
        status: ApprovalEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_approval_entry("code-397", "name-978");
        assert_eq!(item.status, ApprovalEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_approval_entry("code-397", "name-978");
        if item.move_to(ApprovalEntryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalEntryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_approval_entry("code-397", "name-978");
        item.status = ApprovalEntryStatus::Retired;
        assert!(
            item.move_to(ApprovalEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
