use super::*;

use acme_kit::text;

/// Where a scheduled postmortem estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPostmortemEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledPostmortemEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledPostmortemEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled postmortem estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimate {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub batch_size: i64,
    pub reviewer: String,
    pub status: ScheduledPostmortemEstimateStatus,
}

impl ScheduledPostmortemEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPostmortemEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPostmortemEstimateStatus) -> bool {
        match self.status {
            ScheduledPostmortemEstimateStatus::Idle => {
                return next == ScheduledPostmortemEstimateStatus::Warming;
            }
            ScheduledPostmortemEstimateStatus::Warming => {
                return next == ScheduledPostmortemEstimateStatus::Stopped;
            }
            ScheduledPostmortemEstimateStatus::Ready => {
                match next {
                    ScheduledPostmortemEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPostmortemEstimateStatus::Draining => {
                return next == ScheduledPostmortemEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledPostmortemEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled postmortem estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> ScheduledPostmortemEstimate {
        ScheduledPostmortemEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled postmortem estimate in status idle.
pub fn new_scheduled_postmortem_estimate(code: &str, title: &str) -> ScheduledPostmortemEstimate {
    ScheduledPostmortemEstimate {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        batch_size: 0,
        reviewer: "".to_string(),
        status: ScheduledPostmortemEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_postmortem_estimate("code-810", "title-646");
        assert_eq!(item.status, ScheduledPostmortemEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_postmortem_estimate("code-810", "title-646");
        if item.move_to(ScheduledPostmortemEstimateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPostmortemEstimateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_postmortem_estimate("code-810", "title-646");
        item.status = ScheduledPostmortemEstimateStatus::Stopped;
        assert!(
            item.move_to(ScheduledPostmortemEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
