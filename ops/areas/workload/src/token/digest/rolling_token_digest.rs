use super::*;

use acme_kit::text;

/// Where a rolling token digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTokenDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingTokenDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingTokenDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling token digest as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigest {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub discount: f64,
    pub unit: String,
    pub status: RollingTokenDigestStatus,
}

impl RollingTokenDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTokenDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTokenDigestStatus) -> bool {
        match self.status {
            RollingTokenDigestStatus::Created => {
                return next == RollingTokenDigestStatus::Validated || next == RollingTokenDigestStatus::Published;
            }
            RollingTokenDigestStatus::Validated => {
                return next == RollingTokenDigestStatus::Published || next == RollingTokenDigestStatus::Retired;
            }
            RollingTokenDigestStatus::Published => {
                match next {
                    RollingTokenDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollingTokenDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling token digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling token digest", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTokenDigest {
        RollingTokenDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            discount: self.discount,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling token digest in status created.
pub fn new_rolling_token_digest(code: &str, title: &str) -> RollingTokenDigest {
    RollingTokenDigest {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        discount: 0.0,
        unit: "".to_string(),
        status: RollingTokenDigestStatus::Created,
    }
}
