use super::*;

use acme_kit::text;

/// Where a default delegation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDelegationManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultDelegationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultDelegationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default delegation manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationManifest {
    pub code: String,
    pub title: String,
    pub label: String,
    pub scope: String,
    pub cost: f64,
    pub threshold: f64,
    pub port: i64,
    pub updated_at: i64,
    pub active: bool,
    pub status: DefaultDelegationManifestStatus,
}

impl DefaultDelegationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDelegationManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDelegationManifestStatus) -> bool {
        match self.status {
            DefaultDelegationManifestStatus::Requested => {
                return next == DefaultDelegationManifestStatus::Expired;
            }
            DefaultDelegationManifestStatus::Granted => {
                match next {
                    DefaultDelegationManifestStatus::Revoked | DefaultDelegationManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDelegationManifestStatus::Revoked => {
                match next {
                    DefaultDelegationManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultDelegationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default delegation manifest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDelegationManifest {
        DefaultDelegationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            cost: self.cost,
            threshold: self.threshold,
            port: self.port,
            updated_at: self.updated_at,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default delegation manifest in status requested.
pub fn new_default_delegation_manifest(code: &str, title: &str) -> DefaultDelegationManifest {
    DefaultDelegationManifest {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        scope: "".to_string(),
        cost: 0.0,
        threshold: 0.0,
        port: 0,
        updated_at: 0,
        active: false,
        status: DefaultDelegationManifestStatus::Requested,
    }
}
