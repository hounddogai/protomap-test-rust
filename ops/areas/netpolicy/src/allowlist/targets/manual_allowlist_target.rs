use super::*;

use acme_kit::text;

/// Where a manual allowlist target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAllowlistTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualAllowlistTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ManualAllowlistTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual allowlist target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistTarget {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub active: bool,
    pub window_days: i64,
    pub issuer: String,
    pub status: ManualAllowlistTargetStatus,
}

impl ManualAllowlistTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAllowlistTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAllowlistTargetStatus) -> bool {
        match self.status {
            ManualAllowlistTargetStatus::Planned => {
                return next == ManualAllowlistTargetStatus::InProgress
                    || next == ManualAllowlistTargetStatus::Completed;
            }
            ManualAllowlistTargetStatus::Scheduled => {
                return next == ManualAllowlistTargetStatus::Completed;
            }
            ManualAllowlistTargetStatus::InProgress => {
                match next {
                    ManualAllowlistTargetStatus::Completed | ManualAllowlistTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAllowlistTargetStatus::Completed => {
                return next == ManualAllowlistTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualAllowlistTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual allowlist target {} cannot move from {} to {}",
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
        self.window_days as f64
    }

    pub fn copy(&self) -> ManualAllowlistTarget {
        ManualAllowlistTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            active: self.active,
            window_days: self.window_days,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual allowlist target in status planned.
pub fn new_manual_allowlist_target(code: &str, title: &str) -> ManualAllowlistTarget {
    ManualAllowlistTarget {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        active: false,
        window_days: 0,
        issuer: "".to_string(),
        status: ManualAllowlistTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_allowlist_target("code-586", "title-985");
        assert_eq!(item.status, ManualAllowlistTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_manual_allowlist_target("code-586", "title-985");
        if item.move_to(ManualAllowlistTargetStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualAllowlistTargetStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_allowlist_target("code-586", "title-985");
        item.status = ManualAllowlistTargetStatus::Cancelled;
        assert!(
            item.move_to(ManualAllowlistTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
