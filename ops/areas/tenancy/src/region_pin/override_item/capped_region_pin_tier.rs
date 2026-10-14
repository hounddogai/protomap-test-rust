use super::*;

use acme_kit::text;

/// Where a capped region pin tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRegionPinTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedRegionPinTierStatus {
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

impl std::fmt::Display for CappedRegionPinTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped region pin tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTier {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub version: i64,
    pub budget: f64,
    pub retries: i64,
    pub notes: String,
    pub title: String,
    pub rate: f64,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub status: CappedRegionPinTierStatus,
}

impl CappedRegionPinTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRegionPinTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRegionPinTierStatus) -> bool {
        match self.status {
            CappedRegionPinTierStatus::Open => {
                match next {
                    CappedRegionPinTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRegionPinTierStatus::InReview => {
                match next {
                    CappedRegionPinTierStatus::Resolved | CappedRegionPinTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRegionPinTierStatus::Resolved => {
                return next == CappedRegionPinTierStatus::Reopened || next == CappedRegionPinTierStatus::Archived;
            }
            CappedRegionPinTierStatus::Reopened => {
                return next == CappedRegionPinTierStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRegionPinTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped region pin tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped region pin tier", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.expires_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRegionPinTier {
        CappedRegionPinTier {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            version: self.version,
            budget: self.budget,
            retries: self.retries,
            notes: self.notes.clone(),
            title: self.title.clone(),
            rate: self.rate,
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped region pin tier in status open.
pub fn new_capped_region_pin_tier(code: &str, name: &str) -> CappedRegionPinTier {
    CappedRegionPinTier {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        version: 0,
        budget: 0.0,
        retries: 0,
        notes: "".to_string(),
        title: "".to_string(),
        rate: 0.0,
        ttl_seconds: 0,
        expires_at: 0,
        status: CappedRegionPinTierStatus::Open,
    }
}
