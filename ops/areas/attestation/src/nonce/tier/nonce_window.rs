use super::*;

use acme_kit::text;

/// Where a nonce window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl NonceWindowStatus {
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

impl std::fmt::Display for NonceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceWindow {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub active: bool,
    pub score: f64,
    pub attempts: i64,
    pub summary: String,
    pub title: String,
    pub port: i64,
    pub locale: String,
    pub status: NonceWindowStatus,
}

impl NonceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceWindowStatus) -> bool {
        match self.status {
            NonceWindowStatus::New => {
                match next {
                    NonceWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceWindowStatus::Queued => {
                return next == NonceWindowStatus::Failed;
            }
            NonceWindowStatus::Running => {
                match next {
                    NonceWindowStatus::Done | NonceWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceWindowStatus::Done => {
                return next == NonceWindowStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.margin - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceWindow {
        NonceWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            active: self.active,
            score: self.score,
            attempts: self.attempts,
            summary: self.summary.clone(),
            title: self.title.clone(),
            port: self.port,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a nonce window in status new.
pub fn new_nonce_window(code: &str, label: &str) -> NonceWindow {
    NonceWindow {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        active: false,
        score: 0.0,
        attempts: 0,
        summary: "".to_string(),
        title: "".to_string(),
        port: 0,
        locale: "".to_string(),
        status: NonceWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_nonce_window("code-487", "label-628");
        assert_eq!(item.status, NonceWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_nonce_window("code-487", "label-628");
        if item.move_to(NonceWindowStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceWindowStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_nonce_window("code-487", "label-628");
        item.status = NonceWindowStatus::Failed;
        assert!(item.move_to(NonceWindowStatus::New).is_err(), "expected move to fail");
    }
}
