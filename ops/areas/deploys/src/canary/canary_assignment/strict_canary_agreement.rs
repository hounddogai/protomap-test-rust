use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict canary agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictCanaryAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictCanaryAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for StrictCanaryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict canary agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreement {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub key_id: String,
    pub updated_at: i64,
    pub category: String,
    pub created_at: i64,
    pub fingerprint: String,
    pub contact_email: String,
    pub attributes: BTreeMap<String, String>,
    pub status: StrictCanaryAgreementStatus,
}

impl StrictCanaryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictCanaryAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictCanaryAgreementStatus) -> bool {
        match self.status {
            StrictCanaryAgreementStatus::Idle => {
                match next {
                    StrictCanaryAgreementStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCanaryAgreementStatus::Warming => {
                return next == StrictCanaryAgreementStatus::Stopped;
            }
            StrictCanaryAgreementStatus::Ready => {
                match next {
                    StrictCanaryAgreementStatus::Draining | StrictCanaryAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCanaryAgreementStatus::Draining => {
                return next == StrictCanaryAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictCanaryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict canary agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictCanaryAgreement {
        StrictCanaryAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            key_id: self.key_id.clone(),
            updated_at: self.updated_at,
            category: self.category.clone(),
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            contact_email: self.contact_email.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict canary agreement in status idle.
pub fn new_strict_canary_agreement(code: &str, name: &str) -> StrictCanaryAgreement {
    StrictCanaryAgreement {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        key_id: "".to_string(),
        updated_at: 0,
        category: "".to_string(),
        created_at: 0,
        fingerprint: "".to_string(),
        contact_email: "".to_string(),
        attributes: BTreeMap::new(),
        status: StrictCanaryAgreementStatus::Idle,
    }
}
