use super::*;

use acme_kit::text;

/// Where a limit draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitDraft {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub window_days: i64,
    pub country: String,
    pub amount: f64,
    pub reviewer: String,
    pub external_ref: String,
    pub key_id: String,
    pub status: LimitDraftStatus,
}

impl LimitDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitDraftStatus) -> bool {
        match self.status {
            LimitDraftStatus::Requested => {
                return next == LimitDraftStatus::Revoked || next == LimitDraftStatus::Expired;
            }
            LimitDraftStatus::Granted => {
                return next == LimitDraftStatus::Expired;
            }
            LimitDraftStatus::Revoked => {
                match next {
                    LimitDraftStatus::Expired => {
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

    pub fn move_to(&mut self, next: LimitDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.amount * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitDraft {
        LimitDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            window_days: self.window_days,
            country: self.country.clone(),
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a limit draft in status requested.
pub fn new_limit_draft(code: &str, title: &str) -> LimitDraft {
    LimitDraft {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        window_days: 0,
        country: "".to_string(),
        amount: 0.0,
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        key_id: "".to_string(),
        status: LimitDraftStatus::Requested,
    }
}
