use super::*;

use acme_kit::text;

/// Where a postmortem target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemTargetStatus {
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

impl std::fmt::Display for PostmortemTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem target as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTarget {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub score: f64,
    pub category: String,
    pub grace_days: i64,
    pub phone: String,
    pub status: PostmortemTargetStatus,
}

impl PostmortemTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemTargetStatus) -> bool {
        match self.status {
            PostmortemTargetStatus::Idle => {
                return next == PostmortemTargetStatus::Stopped;
            }
            PostmortemTargetStatus::Warming => {
                return next == PostmortemTargetStatus::Ready || next == PostmortemTargetStatus::Stopped;
            }
            PostmortemTargetStatus::Ready => {
                return next == PostmortemTargetStatus::Draining || next == PostmortemTargetStatus::Stopped;
            }
            PostmortemTargetStatus::Draining => {
                match next {
                    PostmortemTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PostmortemTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.score - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemTarget {
        PostmortemTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            score: self.score,
            category: self.category.clone(),
            grace_days: self.grace_days,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a postmortem target in status idle.
pub fn new_postmortem_target(code: &str, label: &str) -> PostmortemTarget {
    PostmortemTarget {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        score: 0.0,
        category: "".to_string(),
        grace_days: 0,
        phone: "".to_string(),
        status: PostmortemTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_postmortem_target("code-697", "label-878");
        assert_eq!(item.status, PostmortemTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_postmortem_target("code-697", "label-878");
        if item.move_to(PostmortemTargetStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemTargetStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_postmortem_target("code-697", "label-878");
        item.status = PostmortemTargetStatus::Stopped;
        assert!(
            item.move_to(PostmortemTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
