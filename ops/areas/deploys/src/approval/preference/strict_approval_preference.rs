use super::*;

use acme_kit::text;

/// Where a strict approval preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictApprovalPreferenceStatus {
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

impl std::fmt::Display for StrictApprovalPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict approval preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreference {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub margin: f64,
    pub sequence: i64,
    pub checksum: String,
    pub status: StrictApprovalPreferenceStatus,
}

impl StrictApprovalPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalPreferenceStatus) -> bool {
        match self.status {
            StrictApprovalPreferenceStatus::Idle => {
                return next == StrictApprovalPreferenceStatus::Warming
                    || next == StrictApprovalPreferenceStatus::Ready;
            }
            StrictApprovalPreferenceStatus::Warming => {
                match next {
                    StrictApprovalPreferenceStatus::Ready | StrictApprovalPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalPreferenceStatus::Ready => {
                match next {
                    StrictApprovalPreferenceStatus::Draining | StrictApprovalPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalPreferenceStatus::Draining => {
                return next == StrictApprovalPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictApprovalPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict approval preference", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.port as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictApprovalPreference {
        StrictApprovalPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            margin: self.margin,
            sequence: self.sequence,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict approval preference in status idle.
pub fn new_strict_approval_preference(code: &str, name: &str) -> StrictApprovalPreference {
    StrictApprovalPreference {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        margin: 0.0,
        sequence: 0,
        checksum: "".to_string(),
        status: StrictApprovalPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_approval_preference("code-72", "name-358");
        assert_eq!(item.status, StrictApprovalPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_approval_preference("code-72", "name-358");
        if item.move_to(StrictApprovalPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_approval_preference("code-72", "name-358");
        item.status = StrictApprovalPreferenceStatus::Stopped;
        assert!(
            item.move_to(StrictApprovalPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
