use super::*;

use acme_kit::text;

/// Where a strict federation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFederationScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictFederationScheduleStatus {
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

impl std::fmt::Display for StrictFederationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict federation schedule as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationSchedule {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub channel: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub archived: bool,
    pub budget: f64,
    pub discount: f64,
    pub grace_days: i64,
    pub status: StrictFederationScheduleStatus,
}

impl StrictFederationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFederationScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFederationScheduleStatus) -> bool {
        match self.status {
            StrictFederationScheduleStatus::Planned => {
                return next == StrictFederationScheduleStatus::InProgress
                    || next == StrictFederationScheduleStatus::Completed;
            }
            StrictFederationScheduleStatus::Scheduled => {
                match next {
                    StrictFederationScheduleStatus::InProgress | StrictFederationScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFederationScheduleStatus::InProgress => {
                match next {
                    StrictFederationScheduleStatus::Completed | StrictFederationScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFederationScheduleStatus::Completed => {
                match next {
                    StrictFederationScheduleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: StrictFederationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict federation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict federation schedule", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFederationSchedule {
        StrictFederationSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            channel: self.channel.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            archived: self.archived,
            budget: self.budget,
            discount: self.discount,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a strict federation schedule in status planned.
pub fn new_strict_federation_schedule(code: &str, name: &str) -> StrictFederationSchedule {
    StrictFederationSchedule {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        channel: "".to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        archived: false,
        budget: 0.0,
        discount: 0.0,
        grace_days: 0,
        status: StrictFederationScheduleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_federation_schedule("code-246", "name-599");
        assert_eq!(item.status, StrictFederationScheduleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_strict_federation_schedule("code-246", "name-599");
        if item.move_to(StrictFederationScheduleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFederationScheduleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_federation_schedule("code-246", "name-599");
        item.status = StrictFederationScheduleStatus::Cancelled;
        assert!(
            item.move_to(StrictFederationScheduleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
