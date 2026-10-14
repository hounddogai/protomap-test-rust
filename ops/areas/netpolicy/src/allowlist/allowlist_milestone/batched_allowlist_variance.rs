use super::*;

use acme_kit::text;

/// Where a batched allowlist variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedAllowlistVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched allowlist variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistVariance {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub threshold: f64,
    pub starts_at: i64,
    pub locale: String,
    pub source: String,
    pub status: BatchedAllowlistVarianceStatus,
}

impl BatchedAllowlistVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistVarianceStatus) -> bool {
        match self.status {
            BatchedAllowlistVarianceStatus::Created => {
                return next == BatchedAllowlistVarianceStatus::Published
                    || next == BatchedAllowlistVarianceStatus::Retired;
            }
            BatchedAllowlistVarianceStatus::Validated => {
                match next {
                    BatchedAllowlistVarianceStatus::Published | BatchedAllowlistVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAllowlistVarianceStatus::Published => {
                return next == BatchedAllowlistVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAllowlistVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist variance {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.margin - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistVariance {
        BatchedAllowlistVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            threshold: self.threshold,
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched allowlist variance in status created.
pub fn new_batched_allowlist_variance(code: &str, title: &str) -> BatchedAllowlistVariance {
    BatchedAllowlistVariance {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        threshold: 0.0,
        starts_at: 0,
        locale: "".to_string(),
        source: "".to_string(),
        status: BatchedAllowlistVarianceStatus::Created,
    }
}
