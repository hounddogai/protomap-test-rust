use super::*;

use acme_kit::text;

/// Where a identity tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IdentityTierStatus {
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

impl std::fmt::Display for IdentityTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTier {
    pub code: String,
    pub title: String,
    pub name: String,
    pub host: String,
    pub active: bool,
    pub quantity: i64,
    pub status: IdentityTierStatus,
}

impl IdentityTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityTierStatus) -> bool {
        match self.status {
            IdentityTierStatus::Draft => {
                match next {
                    IdentityTierStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityTierStatus::Pending => {
                match next {
                    IdentityTierStatus::Active | IdentityTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityTierStatus::Active => {
                match next {
                    IdentityTierStatus::Suspended | IdentityTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityTierStatus::Suspended => {
                return next == IdentityTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "identity tier", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> IdentityTier {
        IdentityTier {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            active: self.active,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a identity tier in status draft.
pub fn new_identity_tier(code: &str, title: &str) -> IdentityTier {
    IdentityTier {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        host: "".to_string(),
        active: false,
        quantity: 0,
        status: IdentityTierStatus::Draft,
    }
}
