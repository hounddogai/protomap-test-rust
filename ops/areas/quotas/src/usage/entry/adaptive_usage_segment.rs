use super::*;

use acme_kit::text;

/// Where a adaptive usage segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveUsageSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveUsageSegmentStatus {
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

impl std::fmt::Display for AdaptiveUsageSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive usage segment as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageSegment {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub ends_at: i64,
    pub scope: String,
    pub status: AdaptiveUsageSegmentStatus,
}

impl AdaptiveUsageSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveUsageSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveUsageSegmentStatus) -> bool {
        match self.status {
            AdaptiveUsageSegmentStatus::Planned => {
                match next {
                    AdaptiveUsageSegmentStatus::Scheduled | AdaptiveUsageSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveUsageSegmentStatus::Scheduled => {
                return next == AdaptiveUsageSegmentStatus::InProgress || next == AdaptiveUsageSegmentStatus::Completed;
            }
            AdaptiveUsageSegmentStatus::InProgress => {
                return next == AdaptiveUsageSegmentStatus::Completed;
            }
            AdaptiveUsageSegmentStatus::Completed => {
                return next == AdaptiveUsageSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveUsageSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive usage segment {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveUsageSegment {
        AdaptiveUsageSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            ends_at: self.ends_at,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive usage segment in status planned.
pub fn new_adaptive_usage_segment(code: &str, name: &str) -> AdaptiveUsageSegment {
    AdaptiveUsageSegment {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        ends_at: 0,
        scope: "".to_string(),
        status: AdaptiveUsageSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_usage_segment("code-496", "name-342");
        assert_eq!(item.status, AdaptiveUsageSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_adaptive_usage_segment("code-496", "name-342");
        if item.move_to(AdaptiveUsageSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveUsageSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_usage_segment("code-496", "name-342");
        item.status = AdaptiveUsageSegmentStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveUsageSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
