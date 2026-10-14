use super::*;

use acme_kit::text;

/// Where a weighted zone profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedZoneProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedZoneProfileStatus {
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

impl std::fmt::Display for WeightedZoneProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted zone profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneProfile {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub burst: i64,
    pub duration_minutes: i64,
    pub pinned: bool,
    pub description: String,
    pub status: WeightedZoneProfileStatus,
}

impl WeightedZoneProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedZoneProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedZoneProfileStatus) -> bool {
        match self.status {
            WeightedZoneProfileStatus::Planned => {
                return next == WeightedZoneProfileStatus::Scheduled || next == WeightedZoneProfileStatus::InProgress;
            }
            WeightedZoneProfileStatus::Scheduled => {
                return next == WeightedZoneProfileStatus::InProgress || next == WeightedZoneProfileStatus::Completed;
            }
            WeightedZoneProfileStatus::InProgress => {
                match next {
                    WeightedZoneProfileStatus::Completed | WeightedZoneProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedZoneProfileStatus::Completed => {
                return next == WeightedZoneProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedZoneProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted zone profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedZoneProfile {
        WeightedZoneProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            burst: self.burst,
            duration_minutes: self.duration_minutes,
            pinned: self.pinned,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted zone profile in status planned.
pub fn new_weighted_zone_profile(code: &str, title: &str) -> WeightedZoneProfile {
    WeightedZoneProfile {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        burst: 0,
        duration_minutes: 0,
        pinned: false,
        description: "".to_string(),
        status: WeightedZoneProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_zone_profile("code-493", "title-719");
        assert_eq!(item.status, WeightedZoneProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_weighted_zone_profile("code-493", "title-719");
        if item.move_to(WeightedZoneProfileStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedZoneProfileStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_zone_profile("code-493", "title-719");
        item.status = WeightedZoneProfileStatus::Cancelled;
        assert!(
            item.move_to(WeightedZoneProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
