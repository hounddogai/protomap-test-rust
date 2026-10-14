use super::*;

use acme_kit::text;

/// Where a regional retention schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRetentionScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalRetentionScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalRetentionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional retention schedule as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionSchedule {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub currency: String,
    pub grace_days: i64,
    pub cluster: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub unit: String,
    pub host: String,
    pub status: RegionalRetentionScheduleStatus,
}

impl RegionalRetentionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRetentionScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRetentionScheduleStatus) -> bool {
        match self.status {
            RegionalRetentionScheduleStatus::Proposed => {
                match next {
                    RegionalRetentionScheduleStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionScheduleStatus::Approved => {
                return next == RegionalRetentionScheduleStatus::Rejected
                    || next == RegionalRetentionScheduleStatus::Withdrawn;
            }
            RegionalRetentionScheduleStatus::Rejected => {
                return next == RegionalRetentionScheduleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRetentionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional retention schedule {} cannot move from {} to {}",
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
        format!("{} {}", "regional retention schedule", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.window_days as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRetentionSchedule {
        RegionalRetentionSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            currency: self.currency.clone(),
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            unit: self.unit.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a regional retention schedule in status proposed.
pub fn new_regional_retention_schedule(code: &str, title: &str) -> RegionalRetentionSchedule {
    RegionalRetentionSchedule {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        currency: "".to_string(),
        grace_days: 0,
        cluster: "".to_string(),
        duration_minutes: 0,
        window_days: 0,
        unit: "".to_string(),
        host: "".to_string(),
        status: RegionalRetentionScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_retention_schedule("code-714", "title-134");
        assert_eq!(item.status, RegionalRetentionScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_retention_schedule("code-714", "title-134");
        if item.move_to(RegionalRetentionScheduleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRetentionScheduleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_retention_schedule("code-714", "title-134");
        item.status = RegionalRetentionScheduleStatus::Withdrawn;
        assert!(
            item.move_to(RegionalRetentionScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
