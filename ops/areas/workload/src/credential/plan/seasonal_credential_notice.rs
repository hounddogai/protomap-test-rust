use super::*;

use acme_kit::text;

/// Where a seasonal credential notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCredentialNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalCredentialNoticeStatus {
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

impl std::fmt::Display for SeasonalCredentialNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal credential notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialNotice {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub ends_at: i64,
    pub limit: i64,
    pub status: SeasonalCredentialNoticeStatus,
}

impl SeasonalCredentialNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCredentialNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCredentialNoticeStatus) -> bool {
        match self.status {
            SeasonalCredentialNoticeStatus::Draft => {
                match next {
                    SeasonalCredentialNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialNoticeStatus::Pending => {
                return next == SeasonalCredentialNoticeStatus::Closed;
            }
            SeasonalCredentialNoticeStatus::Active => {
                match next {
                    SeasonalCredentialNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialNoticeStatus::Suspended => {
                return next == SeasonalCredentialNoticeStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalCredentialNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal credential notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal credential notice", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCredentialNotice {
        SeasonalCredentialNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            ends_at: self.ends_at,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal credential notice in status draft.
pub fn new_seasonal_credential_notice(code: &str, label: &str) -> SeasonalCredentialNotice {
    SeasonalCredentialNotice {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        ends_at: 0,
        limit: 0,
        status: SeasonalCredentialNoticeStatus::Draft,
    }
}
