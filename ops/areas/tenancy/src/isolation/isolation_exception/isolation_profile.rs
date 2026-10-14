use super::*;

use acme_kit::text;

/// Where a isolation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationProfileStatus {
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

impl std::fmt::Display for IsolationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfile {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub floor_value: f64,
    pub fingerprint: String,
    pub priority: i64,
    pub ceiling: f64,
    pub name: String,
    pub score: f64,
    pub description: String,
    pub status: IsolationProfileStatus,
}

impl IsolationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationProfileStatus) -> bool {
        match self.status {
            IsolationProfileStatus::Planned => {
                return next == IsolationProfileStatus::Scheduled;
            }
            IsolationProfileStatus::Scheduled => {
                return next == IsolationProfileStatus::InProgress || next == IsolationProfileStatus::Cancelled;
            }
            IsolationProfileStatus::InProgress => {
                return next == IsolationProfileStatus::Completed || next == IsolationProfileStatus::Cancelled;
            }
            IsolationProfileStatus::Completed => {
                return next == IsolationProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.floor_value - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationProfile {
        IsolationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            priority: self.priority,
            ceiling: self.ceiling,
            name: self.name.clone(),
            score: self.score,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a isolation profile in status planned.
pub fn new_isolation_profile(code: &str, title: &str) -> IsolationProfile {
    IsolationProfile {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        floor_value: 0.0,
        fingerprint: "".to_string(),
        priority: 0,
        ceiling: 0.0,
        name: "".to_string(),
        score: 0.0,
        description: "".to_string(),
        status: IsolationProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_profile("code-845", "title-444");
        assert_eq!(item.status, IsolationProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_isolation_profile("code-845", "title-444");
        if item.move_to(IsolationProfileStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationProfileStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_profile("code-845", "title-444");
        item.status = IsolationProfileStatus::Cancelled;
        assert!(
            item.move_to(IsolationProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
