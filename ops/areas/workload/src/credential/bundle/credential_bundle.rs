use super::*;

use acme_kit::text;

/// Where a credential bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CredentialBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CredentialBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundle {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub issuer: String,
    pub maximum: f64,
    pub fingerprint: String,
    pub sequence: i64,
    pub starts_at: i64,
    pub status: CredentialBundleStatus,
}

impl CredentialBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialBundleStatus) -> bool {
        match self.status {
            CredentialBundleStatus::Planned => {
                match next {
                    CredentialBundleStatus::Scheduled | CredentialBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialBundleStatus::Scheduled => {
                match next {
                    CredentialBundleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialBundleStatus::InProgress => {
                match next {
                    CredentialBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialBundleStatus::Completed => {
                match next {
                    CredentialBundleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CredentialBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential bundle", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialBundle {
        CredentialBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential bundle in status planned.
pub fn new_credential_bundle(code: &str, name: &str) -> CredentialBundle {
    CredentialBundle {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        issuer: "".to_string(),
        maximum: 0.0,
        fingerprint: "".to_string(),
        sequence: 0,
        starts_at: 0,
        status: CredentialBundleStatus::Planned,
    }
}
