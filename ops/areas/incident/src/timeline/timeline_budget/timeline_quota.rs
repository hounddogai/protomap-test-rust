use super::*;

use acme_kit::text;

/// Where a timeline quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineQuotaStatus {
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

impl std::fmt::Display for TimelineQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline quota as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuota {
    pub code: String,
    pub name: String,
    pub host: String,
    pub rotation_days: i64,
    pub ttl_seconds: i64,
    pub archived: bool,
    pub region: String,
    pub batch_size: i64,
    pub active: bool,
    pub margin: f64,
    pub status: TimelineQuotaStatus,
}

impl TimelineQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineQuotaStatus) -> bool {
        match self.status {
            TimelineQuotaStatus::Planned => {
                return next == TimelineQuotaStatus::Scheduled || next == TimelineQuotaStatus::InProgress;
            }
            TimelineQuotaStatus::Scheduled => {
                match next {
                    TimelineQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineQuotaStatus::InProgress => {
                match next {
                    TimelineQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineQuotaStatus::Completed => {
                match next {
                    TimelineQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TimelineQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineQuota {
        TimelineQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            ttl_seconds: self.ttl_seconds,
            archived: self.archived,
            region: self.region.clone(),
            batch_size: self.batch_size,
            active: self.active,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a timeline quota in status planned.
pub fn new_timeline_quota(code: &str, name: &str) -> TimelineQuota {
    TimelineQuota {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        rotation_days: 0,
        ttl_seconds: 0,
        archived: false,
        region: "".to_string(),
        batch_size: 0,
        active: false,
        margin: 0.0,
        status: TimelineQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_timeline_quota("code-75", "name-513");
        assert_eq!(item.status, TimelineQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_timeline_quota("code-75", "name-513");
        if item.move_to(TimelineQuotaStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineQuotaStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_timeline_quota("code-75", "name-513");
        item.status = TimelineQuotaStatus::Cancelled;
        assert!(
            item.move_to(TimelineQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
