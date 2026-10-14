use super::*;

use acme_kit::text;

/// Where a binding notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BindingNoticeStatus {
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

impl std::fmt::Display for BindingNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingNotice {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub burst: i64,
    pub margin: f64,
    pub notes: String,
    pub status: BindingNoticeStatus,
}

impl BindingNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingNoticeStatus) -> bool {
        match self.status {
            BindingNoticeStatus::New => {
                match next {
                    BindingNoticeStatus::Running | BindingNoticeStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingNoticeStatus::Queued => {
                match next {
                    BindingNoticeStatus::Running | BindingNoticeStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingNoticeStatus::Running => {
                return next == BindingNoticeStatus::Done || next == BindingNoticeStatus::Failed;
            }
            BindingNoticeStatus::Done => {
                return next == BindingNoticeStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.amount - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingNotice {
        BindingNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            burst: self.burst,
            margin: self.margin,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a binding notice in status new.
pub fn new_binding_notice(code: &str, name: &str) -> BindingNotice {
    BindingNotice {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        burst: 0,
        margin: 0.0,
        notes: "".to_string(),
        status: BindingNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_binding_notice("code-996", "name-475");
        assert_eq!(item.status, BindingNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_binding_notice("code-996", "name-475");
        if item.move_to(BindingNoticeStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingNoticeStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_binding_notice("code-996", "name-475");
        item.status = BindingNoticeStatus::Failed;
        assert!(item.move_to(BindingNoticeStatus::New).is_err(), "expected move to fail");
    }
}
