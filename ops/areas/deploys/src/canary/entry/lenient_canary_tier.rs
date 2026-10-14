use super::*;

use acme_kit::text;

/// Where a lenient canary tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCanaryTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientCanaryTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientCanaryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient canary tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTier {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub budget: f64,
    pub updated_at: i64,
    pub key_id: String,
    pub sequence: i64,
    pub checksum: String,
    pub discount: f64,
    pub attempts: i64,
    pub phone: String,
    pub status: LenientCanaryTierStatus,
}

impl LenientCanaryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCanaryTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCanaryTierStatus) -> bool {
        match self.status {
            LenientCanaryTierStatus::Requested => {
                return next == LenientCanaryTierStatus::Granted || next == LenientCanaryTierStatus::Revoked;
            }
            LenientCanaryTierStatus::Granted => {
                match next {
                    LenientCanaryTierStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCanaryTierStatus::Revoked => {
                return next == LenientCanaryTierStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientCanaryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient canary tier {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.updated_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCanaryTier {
        LenientCanaryTier {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            budget: self.budget,
            updated_at: self.updated_at,
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            checksum: self.checksum.clone(),
            discount: self.discount,
            attempts: self.attempts,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient canary tier in status requested.
pub fn new_lenient_canary_tier(code: &str, label: &str) -> LenientCanaryTier {
    LenientCanaryTier {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        budget: 0.0,
        updated_at: 0,
        key_id: "".to_string(),
        sequence: 0,
        checksum: "".to_string(),
        discount: 0.0,
        attempts: 0,
        phone: "".to_string(),
        status: LenientCanaryTierStatus::Requested,
    }
}
