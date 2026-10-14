use super::*;

use acme_kit::text;

/// Where a legacy usage target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyUsageTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyUsageTargetStatus {
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

impl std::fmt::Display for LegacyUsageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy usage target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTarget {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub contact_email: String,
    pub principal: String,
    pub rotation_days: i64,
    pub status: LegacyUsageTargetStatus,
}

impl LegacyUsageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyUsageTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyUsageTargetStatus) -> bool {
        match self.status {
            LegacyUsageTargetStatus::Idle => {
                return next == LegacyUsageTargetStatus::Warming;
            }
            LegacyUsageTargetStatus::Warming => {
                match next {
                    LegacyUsageTargetStatus::Draining | LegacyUsageTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsageTargetStatus::Ready => {
                return next == LegacyUsageTargetStatus::Stopped;
            }
            LegacyUsageTargetStatus::Draining => {
                match next {
                    LegacyUsageTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyUsageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy usage target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> LegacyUsageTarget {
        LegacyUsageTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            contact_email: self.contact_email.clone(),
            principal: self.principal.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy usage target in status idle.
pub fn new_legacy_usage_target(code: &str, title: &str) -> LegacyUsageTarget {
    LegacyUsageTarget {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        contact_email: "".to_string(),
        principal: "".to_string(),
        rotation_days: 0,
        status: LegacyUsageTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_usage_target("code-929", "title-166");
        assert_eq!(item.status, LegacyUsageTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_legacy_usage_target("code-929", "title-166");
        if item.move_to(LegacyUsageTargetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyUsageTargetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_usage_target("code-929", "title-166");
        item.status = LegacyUsageTargetStatus::Stopped;
        assert!(
            item.move_to(LegacyUsageTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
