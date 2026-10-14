use super::*;

use acme_kit::text;

/// Where a legacy canary rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyCanaryRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyCanaryRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyCanaryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy canary rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollup {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub priority: i64,
    pub revision: i64,
    pub status: LegacyCanaryRollupStatus,
}

impl LegacyCanaryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyCanaryRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyCanaryRollupStatus) -> bool {
        match self.status {
            LegacyCanaryRollupStatus::Proposed => {
                match next {
                    LegacyCanaryRollupStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCanaryRollupStatus::Approved => {
                return next == LegacyCanaryRollupStatus::Rejected;
            }
            LegacyCanaryRollupStatus::Rejected => {
                match next {
                    LegacyCanaryRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyCanaryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy canary rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy canary rollup", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.priority as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyCanaryRollup {
        LegacyCanaryRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            priority: self.priority,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy canary rollup in status proposed.
pub fn new_legacy_canary_rollup(code: &str, title: &str) -> LegacyCanaryRollup {
    LegacyCanaryRollup {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        priority: 0,
        revision: 0,
        status: LegacyCanaryRollupStatus::Proposed,
    }
}
