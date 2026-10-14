use super::*;

use acme_kit::text;

/// Where a default query review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultQueryReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultQueryReviewStatus {
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

impl std::fmt::Display for DefaultQueryReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default query review as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReview {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub summary: String,
    pub port: i64,
    pub status: DefaultQueryReviewStatus,
}

impl DefaultQueryReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultQueryReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultQueryReviewStatus) -> bool {
        match self.status {
            DefaultQueryReviewStatus::Idle => {
                return next == DefaultQueryReviewStatus::Stopped;
            }
            DefaultQueryReviewStatus::Warming => {
                match next {
                    DefaultQueryReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultQueryReviewStatus::Ready => {
                return next == DefaultQueryReviewStatus::Stopped;
            }
            DefaultQueryReviewStatus::Draining => {
                return next == DefaultQueryReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultQueryReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default query review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultQueryReview {
        DefaultQueryReview {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            summary: self.summary.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default query review in status idle.
pub fn new_default_query_review(code: &str, label: &str) -> DefaultQueryReview {
    DefaultQueryReview {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        summary: "".to_string(),
        port: 0,
        status: DefaultQueryReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_query_review("code-805", "label-10");
        assert_eq!(item.status, DefaultQueryReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_default_query_review("code-805", "label-10");
        if item.move_to(DefaultQueryReviewStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultQueryReviewStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_query_review("code-805", "label-10");
        item.status = DefaultQueryReviewStatus::Stopped;
        assert!(
            item.move_to(DefaultQueryReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
