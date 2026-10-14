use super::*;

use acme_kit::text;

/// Where a pending trust domain slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTrustDomainSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingTrustDomainSlotStatus {
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

impl std::fmt::Display for PendingTrustDomainSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending trust domain slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainSlot {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub source: String,
    pub updated_at: i64,
    pub notes: String,
    pub limit: i64,
    pub status: PendingTrustDomainSlotStatus,
}

impl PendingTrustDomainSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTrustDomainSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTrustDomainSlotStatus) -> bool {
        match self.status {
            PendingTrustDomainSlotStatus::Open => {
                match next {
                    PendingTrustDomainSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTrustDomainSlotStatus::InReview => {
                return next == PendingTrustDomainSlotStatus::Reopened;
            }
            PendingTrustDomainSlotStatus::Resolved => {
                match next {
                    PendingTrustDomainSlotStatus::Reopened | PendingTrustDomainSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTrustDomainSlotStatus::Reopened => {
                match next {
                    PendingTrustDomainSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingTrustDomainSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending trust domain slot {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.minimum + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTrustDomainSlot {
        PendingTrustDomainSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            source: self.source.clone(),
            updated_at: self.updated_at,
            notes: self.notes.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending trust domain slot in status open.
pub fn new_pending_trust_domain_slot(code: &str, name: &str) -> PendingTrustDomainSlot {
    PendingTrustDomainSlot {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        source: "".to_string(),
        updated_at: 0,
        notes: "".to_string(),
        limit: 0,
        status: PendingTrustDomainSlotStatus::Open,
    }
}
