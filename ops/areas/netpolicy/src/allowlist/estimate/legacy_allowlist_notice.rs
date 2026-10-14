use super::*;

use acme_kit::text;

/// Where a legacy allowlist notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyAllowlistNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LegacyAllowlistNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy allowlist notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistNotice {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub visible: bool,
    pub amount: f64,
    pub status: LegacyAllowlistNoticeStatus,
}

impl LegacyAllowlistNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistNoticeStatus) -> bool {
        match self.status {
            LegacyAllowlistNoticeStatus::Draft => {
                return next == LegacyAllowlistNoticeStatus::Pending || next == LegacyAllowlistNoticeStatus::Active;
            }
            LegacyAllowlistNoticeStatus::Pending => {
                return next == LegacyAllowlistNoticeStatus::Suspended || next == LegacyAllowlistNoticeStatus::Closed;
            }
            LegacyAllowlistNoticeStatus::Active => {
                match next {
                    LegacyAllowlistNoticeStatus::Suspended | LegacyAllowlistNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistNoticeStatus::Suspended => {
                match next {
                    LegacyAllowlistNoticeStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyAllowlistNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy allowlist notice", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.amount * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistNotice {
        LegacyAllowlistNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            visible: self.visible,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy allowlist notice in status draft.
pub fn new_legacy_allowlist_notice(code: &str, title: &str) -> LegacyAllowlistNotice {
    LegacyAllowlistNotice {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        visible: false,
        amount: 0.0,
        status: LegacyAllowlistNoticeStatus::Draft,
    }
}
