use super::*;

use acme_kit::text;

/// Where a tiered postmortem digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPostmortemDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredPostmortemDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredPostmortemDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered postmortem digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDigest {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub rotation_days: i64,
    pub starts_at: i64,
    pub status: TieredPostmortemDigestStatus,
}

impl TieredPostmortemDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPostmortemDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPostmortemDigestStatus) -> bool {
        match self.status {
            TieredPostmortemDigestStatus::Created => {
                match next {
                    TieredPostmortemDigestStatus::Validated | TieredPostmortemDigestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPostmortemDigestStatus::Validated => {
                return next == TieredPostmortemDigestStatus::Published
                    || next == TieredPostmortemDigestStatus::Retired;
            }
            TieredPostmortemDigestStatus::Published => {
                match next {
                    TieredPostmortemDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredPostmortemDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered postmortem digest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPostmortemDigest {
        TieredPostmortemDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            rotation_days: self.rotation_days,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered postmortem digest in status created.
pub fn new_tiered_postmortem_digest(code: &str, name: &str) -> TieredPostmortemDigest {
    TieredPostmortemDigest {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        rotation_days: 0,
        starts_at: 0,
        status: TieredPostmortemDigestStatus::Created,
    }
}
