use super::*;

use acme_kit::text;

/// Where a postmortem override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemOverrideStatus {
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

impl std::fmt::Display for PostmortemOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemOverride {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub limit: i64,
    pub timezone: String,
    pub revision: i64,
    pub cluster: String,
    pub status: PostmortemOverrideStatus,
}

impl PostmortemOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemOverrideStatus) -> bool {
        match self.status {
            PostmortemOverrideStatus::Idle => {
                match next {
                    PostmortemOverrideStatus::Ready | PostmortemOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemOverrideStatus::Warming => {
                match next {
                    PostmortemOverrideStatus::Draining | PostmortemOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemOverrideStatus::Ready => {
                match next {
                    PostmortemOverrideStatus::Draining | PostmortemOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemOverrideStatus::Draining => {
                match next {
                    PostmortemOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PostmortemOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem override", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemOverride {
        PostmortemOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            limit: self.limit,
            timezone: self.timezone.clone(),
            revision: self.revision,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a postmortem override in status idle.
pub fn new_postmortem_override(code: &str, label: &str) -> PostmortemOverride {
    PostmortemOverride {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        limit: 0,
        timezone: "".to_string(),
        revision: 0,
        cluster: "".to_string(),
        status: PostmortemOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_postmortem_override("code-361", "label-759");
        assert_eq!(item.status, PostmortemOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_postmortem_override("code-361", "label-759");
        if item.move_to(PostmortemOverrideStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemOverrideStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_postmortem_override("code-361", "label-759");
        item.status = PostmortemOverrideStatus::Stopped;
        assert!(
            item.move_to(PostmortemOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
