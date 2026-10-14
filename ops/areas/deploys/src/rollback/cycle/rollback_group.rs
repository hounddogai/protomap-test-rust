use super::*;

use acme_kit::text;

/// Where a rollback group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollbackGroupStatus {
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

impl std::fmt::Display for RollbackGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroup {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub status: RollbackGroupStatus,
}

impl RollbackGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackGroupStatus) -> bool {
        match self.status {
            RollbackGroupStatus::Planned => {
                match next {
                    RollbackGroupStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackGroupStatus::Scheduled => {
                return next == RollbackGroupStatus::Cancelled;
            }
            RollbackGroupStatus::InProgress => {
                match next {
                    RollbackGroupStatus::Completed | RollbackGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackGroupStatus::Completed => {
                match next {
                    RollbackGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollbackGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.floor_value - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackGroup {
        RollbackGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollback group in status planned.
pub fn new_rollback_group(code: &str, name: &str) -> RollbackGroup {
    RollbackGroup {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        tags: Vec::new(),
        quota_units: 0,
        status: RollbackGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollback_group("code-438", "name-587");
        assert_eq!(item.status, RollbackGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rollback_group("code-438", "name-587");
        if item.move_to(RollbackGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollback_group("code-438", "name-587");
        item.status = RollbackGroupStatus::Cancelled;
        assert!(
            item.move_to(RollbackGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
