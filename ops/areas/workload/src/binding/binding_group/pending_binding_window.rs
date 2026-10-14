use super::*;

use acme_kit::text;

/// Where a pending binding window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingBindingWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingBindingWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PendingBindingWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending binding window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingWindow {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub rotation_days: i64,
    pub issuer: String,
    pub host: String,
    pub status: PendingBindingWindowStatus,
}

impl PendingBindingWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingBindingWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingBindingWindowStatus) -> bool {
        match self.status {
            PendingBindingWindowStatus::New => {
                return next == PendingBindingWindowStatus::Running;
            }
            PendingBindingWindowStatus::Queued => {
                match next {
                    PendingBindingWindowStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBindingWindowStatus::Running => {
                return next == PendingBindingWindowStatus::Failed;
            }
            PendingBindingWindowStatus::Done => {
                match next {
                    PendingBindingWindowStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingBindingWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending binding window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> PendingBindingWindow {
        PendingBindingWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending binding window in status new.
pub fn new_pending_binding_window(code: &str, name: &str) -> PendingBindingWindow {
    PendingBindingWindow {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        rotation_days: 0,
        issuer: "".to_string(),
        host: "".to_string(),
        status: PendingBindingWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_binding_window("code-836", "name-770");
        assert_eq!(item.status, PendingBindingWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_pending_binding_window("code-836", "name-770");
        if item.move_to(PendingBindingWindowStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingBindingWindowStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_binding_window("code-836", "name-770");
        item.status = PendingBindingWindowStatus::Failed;
        assert!(
            item.move_to(PendingBindingWindowStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
