use super::*;

use acme_kit::text;

/// Where a credential slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential slot as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSlot {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub score: f64,
    pub timezone: String,
    pub visible: bool,
    pub starts_at: i64,
    pub status: CredentialSlotStatus,
}

impl CredentialSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialSlotStatus) -> bool {
        match self.status {
            CredentialSlotStatus::Requested => {
                match next {
                    CredentialSlotStatus::Revoked | CredentialSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialSlotStatus::Granted => {
                match next {
                    CredentialSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialSlotStatus::Revoked => {
                match next {
                    CredentialSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: CredentialSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential slot {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.starts_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialSlot {
        CredentialSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            score: self.score,
            timezone: self.timezone.clone(),
            visible: self.visible,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential slot in status requested.
pub fn new_credential_slot(code: &str, name: &str) -> CredentialSlot {
    CredentialSlot {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        score: 0.0,
        timezone: "".to_string(),
        visible: false,
        starts_at: 0,
        status: CredentialSlotStatus::Requested,
    }
}
