use super::*;

use acme_kit::text;

/// Where a manual zone override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualZoneOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualZoneOverrideStatus {
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

impl std::fmt::Display for ManualZoneOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual zone override as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualZoneOverride {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub burst: i64,
    pub port: i64,
    pub description: String,
    pub archived: bool,
    pub floor_value: f64,
    pub region: String,
    pub grace_days: i64,
    pub audience: String,
    pub status: ManualZoneOverrideStatus,
}

impl ManualZoneOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualZoneOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualZoneOverrideStatus) -> bool {
        match self.status {
            ManualZoneOverrideStatus::Planned => {
                return next == ManualZoneOverrideStatus::InProgress;
            }
            ManualZoneOverrideStatus::Scheduled => {
                return next == ManualZoneOverrideStatus::Cancelled;
            }
            ManualZoneOverrideStatus::InProgress => {
                return next == ManualZoneOverrideStatus::Completed || next == ManualZoneOverrideStatus::Cancelled;
            }
            ManualZoneOverrideStatus::Completed => {
                match next {
                    ManualZoneOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ManualZoneOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual zone override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual zone override", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.grace_days as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualZoneOverride {
        ManualZoneOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            burst: self.burst,
            port: self.port,
            description: self.description.clone(),
            archived: self.archived,
            floor_value: self.floor_value,
            region: self.region.clone(),
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual zone override in status planned.
pub fn new_manual_zone_override(code: &str, name: &str) -> ManualZoneOverride {
    ManualZoneOverride {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        burst: 0,
        port: 0,
        description: "".to_string(),
        archived: false,
        floor_value: 0.0,
        region: "".to_string(),
        grace_days: 0,
        audience: "".to_string(),
        status: ManualZoneOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_zone_override("code-649", "name-7");
        assert_eq!(item.status, ManualZoneOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_manual_zone_override("code-649", "name-7");
        if item.move_to(ManualZoneOverrideStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualZoneOverrideStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_zone_override("code-649", "name-7");
        item.status = ManualZoneOverrideStatus::Cancelled;
        assert!(
            item.move_to(ManualZoneOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
