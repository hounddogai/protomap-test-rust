use super::*;

use acme_kit::text;

/// Where a adaptive sink milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSinkMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveSinkMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveSinkMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive sink milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkMilestone {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub cipher: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub status: AdaptiveSinkMilestoneStatus,
}

impl AdaptiveSinkMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSinkMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSinkMilestoneStatus) -> bool {
        match self.status {
            AdaptiveSinkMilestoneStatus::New => {
                return next == AdaptiveSinkMilestoneStatus::Running;
            }
            AdaptiveSinkMilestoneStatus::Queued => {
                match next {
                    AdaptiveSinkMilestoneStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSinkMilestoneStatus::Running => {
                return next == AdaptiveSinkMilestoneStatus::Done || next == AdaptiveSinkMilestoneStatus::Failed;
            }
            AdaptiveSinkMilestoneStatus::Done => {
                match next {
                    AdaptiveSinkMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveSinkMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive sink milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive sink milestone", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.threshold + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSinkMilestone {
        AdaptiveSinkMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            cipher: self.cipher.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive sink milestone in status new.
pub fn new_adaptive_sink_milestone(code: &str, name: &str) -> AdaptiveSinkMilestone {
    AdaptiveSinkMilestone {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        cipher: "".to_string(),
        duration_minutes: 0,
        window_days: 0,
        status: AdaptiveSinkMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_sink_milestone("code-331", "name-952");
        assert_eq!(item.status, AdaptiveSinkMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_adaptive_sink_milestone("code-331", "name-952");
        if item.move_to(AdaptiveSinkMilestoneStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSinkMilestoneStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_sink_milestone("code-331", "name-952");
        item.status = AdaptiveSinkMilestoneStatus::Failed;
        assert!(
            item.move_to(AdaptiveSinkMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
