use super::*;

use acme_kit::text;

/// Where a ttl policy rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TtlPolicyRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TtlPolicyRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollup {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub weight: f64,
    pub retries: i64,
    pub source: String,
    pub reviewer: String,
    pub starts_at: i64,
    pub summary: String,
    pub checksum: String,
    pub archived: bool,
    pub status: TtlPolicyRollupStatus,
}

impl TtlPolicyRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyRollupStatus) -> bool {
        match self.status {
            TtlPolicyRollupStatus::Created => {
                match next {
                    TtlPolicyRollupStatus::Validated | TtlPolicyRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRollupStatus::Validated => {
                match next {
                    TtlPolicyRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRollupStatus::Published => {
                return next == TtlPolicyRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy rollup {} cannot move from {} to {}",
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
        format!("{} {}", "ttl policy rollup", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.retries as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyRollup {
        TtlPolicyRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            weight: self.weight,
            retries: self.retries,
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            starts_at: self.starts_at,
            summary: self.summary.clone(),
            checksum: self.checksum.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a ttl policy rollup in status created.
pub fn new_ttl_policy_rollup(code: &str, title: &str) -> TtlPolicyRollup {
    TtlPolicyRollup {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        weight: 0.0,
        retries: 0,
        source: "".to_string(),
        reviewer: "".to_string(),
        starts_at: 0,
        summary: "".to_string(),
        checksum: "".to_string(),
        archived: false,
        status: TtlPolicyRollupStatus::Created,
    }
}
