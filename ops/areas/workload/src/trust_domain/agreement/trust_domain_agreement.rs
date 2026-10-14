use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a trust domain agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TrustDomainAgreementStatus {
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

impl std::fmt::Display for TrustDomainAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreement {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub cluster: String,
    pub amount: f64,
    pub checksum: String,
    pub burst: i64,
    pub notes: String,
    pub tenant_id: String,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub status: TrustDomainAgreementStatus,
}

impl TrustDomainAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainAgreementStatus) -> bool {
        match self.status {
            TrustDomainAgreementStatus::Planned => {
                return next == TrustDomainAgreementStatus::Cancelled;
            }
            TrustDomainAgreementStatus::Scheduled => {
                match next {
                    TrustDomainAgreementStatus::InProgress | TrustDomainAgreementStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAgreementStatus::InProgress => {
                match next {
                    TrustDomainAgreementStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAgreementStatus::Completed => {
                return next == TrustDomainAgreementStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain agreement", self.code)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.amount + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainAgreement {
        TrustDomainAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            cluster: self.cluster.clone(),
            amount: self.amount,
            checksum: self.checksum.clone(),
            burst: self.burst,
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a trust domain agreement in status planned.
pub fn new_trust_domain_agreement(code: &str, name: &str) -> TrustDomainAgreement {
    TrustDomainAgreement {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        cluster: "".to_string(),
        amount: 0.0,
        checksum: "".to_string(),
        burst: 0,
        notes: "".to_string(),
        tenant_id: "".to_string(),
        attributes: BTreeMap::new(),
        grace_days: 0,
        status: TrustDomainAgreementStatus::Planned,
    }
}
