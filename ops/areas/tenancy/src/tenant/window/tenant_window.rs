use super::*;

use acme_kit::text;

/// Where a tenant window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantWindow {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub unit: String,
    pub timezone: String,
    pub attempts: i64,
    pub status: TenantWindowStatus,
}

impl TenantWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantWindowStatus) -> bool {
        match self.status {
            TenantWindowStatus::Created => {
                return next == TenantWindowStatus::Published || next == TenantWindowStatus::Retired;
            }
            TenantWindowStatus::Validated => {
                return next == TenantWindowStatus::Retired;
            }
            TenantWindowStatus::Published => {
                match next {
                    TenantWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: TenantWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant window {} cannot move from {} to {}",
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

    pub fn net_value(&self) -> f64 {
        ((self.attempts as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantWindow {
        TenantWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tenant window in status created.
pub fn new_tenant_window(code: &str, title: &str) -> TenantWindow {
    TenantWindow {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        unit: "".to_string(),
        timezone: "".to_string(),
        attempts: 0,
        status: TenantWindowStatus::Created,
    }
}
