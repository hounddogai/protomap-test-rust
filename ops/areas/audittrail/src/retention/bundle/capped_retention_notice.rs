use super::*;

use acme_kit::text;

/// Where a capped retention notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRetentionNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedRetentionNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedRetentionNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped retention notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionNotice {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub unit: String,
    pub owner_id: String,
    pub amount: f64,
    pub status: CappedRetentionNoticeStatus,
}

impl CappedRetentionNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRetentionNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRetentionNoticeStatus) -> bool {
        match self.status {
            CappedRetentionNoticeStatus::Requested => {
                match next {
                    CappedRetentionNoticeStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRetentionNoticeStatus::Granted => {
                return next == CappedRetentionNoticeStatus::Revoked || next == CappedRetentionNoticeStatus::Expired;
            }
            CappedRetentionNoticeStatus::Revoked => {
                return next == CappedRetentionNoticeStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRetentionNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped retention notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> CappedRetentionNotice {
        CappedRetentionNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped retention notice in status requested.
pub fn new_capped_retention_notice(code: &str, label: &str) -> CappedRetentionNotice {
    CappedRetentionNotice {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        unit: "".to_string(),
        owner_id: "".to_string(),
        amount: 0.0,
        status: CappedRetentionNoticeStatus::Requested,
    }
}
