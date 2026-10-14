use super::*;

use acme_kit::text;

/// Where a usage schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl UsageScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for UsageScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSchedule {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub phone: String,
    pub key_id: String,
    pub window_days: i64,
    pub category: String,
    pub threshold: f64,
    pub limit: i64,
    pub checksum: String,
    pub status: UsageScheduleStatus,
}

impl UsageSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageScheduleStatus) -> bool {
        match self.status {
            UsageScheduleStatus::Created => {
                return next == UsageScheduleStatus::Validated || next == UsageScheduleStatus::Published;
            }
            UsageScheduleStatus::Validated => {
                match next {
                    UsageScheduleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageScheduleStatus::Published => {
                return next == UsageScheduleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage schedule", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageSchedule {
        UsageSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            window_days: self.window_days,
            category: self.category.clone(),
            threshold: self.threshold,
            limit: self.limit,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a usage schedule in status created.
pub fn new_usage_schedule(code: &str, name: &str) -> UsageSchedule {
    UsageSchedule {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        phone: "".to_string(),
        key_id: "".to_string(),
        window_days: 0,
        category: "".to_string(),
        threshold: 0.0,
        limit: 0,
        checksum: "".to_string(),
        status: UsageScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_usage_schedule("code-60", "name-304");
        assert_eq!(item.status, UsageScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_usage_schedule("code-60", "name-304");
        if item.move_to(UsageScheduleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageScheduleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_usage_schedule("code-60", "name-304");
        item.status = UsageScheduleStatus::Retired;
        assert!(
            item.move_to(UsageScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
