use super::*;

use acme_kit::text;

/// Where a retention override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionOverrideStatus {
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

impl std::fmt::Display for RetentionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverride {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub tags: Vec<String>,
    pub archived: bool,
    pub fingerprint: String,
    pub active: bool,
    pub status: RetentionOverrideStatus,
}

impl RetentionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionOverrideStatus) -> bool {
        match self.status {
            RetentionOverrideStatus::Idle => {
                return next == RetentionOverrideStatus::Ready || next == RetentionOverrideStatus::Draining;
            }
            RetentionOverrideStatus::Warming => {
                match next {
                    RetentionOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionOverrideStatus::Ready => {
                return next == RetentionOverrideStatus::Draining || next == RetentionOverrideStatus::Stopped;
            }
            RetentionOverrideStatus::Draining => {
                match next {
                    RetentionOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RetentionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> RetentionOverride {
        RetentionOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            tags: self.tags.clone(),
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a retention override in status idle.
pub fn new_retention_override(code: &str, name: &str) -> RetentionOverride {
    RetentionOverride {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        tags: Vec::new(),
        archived: false,
        fingerprint: "".to_string(),
        active: false,
        status: RetentionOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_override("code-151", "name-983");
        assert_eq!(item.status, RetentionOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_retention_override("code-151", "name-983");
        if item.move_to(RetentionOverrideStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionOverrideStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_override("code-151", "name-983");
        item.status = RetentionOverrideStatus::Stopped;
        assert!(
            item.move_to(RetentionOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
