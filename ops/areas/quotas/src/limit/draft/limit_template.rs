use super::*;

use acme_kit::text;

/// Where a limit template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a limit template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitTemplate {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub capacity: i64,
    pub phone: String,
    pub status: LimitTemplateStatus,
}

impl LimitTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitTemplateStatus) -> bool {
        match self.status {
            LimitTemplateStatus::Requested => {
                return next == LimitTemplateStatus::Granted;
            }
            LimitTemplateStatus::Granted => {
                match next {
                    LimitTemplateStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitTemplateStatus::Revoked => {
                return next == LimitTemplateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "limit template", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> LimitTemplate {
        LimitTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            capacity: self.capacity,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a limit template in status requested.
pub fn new_limit_template(code: &str, title: &str) -> LimitTemplate {
    LimitTemplate {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        capacity: 0,
        phone: "".to_string(),
        status: LimitTemplateStatus::Requested,
    }
}
