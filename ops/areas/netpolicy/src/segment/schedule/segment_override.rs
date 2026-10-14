use super::*;

use acme_kit::text;

/// Where a segment override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SegmentOverrideStatus {
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

impl std::fmt::Display for SegmentOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentOverride {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub reviewer: String,
    pub floor_value: f64,
    pub archived: bool,
    pub budget: f64,
    pub quantity: i64,
    pub weight: f64,
    pub threshold: f64,
    pub grace_days: i64,
    pub status: SegmentOverrideStatus,
}

impl SegmentOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentOverrideStatus) -> bool {
        match self.status {
            SegmentOverrideStatus::Planned => {
                return next == SegmentOverrideStatus::Scheduled || next == SegmentOverrideStatus::Completed;
            }
            SegmentOverrideStatus::Scheduled => {
                return next == SegmentOverrideStatus::InProgress || next == SegmentOverrideStatus::Completed;
            }
            SegmentOverrideStatus::InProgress => {
                match next {
                    SegmentOverrideStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentOverrideStatus::Completed => {
                return next == SegmentOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quantity as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentOverride {
        SegmentOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            reviewer: self.reviewer.clone(),
            floor_value: self.floor_value,
            archived: self.archived,
            budget: self.budget,
            quantity: self.quantity,
            weight: self.weight,
            threshold: self.threshold,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment override in status planned.
pub fn new_segment_override(code: &str, title: &str) -> SegmentOverride {
    SegmentOverride {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        reviewer: "".to_string(),
        floor_value: 0.0,
        archived: false,
        budget: 0.0,
        quantity: 0,
        weight: 0.0,
        threshold: 0.0,
        grace_days: 0,
        status: SegmentOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_segment_override("code-111", "title-49");
        assert_eq!(item.status, SegmentOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_segment_override("code-111", "title-49");
        if item.move_to(SegmentOverrideStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentOverrideStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_segment_override("code-111", "title-49");
        item.status = SegmentOverrideStatus::Cancelled;
        assert!(
            item.move_to(SegmentOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
