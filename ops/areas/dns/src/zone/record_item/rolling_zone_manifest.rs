use super::*;

use acme_kit::text;

/// Where a rolling zone manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingZoneManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingZoneManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RollingZoneManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling zone manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifest {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub currency: String,
    pub ratio: f64,
    pub status: RollingZoneManifestStatus,
}

impl RollingZoneManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingZoneManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingZoneManifestStatus) -> bool {
        match self.status {
            RollingZoneManifestStatus::Open => {
                return next == RollingZoneManifestStatus::InReview;
            }
            RollingZoneManifestStatus::InReview => {
                match next {
                    RollingZoneManifestStatus::Resolved | RollingZoneManifestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingZoneManifestStatus::Resolved => {
                match next {
                    RollingZoneManifestStatus::Reopened | RollingZoneManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingZoneManifestStatus::Reopened => {
                return next == RollingZoneManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingZoneManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling zone manifest {} cannot move from {} to {}",
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
        format!("{} {}", "rolling zone manifest", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingZoneManifest {
        RollingZoneManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            currency: self.currency.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling zone manifest in status open.
pub fn new_rolling_zone_manifest(code: &str, title: &str) -> RollingZoneManifest {
    RollingZoneManifest {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        currency: "".to_string(),
        ratio: 0.0,
        status: RollingZoneManifestStatus::Open,
    }
}
