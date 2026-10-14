use super::*;

use acme_kit::text;

/// Where a suspension exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionException {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub visible: bool,
    pub label: String,
    pub amount: f64,
    pub limit: i64,
    pub burst: i64,
    pub status: SuspensionExceptionStatus,
}

impl SuspensionException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionExceptionStatus) -> bool {
        match self.status {
            SuspensionExceptionStatus::Requested => {
                match next {
                    SuspensionExceptionStatus::Granted | SuspensionExceptionStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionExceptionStatus::Granted => {
                match next {
                    SuspensionExceptionStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionExceptionStatus::Revoked => {
                return next == SuspensionExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionException {
        SuspensionException {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            visible: self.visible,
            label: self.label.clone(),
            amount: self.amount,
            limit: self.limit,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension exception in status requested.
pub fn new_suspension_exception(code: &str, name: &str) -> SuspensionException {
    SuspensionException {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        visible: false,
        label: "".to_string(),
        amount: 0.0,
        limit: 0,
        burst: 0,
        status: SuspensionExceptionStatus::Requested,
    }
}
