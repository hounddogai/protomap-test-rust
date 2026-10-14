use super::*;

use acme_kit::text;

/// Where a manual approval preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualApprovalPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualApprovalPreferenceStatus {
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

impl std::fmt::Display for ManualApprovalPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual approval preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualApprovalPreference {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub quantity: i64,
    pub margin: f64,
    pub status: ManualApprovalPreferenceStatus,
}

impl ManualApprovalPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualApprovalPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualApprovalPreferenceStatus) -> bool {
        match self.status {
            ManualApprovalPreferenceStatus::Idle => {
                match next {
                    ManualApprovalPreferenceStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualApprovalPreferenceStatus::Warming => {
                return next == ManualApprovalPreferenceStatus::Ready;
            }
            ManualApprovalPreferenceStatus::Ready => {
                match next {
                    ManualApprovalPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualApprovalPreferenceStatus::Draining => {
                return next == ManualApprovalPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualApprovalPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual approval preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual approval preference", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quantity as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualApprovalPreference {
        ManualApprovalPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            quantity: self.quantity,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual approval preference in status idle.
pub fn new_manual_approval_preference(code: &str, title: &str) -> ManualApprovalPreference {
    ManualApprovalPreference {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        quantity: 0,
        margin: 0.0,
        status: ManualApprovalPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_approval_preference("code-554", "title-22");
        assert_eq!(item.status, ManualApprovalPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_approval_preference("code-554", "title-22");
        if item.move_to(ManualApprovalPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualApprovalPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_approval_preference("code-554", "title-22");
        item.status = ManualApprovalPreferenceStatus::Stopped;
        assert!(
            item.move_to(ManualApprovalPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
