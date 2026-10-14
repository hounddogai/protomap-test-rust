use super::*;

use acme_kit::text;

/// Where a default action item target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultActionItemTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultActionItemTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultActionItemTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default action item target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTarget {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub floor_value: f64,
    pub description: String,
    pub updated_at: i64,
    pub window_days: i64,
    pub name: String,
    pub status: DefaultActionItemTargetStatus,
}

impl DefaultActionItemTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultActionItemTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultActionItemTargetStatus) -> bool {
        match self.status {
            DefaultActionItemTargetStatus::Requested => {
                match next {
                    DefaultActionItemTargetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultActionItemTargetStatus::Granted => {
                return next == DefaultActionItemTargetStatus::Expired;
            }
            DefaultActionItemTargetStatus::Revoked => {
                return next == DefaultActionItemTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultActionItemTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default action item target {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.updated_at as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultActionItemTarget {
        DefaultActionItemTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            floor_value: self.floor_value,
            description: self.description.clone(),
            updated_at: self.updated_at,
            window_days: self.window_days,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default action item target in status requested.
pub fn new_default_action_item_target(code: &str, title: &str) -> DefaultActionItemTarget {
    DefaultActionItemTarget {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        floor_value: 0.0,
        description: "".to_string(),
        updated_at: 0,
        window_days: 0,
        name: "".to_string(),
        status: DefaultActionItemTargetStatus::Requested,
    }
}
