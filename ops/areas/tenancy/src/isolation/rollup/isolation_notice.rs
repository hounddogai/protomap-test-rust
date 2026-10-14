use super::*;

use acme_kit::text;

/// Where a isolation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IsolationNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IsolationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationNotice {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub cost: f64,
    pub priority: i64,
    pub budget: f64,
    pub status: IsolationNoticeStatus,
}

impl IsolationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationNoticeStatus) -> bool {
        match self.status {
            IsolationNoticeStatus::Requested => {
                return next == IsolationNoticeStatus::Granted || next == IsolationNoticeStatus::Revoked;
            }
            IsolationNoticeStatus::Granted => {
                return next == IsolationNoticeStatus::Revoked || next == IsolationNoticeStatus::Expired;
            }
            IsolationNoticeStatus::Revoked => {
                match next {
                    IsolationNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: IsolationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.budget - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationNotice {
        IsolationNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            cost: self.cost,
            priority: self.priority,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a isolation notice in status requested.
pub fn new_isolation_notice(code: &str, name: &str) -> IsolationNotice {
    IsolationNotice {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        cost: 0.0,
        priority: 0,
        budget: 0.0,
        status: IsolationNoticeStatus::Requested,
    }
}
