use super::*;

use acme_kit::text;

/// Where a tiered slo manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSloManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredSloManifestStatus {
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

impl std::fmt::Display for TieredSloManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered slo manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifest {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub quantity: i64,
    pub currency: String,
    pub timezone: String,
    pub starts_at: i64,
    pub ceiling: f64,
    pub segment: String,
    pub margin: f64,
    pub status: TieredSloManifestStatus,
}

impl TieredSloManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSloManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSloManifestStatus) -> bool {
        match self.status {
            TieredSloManifestStatus::Idle => {
                return next == TieredSloManifestStatus::Ready || next == TieredSloManifestStatus::Draining;
            }
            TieredSloManifestStatus::Warming => {
                match next {
                    TieredSloManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSloManifestStatus::Ready => {
                match next {
                    TieredSloManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSloManifestStatus::Draining => {
                return next == TieredSloManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredSloManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered slo manifest {} cannot move from {} to {}",
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

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSloManifest {
        TieredSloManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            quantity: self.quantity,
            currency: self.currency.clone(),
            timezone: self.timezone.clone(),
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            segment: self.segment.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered slo manifest in status idle.
pub fn new_tiered_slo_manifest(code: &str, name: &str) -> TieredSloManifest {
    TieredSloManifest {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        quantity: 0,
        currency: "".to_string(),
        timezone: "".to_string(),
        starts_at: 0,
        ceiling: 0.0,
        segment: "".to_string(),
        margin: 0.0,
        status: TieredSloManifestStatus::Idle,
    }
}
