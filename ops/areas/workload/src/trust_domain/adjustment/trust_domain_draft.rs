use super::*;

use acme_kit::text;

/// Where a trust domain draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TrustDomainDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TrustDomainDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDraft {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub capacity: i64,
    pub principal: String,
    pub ttl_seconds: i64,
    pub description: String,
    pub batch_size: i64,
    pub region: String,
    pub status: TrustDomainDraftStatus,
}

impl TrustDomainDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainDraftStatus) -> bool {
        match self.status {
            TrustDomainDraftStatus::Requested => {
                return next == TrustDomainDraftStatus::Granted;
            }
            TrustDomainDraftStatus::Granted => {
                match next {
                    TrustDomainDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainDraftStatus::Revoked => {
                return next == TrustDomainDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainDraft {
        TrustDomainDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            capacity: self.capacity,
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            description: self.description.clone(),
            batch_size: self.batch_size,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a trust domain draft in status requested.
pub fn new_trust_domain_draft(code: &str, label: &str) -> TrustDomainDraft {
    TrustDomainDraft {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        capacity: 0,
        principal: "".to_string(),
        ttl_seconds: 0,
        description: "".to_string(),
        batch_size: 0,
        region: "".to_string(),
        status: TrustDomainDraftStatus::Requested,
    }
}
