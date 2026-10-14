use super::*;

use acme_kit::text;

/// Where a default action item adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultActionItemAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultActionItemAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultActionItemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default action item adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAdjustment {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub expires_at: i64,
    pub ceiling: f64,
    pub approved: bool,
    pub scope: String,
    pub status: DefaultActionItemAdjustmentStatus,
}

impl DefaultActionItemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultActionItemAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultActionItemAdjustmentStatus) -> bool {
        match self.status {
            DefaultActionItemAdjustmentStatus::Requested => {
                return next == DefaultActionItemAdjustmentStatus::Granted;
            }
            DefaultActionItemAdjustmentStatus::Granted => {
                match next {
                    DefaultActionItemAdjustmentStatus::Revoked | DefaultActionItemAdjustmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultActionItemAdjustmentStatus::Revoked => {
                match next {
                    DefaultActionItemAdjustmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultActionItemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default action item adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.expires_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultActionItemAdjustment {
        DefaultActionItemAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            expires_at: self.expires_at,
            ceiling: self.ceiling,
            approved: self.approved,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default action item adjustment in status requested.
pub fn new_default_action_item_adjustment(code: &str, name: &str) -> DefaultActionItemAdjustment {
    DefaultActionItemAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        expires_at: 0,
        ceiling: 0.0,
        approved: false,
        scope: "".to_string(),
        status: DefaultActionItemAdjustmentStatus::Requested,
    }
}
