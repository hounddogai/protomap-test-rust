use super::*;

use acme_kit::text;

/// Where a suspension schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SuspensionScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SuspensionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSchedule {
    pub code: String,
    pub title: String,
    pub country: String,
    pub quota_units: i64,
    pub approved: bool,
    pub minimum: f64,
    pub starts_at: i64,
    pub host: String,
    pub status: SuspensionScheduleStatus,
}

impl SuspensionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionScheduleStatus) -> bool {
        match self.status {
            SuspensionScheduleStatus::Proposed => {
                return next == SuspensionScheduleStatus::Withdrawn;
            }
            SuspensionScheduleStatus::Approved => {
                return next == SuspensionScheduleStatus::Rejected;
            }
            SuspensionScheduleStatus::Rejected => {
                match next {
                    SuspensionScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SuspensionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension schedule {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionSchedule {
        SuspensionSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            quota_units: self.quota_units,
            approved: self.approved,
            minimum: self.minimum,
            starts_at: self.starts_at,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a suspension schedule in status proposed.
pub fn new_suspension_schedule(code: &str, title: &str) -> SuspensionSchedule {
    SuspensionSchedule {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        quota_units: 0,
        approved: false,
        minimum: 0.0,
        starts_at: 0,
        host: "".to_string(),
        status: SuspensionScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_suspension_schedule("code-816", "title-347");
        assert_eq!(item.status, SuspensionScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_suspension_schedule("code-816", "title-347");
        if item.move_to(SuspensionScheduleStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionScheduleStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_suspension_schedule("code-816", "title-347");
        item.status = SuspensionScheduleStatus::Withdrawn;
        assert!(
            item.move_to(SuspensionScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
