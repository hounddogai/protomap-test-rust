use super::*;

use acme_kit::text;

/// Where a primary limit override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryLimitOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PrimaryLimitOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary limit override as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverride {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub locale: String,
    pub ratio: f64,
    pub ttl_seconds: i64,
    pub maximum: f64,
    pub rotation_days: i64,
    pub status: PrimaryLimitOverrideStatus,
}

impl PrimaryLimitOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitOverrideStatus) -> bool {
        match self.status {
            PrimaryLimitOverrideStatus::Planned => {
                match next {
                    PrimaryLimitOverrideStatus::Completed | PrimaryLimitOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryLimitOverrideStatus::Scheduled => {
                return next == PrimaryLimitOverrideStatus::InProgress || next == PrimaryLimitOverrideStatus::Completed;
            }
            PrimaryLimitOverrideStatus::InProgress => {
                match next {
                    PrimaryLimitOverrideStatus::Completed | PrimaryLimitOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryLimitOverrideStatus::Completed => {
                return next == PrimaryLimitOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryLimitOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary limit override", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ratio - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryLimitOverride {
        PrimaryLimitOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            ratio: self.ratio,
            ttl_seconds: self.ttl_seconds,
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a primary limit override in status planned.
pub fn new_primary_limit_override(code: &str, name: &str) -> PrimaryLimitOverride {
    PrimaryLimitOverride {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        locale: "".to_string(),
        ratio: 0.0,
        ttl_seconds: 0,
        maximum: 0.0,
        rotation_days: 0,
        status: PrimaryLimitOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_limit_override("code-580", "name-532");
        assert_eq!(item.status, PrimaryLimitOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_primary_limit_override("code-580", "name-532");
        if item.move_to(PrimaryLimitOverrideStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryLimitOverrideStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_limit_override("code-580", "name-532");
        item.status = PrimaryLimitOverrideStatus::Cancelled;
        assert!(
            item.move_to(PrimaryLimitOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
