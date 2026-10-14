use super::*;

use acme_kit::text;

/// Where a manual isolation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIsolationEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualIsolationEntryStatus {
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

impl std::fmt::Display for ManualIsolationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual isolation entry as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationEntry {
    pub code: String,
    pub label: String,
    pub region: String,
    pub expires_at: i64,
    pub namespace: String,
    pub host: String,
    pub batch_size: i64,
    pub revision: i64,
    pub key_id: String,
    pub status: ManualIsolationEntryStatus,
}

impl ManualIsolationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIsolationEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIsolationEntryStatus) -> bool {
        match self.status {
            ManualIsolationEntryStatus::Open => {
                match next {
                    ManualIsolationEntryStatus::Resolved | ManualIsolationEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationEntryStatus::InReview => {
                match next {
                    ManualIsolationEntryStatus::Reopened | ManualIsolationEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationEntryStatus::Resolved => {
                return next == ManualIsolationEntryStatus::Reopened || next == ManualIsolationEntryStatus::Archived;
            }
            ManualIsolationEntryStatus::Reopened => {
                match next {
                    ManualIsolationEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualIsolationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual isolation entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIsolationEntry {
        ManualIsolationEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            host: self.host.clone(),
            batch_size: self.batch_size,
            revision: self.revision,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual isolation entry in status open.
pub fn new_manual_isolation_entry(code: &str, label: &str) -> ManualIsolationEntry {
    ManualIsolationEntry {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        expires_at: 0,
        namespace: "".to_string(),
        host: "".to_string(),
        batch_size: 0,
        revision: 0,
        key_id: "".to_string(),
        status: ManualIsolationEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_isolation_entry("code-920", "label-81");
        assert_eq!(item.status, ManualIsolationEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_manual_isolation_entry("code-920", "label-81");
        if item.move_to(ManualIsolationEntryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualIsolationEntryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_isolation_entry("code-920", "label-81");
        item.status = ManualIsolationEntryStatus::Archived;
        assert!(
            item.move_to(ManualIsolationEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
