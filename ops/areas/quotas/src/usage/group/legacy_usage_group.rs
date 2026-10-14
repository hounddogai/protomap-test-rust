use super::*;

use acme_kit::text;

/// Where a legacy usage group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyUsageGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyUsageGroupStatus {
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

impl std::fmt::Display for LegacyUsageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy usage group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageGroup {
    pub code: String,
    pub name: String,
    pub host: String,
    pub threshold: f64,
    pub batch_size: i64,
    pub retries: i64,
    pub unit: String,
    pub created_at: i64,
    pub status: LegacyUsageGroupStatus,
}

impl LegacyUsageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyUsageGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyUsageGroupStatus) -> bool {
        match self.status {
            LegacyUsageGroupStatus::Idle => {
                match next {
                    LegacyUsageGroupStatus::Ready | LegacyUsageGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsageGroupStatus::Warming => {
                match next {
                    LegacyUsageGroupStatus::Draining | LegacyUsageGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsageGroupStatus::Ready => {
                match next {
                    LegacyUsageGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsageGroupStatus::Draining => {
                match next {
                    LegacyUsageGroupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyUsageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy usage group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.threshold * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyUsageGroup {
        LegacyUsageGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            threshold: self.threshold,
            batch_size: self.batch_size,
            retries: self.retries,
            unit: self.unit.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy usage group in status idle.
pub fn new_legacy_usage_group(code: &str, name: &str) -> LegacyUsageGroup {
    LegacyUsageGroup {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        threshold: 0.0,
        batch_size: 0,
        retries: 0,
        unit: "".to_string(),
        created_at: 0,
        status: LegacyUsageGroupStatus::Idle,
    }
}
