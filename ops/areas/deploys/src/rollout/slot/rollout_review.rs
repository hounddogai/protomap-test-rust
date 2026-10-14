use super::*;

use acme_kit::text;

/// Where a rollout review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutReviewStatus {
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

impl std::fmt::Display for RolloutReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout review as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReview {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub score: f64,
    pub margin: f64,
    pub cipher: String,
    pub host: String,
    pub region: String,
    pub status: RolloutReviewStatus,
}

impl RolloutReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutReviewStatus) -> bool {
        match self.status {
            RolloutReviewStatus::New => {
                match next {
                    RolloutReviewStatus::Queued | RolloutReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutReviewStatus::Queued => {
                match next {
                    RolloutReviewStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutReviewStatus::Running => {
                match next {
                    RolloutReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutReviewStatus::Done => {
                match next {
                    RolloutReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: RolloutReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.margin + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutReview {
        RolloutReview {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            score: self.score,
            margin: self.margin,
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rollout review in status new.
pub fn new_rollout_review(code: &str, title: &str) -> RolloutReview {
    RolloutReview {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        score: 0.0,
        margin: 0.0,
        cipher: "".to_string(),
        host: "".to_string(),
        region: "".to_string(),
        status: RolloutReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollout_review("code-682", "title-536");
        assert_eq!(item.status, RolloutReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollout_review("code-682", "title-536");
        if item.move_to(RolloutReviewStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutReviewStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollout_review("code-682", "title-536");
        item.status = RolloutReviewStatus::Failed;
        assert!(item.move_to(RolloutReviewStatus::New).is_err(), "expected move to fail");
    }
}
