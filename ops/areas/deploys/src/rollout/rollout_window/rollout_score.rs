use super::*;

use acme_kit::text;

/// Where a rollout score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutScoreStatus {
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

impl std::fmt::Display for RolloutScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutScore {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub reviewer: String,
    pub region: String,
    pub retries: i64,
    pub audience: String,
    pub status: RolloutScoreStatus,
}

impl RolloutScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutScoreStatus) -> bool {
        match self.status {
            RolloutScoreStatus::New => {
                return next == RolloutScoreStatus::Queued || next == RolloutScoreStatus::Done;
            }
            RolloutScoreStatus::Queued => {
                return next == RolloutScoreStatus::Done || next == RolloutScoreStatus::Failed;
            }
            RolloutScoreStatus::Running => {
                match next {
                    RolloutScoreStatus::Done | RolloutScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutScoreStatus::Done => {
                match next {
                    RolloutScoreStatus::Failed => {
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

    pub fn move_to(&mut self, next: RolloutScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout score", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutScore {
        RolloutScore {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            reviewer: self.reviewer.clone(),
            region: self.region.clone(),
            retries: self.retries,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rollout score in status new.
pub fn new_rollout_score(code: &str, title: &str) -> RolloutScore {
    RolloutScore {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        reviewer: "".to_string(),
        region: "".to_string(),
        retries: 0,
        audience: "".to_string(),
        status: RolloutScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollout_score("code-785", "title-294");
        assert_eq!(item.status, RolloutScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollout_score("code-785", "title-294");
        if item.move_to(RolloutScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollout_score("code-785", "title-294");
        item.status = RolloutScoreStatus::Failed;
        assert!(item.move_to(RolloutScoreStatus::New).is_err(), "expected move to fail");
    }
}
