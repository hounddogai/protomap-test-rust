use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal credential entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCredentialEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalCredentialEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalCredentialEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal credential entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialEntry {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub weight: f64,
    pub cluster: String,
    pub status: SeasonalCredentialEntryStatus,
}

impl SeasonalCredentialEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCredentialEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCredentialEntryStatus) -> bool {
        match self.status {
            SeasonalCredentialEntryStatus::Requested => {
                return next == SeasonalCredentialEntryStatus::Expired;
            }
            SeasonalCredentialEntryStatus::Granted => {
                match next {
                    SeasonalCredentialEntryStatus::Revoked | SeasonalCredentialEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialEntryStatus::Revoked => {
                return next == SeasonalCredentialEntryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalCredentialEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal credential entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> SeasonalCredentialEntry {
        SeasonalCredentialEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            weight: self.weight,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal credential entry in status requested.
pub fn new_seasonal_credential_entry(code: &str, label: &str) -> SeasonalCredentialEntry {
    SeasonalCredentialEntry {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        weight: 0.0,
        cluster: "".to_string(),
        status: SeasonalCredentialEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_credential_entry("code-685", "label-401");
        assert_eq!(item.status, SeasonalCredentialEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_seasonal_credential_entry("code-685", "label-401");
        if item.move_to(SeasonalCredentialEntryStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCredentialEntryStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_credential_entry("code-685", "label-401");
        item.status = SeasonalCredentialEntryStatus::Expired;
        assert!(
            item.move_to(SeasonalCredentialEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
