use super::*;

use acme_kit::text;

/// Where a usage window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindow {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub scope: String,
    pub ceiling: f64,
    pub region: String,
    pub ends_at: i64,
    pub ratio: f64,
    pub status: UsageWindowStatus,
}

impl UsageWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageWindowStatus) -> bool {
        match self.status {
            UsageWindowStatus::Requested => {
                match next {
                    UsageWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageWindowStatus::Granted => {
                match next {
                    UsageWindowStatus::Revoked | UsageWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageWindowStatus::Revoked => {
                match next {
                    UsageWindowStatus::Expired => {
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

    pub fn move_to(&mut self, next: UsageWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage window {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ends_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageWindow {
        UsageWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            ends_at: self.ends_at,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a usage window in status requested.
pub fn new_usage_window(code: &str, title: &str) -> UsageWindow {
    UsageWindow {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        scope: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        ends_at: 0,
        ratio: 0.0,
        status: UsageWindowStatus::Requested,
    }
}
