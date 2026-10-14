use super::*;

use acme_kit::text;

/// Where a rolling grant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingGrantTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingGrantTargetStatus {
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

impl std::fmt::Display for RollingGrantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling grant target as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTarget {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub principal: String,
    pub checksum: String,
    pub attempts: i64,
    pub namespace: String,
    pub status: RollingGrantTargetStatus,
}

impl RollingGrantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingGrantTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingGrantTargetStatus) -> bool {
        match self.status {
            RollingGrantTargetStatus::Draft => {
                match next {
                    RollingGrantTargetStatus::Pending | RollingGrantTargetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingGrantTargetStatus::Pending => {
                return next == RollingGrantTargetStatus::Suspended || next == RollingGrantTargetStatus::Closed;
            }
            RollingGrantTargetStatus::Active => {
                return next == RollingGrantTargetStatus::Suspended;
            }
            RollingGrantTargetStatus::Suspended => {
                return next == RollingGrantTargetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingGrantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling grant target {} cannot move from {} to {}",
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.expires_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingGrantTarget {
        RollingGrantTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            principal: self.principal.clone(),
            checksum: self.checksum.clone(),
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling grant target in status draft.
pub fn new_rolling_grant_target(code: &str, name: &str) -> RollingGrantTarget {
    RollingGrantTarget {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        principal: "".to_string(),
        checksum: "".to_string(),
        attempts: 0,
        namespace: "".to_string(),
        status: RollingGrantTargetStatus::Draft,
    }
}
