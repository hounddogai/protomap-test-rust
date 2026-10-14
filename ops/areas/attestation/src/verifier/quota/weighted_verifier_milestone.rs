use super::*;

use acme_kit::text;

/// Where a weighted verifier milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedVerifierMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedVerifierMilestoneStatus {
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

impl std::fmt::Display for WeightedVerifierMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted verifier milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedVerifierMilestone {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub country: String,
    pub ratio: f64,
    pub label: String,
    pub limit: i64,
    pub host: String,
    pub priority: i64,
    pub status: WeightedVerifierMilestoneStatus,
}

impl WeightedVerifierMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedVerifierMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedVerifierMilestoneStatus) -> bool {
        match self.status {
            WeightedVerifierMilestoneStatus::Idle => {
                match next {
                    WeightedVerifierMilestoneStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedVerifierMilestoneStatus::Warming => {
                match next {
                    WeightedVerifierMilestoneStatus::Ready | WeightedVerifierMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedVerifierMilestoneStatus::Ready => {
                return next == WeightedVerifierMilestoneStatus::Stopped;
            }
            WeightedVerifierMilestoneStatus::Draining => {
                return next == WeightedVerifierMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedVerifierMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted verifier milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.margin - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedVerifierMilestone {
        WeightedVerifierMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            country: self.country.clone(),
            ratio: self.ratio,
            label: self.label.clone(),
            limit: self.limit,
            host: self.host.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted verifier milestone in status idle.
pub fn new_weighted_verifier_milestone(code: &str, title: &str) -> WeightedVerifierMilestone {
    WeightedVerifierMilestone {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        country: "".to_string(),
        ratio: 0.0,
        label: "".to_string(),
        limit: 0,
        host: "".to_string(),
        priority: 0,
        status: WeightedVerifierMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_verifier_milestone("code-565", "title-486");
        assert_eq!(item.status, WeightedVerifierMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_verifier_milestone("code-565", "title-486");
        if item.move_to(WeightedVerifierMilestoneStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedVerifierMilestoneStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_verifier_milestone("code-565", "title-486");
        item.status = WeightedVerifierMilestoneStatus::Stopped;
        assert!(
            item.move_to(WeightedVerifierMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
