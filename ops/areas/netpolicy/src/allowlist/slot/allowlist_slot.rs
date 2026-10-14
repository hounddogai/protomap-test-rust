use super::*;

use acme_kit::text;

/// Where a allowlist slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistSlotStatus {
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

impl std::fmt::Display for AllowlistSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSlot {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub audience: String,
    pub notes: String,
    pub cost: f64,
    pub revision: i64,
    pub status: AllowlistSlotStatus,
}

impl AllowlistSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistSlotStatus) -> bool {
        match self.status {
            AllowlistSlotStatus::Draft => {
                return next == AllowlistSlotStatus::Closed;
            }
            AllowlistSlotStatus::Pending => {
                return next == AllowlistSlotStatus::Active;
            }
            AllowlistSlotStatus::Active => {
                return next == AllowlistSlotStatus::Suspended || next == AllowlistSlotStatus::Closed;
            }
            AllowlistSlotStatus::Suspended => {
                match next {
                    AllowlistSlotStatus::Closed => {
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

    pub fn move_to(&mut self, next: AllowlistSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.cost * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistSlot {
        AllowlistSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            cost: self.cost,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist slot in status draft.
pub fn new_allowlist_slot(code: &str, name: &str) -> AllowlistSlot {
    AllowlistSlot {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        audience: "".to_string(),
        notes: "".to_string(),
        cost: 0.0,
        revision: 0,
        status: AllowlistSlotStatus::Draft,
    }
}
