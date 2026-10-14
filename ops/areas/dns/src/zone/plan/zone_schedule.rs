use super::*;

use acme_kit::text;

/// Where a zone schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZoneScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZoneScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSchedule {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub region: String,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub priority: i64,
    pub ceiling: f64,
    pub checksum: String,
    pub budget: f64,
    pub status: ZoneScheduleStatus,
}

impl ZoneSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneScheduleStatus) -> bool {
        match self.status {
            ZoneScheduleStatus::Created => {
                return next == ZoneScheduleStatus::Retired;
            }
            ZoneScheduleStatus::Validated => {
                match next {
                    ZoneScheduleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneScheduleStatus::Published => {
                return next == ZoneScheduleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone schedule", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneSchedule {
        ZoneSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            priority: self.priority,
            ceiling: self.ceiling,
            checksum: self.checksum.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a zone schedule in status created.
pub fn new_zone_schedule(code: &str, title: &str) -> ZoneSchedule {
    ZoneSchedule {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        region: "".to_string(),
        archived: false,
        ttl_seconds: 0,
        priority: 0,
        ceiling: 0.0,
        checksum: "".to_string(),
        budget: 0.0,
        status: ZoneScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_zone_schedule("code-546", "title-105");
        assert_eq!(item.status, ZoneScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_zone_schedule("code-546", "title-105");
        if item.move_to(ZoneScheduleStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneScheduleStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_zone_schedule("code-546", "title-105");
        item.status = ZoneScheduleStatus::Retired;
        assert!(
            item.move_to(ZoneScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
