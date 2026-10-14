use super::*;

use acme_kit::text;

/// Where a scheduled query review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQueryReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledQueryReviewStatus {
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

impl std::fmt::Display for ScheduledQueryReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled query review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReview {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub active: bool,
    pub floor_value: f64,
    pub capacity: i64,
    pub rate: f64,
    pub updated_at: i64,
    pub status: ScheduledQueryReviewStatus,
}

impl ScheduledQueryReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQueryReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQueryReviewStatus) -> bool {
        match self.status {
            ScheduledQueryReviewStatus::Idle => {
                return next == ScheduledQueryReviewStatus::Stopped;
            }
            ScheduledQueryReviewStatus::Warming => {
                return next == ScheduledQueryReviewStatus::Ready || next == ScheduledQueryReviewStatus::Draining;
            }
            ScheduledQueryReviewStatus::Ready => {
                match next {
                    ScheduledQueryReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryReviewStatus::Draining => {
                match next {
                    ScheduledQueryReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledQueryReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled query review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled query review", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rate - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQueryReview {
        ScheduledQueryReview {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            active: self.active,
            floor_value: self.floor_value,
            capacity: self.capacity,
            rate: self.rate,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled query review in status idle.
pub fn new_scheduled_query_review(code: &str, title: &str) -> ScheduledQueryReview {
    ScheduledQueryReview {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        active: false,
        floor_value: 0.0,
        capacity: 0,
        rate: 0.0,
        updated_at: 0,
        status: ScheduledQueryReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_query_review("code-903", "title-426");
        assert_eq!(item.status, ScheduledQueryReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_scheduled_query_review("code-903", "title-426");
        if item.move_to(ScheduledQueryReviewStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQueryReviewStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_query_review("code-903", "title-426");
        item.status = ScheduledQueryReviewStatus::Stopped;
        assert!(
            item.move_to(ScheduledQueryReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
