use super::*;

use acme_kit::text;

/// Where a scheduled credential window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledCredentialWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledCredentialWindowStatus {
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

impl std::fmt::Display for ScheduledCredentialWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled credential window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindow {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub rotation_days: i64,
    pub ceiling: f64,
    pub status: ScheduledCredentialWindowStatus,
}

impl ScheduledCredentialWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledCredentialWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledCredentialWindowStatus) -> bool {
        match self.status {
            ScheduledCredentialWindowStatus::Idle => {
                return next == ScheduledCredentialWindowStatus::Warming;
            }
            ScheduledCredentialWindowStatus::Warming => {
                match next {
                    ScheduledCredentialWindowStatus::Ready | ScheduledCredentialWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCredentialWindowStatus::Ready => {
                return next == ScheduledCredentialWindowStatus::Draining;
            }
            ScheduledCredentialWindowStatus::Draining => {
                match next {
                    ScheduledCredentialWindowStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledCredentialWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled credential window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled credential window", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        (self.ceiling * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledCredentialWindow {
        ScheduledCredentialWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            rotation_days: self.rotation_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled credential window in status idle.
pub fn new_scheduled_credential_window(code: &str, label: &str) -> ScheduledCredentialWindow {
    ScheduledCredentialWindow {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        rotation_days: 0,
        ceiling: 0.0,
        status: ScheduledCredentialWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_credential_window("code-188", "label-290");
        assert_eq!(item.status, ScheduledCredentialWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_credential_window("code-188", "label-290");
        if item.move_to(ScheduledCredentialWindowStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledCredentialWindowStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_credential_window("code-188", "label-290");
        item.status = ScheduledCredentialWindowStatus::Stopped;
        assert!(
            item.move_to(ScheduledCredentialWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
