use super::*;

use acme_kit::text;

/// Where a tenant digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TenantDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TenantDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantDigest {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub locale: String,
    pub unit: String,
    pub timezone: String,
    pub cost: f64,
    pub starts_at: i64,
    pub version: i64,
    pub label: String,
    pub name: String,
    pub status: TenantDigestStatus,
}

impl TenantDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantDigestStatus) -> bool {
        match self.status {
            TenantDigestStatus::Idle => {
                match next {
                    TenantDigestStatus::Warming | TenantDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantDigestStatus::Warming => {
                match next {
                    TenantDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantDigestStatus::Ready => {
                return next == TenantDigestStatus::Draining;
            }
            TenantDigestStatus::Draining => {
                return next == TenantDigestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.version as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantDigest {
        TenantDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            locale: self.locale.clone(),
            unit: self.unit.clone(),
            timezone: self.timezone.clone(),
            cost: self.cost,
            starts_at: self.starts_at,
            version: self.version,
            label: self.label.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tenant digest in status idle.
pub fn new_tenant_digest(code: &str, title: &str) -> TenantDigest {
    TenantDigest {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        locale: "".to_string(),
        unit: "".to_string(),
        timezone: "".to_string(),
        cost: 0.0,
        starts_at: 0,
        version: 0,
        label: "".to_string(),
        name: "".to_string(),
        status: TenantDigestStatus::Idle,
    }
}
