use super::*;

use acme_kit::text;

/// Where a legacy federation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFederationScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyFederationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyFederationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy federation schedule as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationSchedule {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub checksum: String,
    pub version: i64,
    pub budget: f64,
    pub ratio: f64,
    pub status: LegacyFederationScheduleStatus,
}

impl LegacyFederationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFederationScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFederationScheduleStatus) -> bool {
        match self.status {
            LegacyFederationScheduleStatus::Created => {
                match next {
                    LegacyFederationScheduleStatus::Published | LegacyFederationScheduleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationScheduleStatus::Validated => {
                match next {
                    LegacyFederationScheduleStatus::Published | LegacyFederationScheduleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationScheduleStatus::Published => {
                return next == LegacyFederationScheduleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFederationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy federation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        (self.ratio * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFederationSchedule {
        LegacyFederationSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            version: self.version,
            budget: self.budget,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy federation schedule in status created.
pub fn new_legacy_federation_schedule(code: &str, title: &str) -> LegacyFederationSchedule {
    LegacyFederationSchedule {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        checksum: "".to_string(),
        version: 0,
        budget: 0.0,
        ratio: 0.0,
        status: LegacyFederationScheduleStatus::Created,
    }
}
