use super::*;

use acme_kit::text;

/// Where a global health check segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalHealthCheckSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalHealthCheckSegmentStatus {
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

impl std::fmt::Display for GlobalHealthCheckSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global health check segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegment {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub currency: String,
    pub active: bool,
    pub summary: String,
    pub timezone: String,
    pub tags: Vec<String>,
    pub status: GlobalHealthCheckSegmentStatus,
}

impl GlobalHealthCheckSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalHealthCheckSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalHealthCheckSegmentStatus) -> bool {
        match self.status {
            GlobalHealthCheckSegmentStatus::Planned => {
                return next == GlobalHealthCheckSegmentStatus::Scheduled
                    || next == GlobalHealthCheckSegmentStatus::InProgress;
            }
            GlobalHealthCheckSegmentStatus::Scheduled => {
                match next {
                    GlobalHealthCheckSegmentStatus::InProgress | GlobalHealthCheckSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalHealthCheckSegmentStatus::InProgress => {
                match next {
                    GlobalHealthCheckSegmentStatus::Completed | GlobalHealthCheckSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalHealthCheckSegmentStatus::Completed => {
                return next == GlobalHealthCheckSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalHealthCheckSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global health check segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global health check segment", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.margin + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalHealthCheckSegment {
        GlobalHealthCheckSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            currency: self.currency.clone(),
            active: self.active,
            summary: self.summary.clone(),
            timezone: self.timezone.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global health check segment in status planned.
pub fn new_global_health_check_segment(code: &str, name: &str) -> GlobalHealthCheckSegment {
    GlobalHealthCheckSegment {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        ttl_seconds: 0,
        margin: 0.0,
        currency: "".to_string(),
        active: false,
        summary: "".to_string(),
        timezone: "".to_string(),
        tags: Vec::new(),
        status: GlobalHealthCheckSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_health_check_segment("code-192", "name-27");
        assert_eq!(item.status, GlobalHealthCheckSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_health_check_segment("code-192", "name-27");
        if item.move_to(GlobalHealthCheckSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalHealthCheckSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_health_check_segment("code-192", "name-27");
        item.status = GlobalHealthCheckSegmentStatus::Cancelled;
        assert!(
            item.move_to(GlobalHealthCheckSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
