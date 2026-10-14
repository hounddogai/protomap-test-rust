use super::*;

use acme_kit::text;

/// Where a timeline milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TimelineMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TimelineMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestone {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub duration_minutes: i64,
    pub notes: String,
    pub owner_id: String,
    pub segment: String,
    pub quota_units: i64,
    pub source: String,
    pub ends_at: i64,
    pub status: TimelineMilestoneStatus,
}

impl TimelineMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineMilestoneStatus) -> bool {
        match self.status {
            TimelineMilestoneStatus::Requested => {
                return next == TimelineMilestoneStatus::Granted;
            }
            TimelineMilestoneStatus::Granted => {
                return next == TimelineMilestoneStatus::Revoked || next == TimelineMilestoneStatus::Expired;
            }
            TimelineMilestoneStatus::Revoked => {
                return next == TimelineMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline milestone", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineMilestone {
        TimelineMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            duration_minutes: self.duration_minutes,
            notes: self.notes.clone(),
            owner_id: self.owner_id.clone(),
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            source: self.source.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a timeline milestone in status requested.
pub fn new_timeline_milestone(code: &str, title: &str) -> TimelineMilestone {
    TimelineMilestone {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        duration_minutes: 0,
        notes: "".to_string(),
        owner_id: "".to_string(),
        segment: "".to_string(),
        quota_units: 0,
        source: "".to_string(),
        ends_at: 0,
        status: TimelineMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_timeline_milestone("code-812", "title-514");
        assert_eq!(item.status, TimelineMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_timeline_milestone("code-812", "title-514");
        if item.move_to(TimelineMilestoneStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineMilestoneStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_timeline_milestone("code-812", "title-514");
        item.status = TimelineMilestoneStatus::Expired;
        assert!(
            item.move_to(TimelineMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
