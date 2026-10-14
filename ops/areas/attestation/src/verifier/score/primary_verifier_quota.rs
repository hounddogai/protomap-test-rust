use super::*;

use acme_kit::text;

/// Where a primary verifier quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryVerifierQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryVerifierQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryVerifierQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary verifier quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierQuota {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub quantity: i64,
    pub channel: String,
    pub status: PrimaryVerifierQuotaStatus,
}

impl PrimaryVerifierQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryVerifierQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryVerifierQuotaStatus) -> bool {
        match self.status {
            PrimaryVerifierQuotaStatus::Requested => {
                match next {
                    PrimaryVerifierQuotaStatus::Revoked | PrimaryVerifierQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierQuotaStatus::Granted => {
                match next {
                    PrimaryVerifierQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierQuotaStatus::Revoked => {
                return next == PrimaryVerifierQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryVerifierQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary verifier quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary verifier quota", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PrimaryVerifierQuota {
        PrimaryVerifierQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary verifier quota in status requested.
pub fn new_primary_verifier_quota(code: &str, title: &str) -> PrimaryVerifierQuota {
    PrimaryVerifierQuota {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        quantity: 0,
        channel: "".to_string(),
        status: PrimaryVerifierQuotaStatus::Requested,
    }
}
