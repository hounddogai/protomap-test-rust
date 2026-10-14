use super::*;

use acme_kit::text;

/// Where a evidence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EvidenceOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EvidenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceOverride {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub floor_value: f64,
    pub expires_at: i64,
    pub archived: bool,
    pub retries: i64,
    pub status: EvidenceOverrideStatus,
}

impl EvidenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceOverrideStatus) -> bool {
        match self.status {
            EvidenceOverrideStatus::Requested => {
                return next == EvidenceOverrideStatus::Granted || next == EvidenceOverrideStatus::Expired;
            }
            EvidenceOverrideStatus::Granted => {
                return next == EvidenceOverrideStatus::Revoked || next == EvidenceOverrideStatus::Expired;
            }
            EvidenceOverrideStatus::Revoked => {
                return next == EvidenceOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "evidence override", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceOverride {
        EvidenceOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            archived: self.archived,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a evidence override in status requested.
pub fn new_evidence_override(code: &str, name: &str) -> EvidenceOverride {
    EvidenceOverride {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        floor_value: 0.0,
        expires_at: 0,
        archived: false,
        retries: 0,
        status: EvidenceOverrideStatus::Requested,
    }
}
