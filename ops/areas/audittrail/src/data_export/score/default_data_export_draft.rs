use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default data export draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDataExportDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultDataExportDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultDataExportDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default data export draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportDraft {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub timezone: String,
    pub attempts: i64,
    pub fingerprint: String,
    pub minimum: f64,
    pub attributes: BTreeMap<String, String>,
    pub locale: String,
    pub cluster: String,
    pub notes: String,
    pub status: DefaultDataExportDraftStatus,
}

impl DefaultDataExportDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDataExportDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDataExportDraftStatus) -> bool {
        match self.status {
            DefaultDataExportDraftStatus::Requested => {
                match next {
                    DefaultDataExportDraftStatus::Granted | DefaultDataExportDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDataExportDraftStatus::Granted => {
                match next {
                    DefaultDataExportDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDataExportDraftStatus::Revoked => {
                match next {
                    DefaultDataExportDraftStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultDataExportDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default data export draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.attempts as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDataExportDraft {
        DefaultDataExportDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            fingerprint: self.fingerprint.clone(),
            minimum: self.minimum,
            attributes: self.attributes.clone(),
            locale: self.locale.clone(),
            cluster: self.cluster.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default data export draft in status requested.
pub fn new_default_data_export_draft(code: &str, title: &str) -> DefaultDataExportDraft {
    DefaultDataExportDraft {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        timezone: "".to_string(),
        attempts: 0,
        fingerprint: "".to_string(),
        minimum: 0.0,
        attributes: BTreeMap::new(),
        locale: "".to_string(),
        cluster: "".to_string(),
        notes: "".to_string(),
        status: DefaultDataExportDraftStatus::Requested,
    }
}
