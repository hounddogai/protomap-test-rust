use super::*;

use acme_kit::text;

/// Where a credential entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CredentialEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CredentialEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntry {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub title: String,
    pub region: String,
    pub issuer: String,
    pub version: i64,
    pub scope: String,
    pub status: CredentialEntryStatus,
}

impl CredentialEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialEntryStatus) -> bool {
        match self.status {
            CredentialEntryStatus::Proposed => {
                match next {
                    CredentialEntryStatus::Approved | CredentialEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialEntryStatus::Approved => {
                return next == CredentialEntryStatus::Rejected || next == CredentialEntryStatus::Withdrawn;
            }
            CredentialEntryStatus::Rejected => {
                match next {
                    CredentialEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CredentialEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential entry", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.burst as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialEntry {
        CredentialEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            title: self.title.clone(),
            region: self.region.clone(),
            issuer: self.issuer.clone(),
            version: self.version,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a credential entry in status proposed.
pub fn new_credential_entry(code: &str, label: &str) -> CredentialEntry {
    CredentialEntry {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        title: "".to_string(),
        region: "".to_string(),
        issuer: "".to_string(),
        version: 0,
        scope: "".to_string(),
        status: CredentialEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_credential_entry("code-889", "label-75");
        assert_eq!(item.status, CredentialEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_credential_entry("code-889", "label-75");
        if item.move_to(CredentialEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_credential_entry("code-889", "label-75");
        item.status = CredentialEntryStatus::Withdrawn;
        assert!(
            item.move_to(CredentialEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
