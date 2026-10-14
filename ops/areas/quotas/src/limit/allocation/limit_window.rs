use super::*;

use acme_kit::text;

/// Where a limit window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LimitWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LimitWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a limit window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindow {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub host: String,
    pub channel: String,
    pub timezone: String,
    pub batch_size: i64,
    pub principal: String,
    pub status: LimitWindowStatus,
}

impl LimitWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitWindowStatus) -> bool {
        match self.status {
            LimitWindowStatus::Open => {
                return next == LimitWindowStatus::Resolved;
            }
            LimitWindowStatus::InReview => {
                return next == LimitWindowStatus::Archived;
            }
            LimitWindowStatus::Resolved => {
                return next == LimitWindowStatus::Reopened;
            }
            LimitWindowStatus::Reopened => {
                match next {
                    LimitWindowStatus::Archived => {
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

    pub fn move_to(&mut self, next: LimitWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> LimitWindow {
        LimitWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            host: self.host.clone(),
            channel: self.channel.clone(),
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a limit window in status open.
pub fn new_limit_window(code: &str, name: &str) -> LimitWindow {
    LimitWindow {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        host: "".to_string(),
        channel: "".to_string(),
        timezone: "".to_string(),
        batch_size: 0,
        principal: "".to_string(),
        status: LimitWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_limit_window("code-848", "name-796");
        assert_eq!(item.status, LimitWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_limit_window("code-848", "name-796");
        if item.move_to(LimitWindowStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitWindowStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_limit_window("code-848", "name-796");
        item.status = LimitWindowStatus::Archived;
        assert!(item.move_to(LimitWindowStatus::Open).is_err(), "expected move to fail");
    }
}
