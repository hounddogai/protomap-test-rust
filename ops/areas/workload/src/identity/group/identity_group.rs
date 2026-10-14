use super::*;

use acme_kit::text;

/// Where a identity group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityGroup {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub starts_at: i64,
    pub priority: i64,
    pub visible: bool,
    pub audience: String,
    pub status: IdentityGroupStatus,
}

impl IdentityGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityGroupStatus) -> bool {
        match self.status {
            IdentityGroupStatus::Requested => {
                match next {
                    IdentityGroupStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityGroupStatus::Granted => {
                match next {
                    IdentityGroupStatus::Revoked | IdentityGroupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityGroupStatus::Revoked => {
                match next {
                    IdentityGroupStatus::Expired => {
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

    pub fn move_to(&mut self, next: IdentityGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityGroup {
        IdentityGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            starts_at: self.starts_at,
            priority: self.priority,
            visible: self.visible,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity group in status requested.
pub fn new_identity_group(code: &str, name: &str) -> IdentityGroup {
    IdentityGroup {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        starts_at: 0,
        priority: 0,
        visible: false,
        audience: "".to_string(),
        status: IdentityGroupStatus::Requested,
    }
}
