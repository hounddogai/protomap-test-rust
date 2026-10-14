use super::*;

use acme_kit::text;

/// Where a verifier window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl VerifierWindowStatus {
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

impl std::fmt::Display for VerifierWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierWindow {
    pub code: String,
    pub title: String,
    pub host: String,
    pub starts_at: i64,
    pub rate: f64,
    pub quantity: i64,
    pub status: VerifierWindowStatus,
}

impl VerifierWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierWindowStatus) -> bool {
        match self.status {
            VerifierWindowStatus::New => {
                match next {
                    VerifierWindowStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierWindowStatus::Queued => {
                match next {
                    VerifierWindowStatus::Running | VerifierWindowStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierWindowStatus::Running => {
                match next {
                    VerifierWindowStatus::Done | VerifierWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierWindowStatus::Done => {
                return next == VerifierWindowStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.quantity as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierWindow {
        VerifierWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            starts_at: self.starts_at,
            rate: self.rate,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a verifier window in status new.
pub fn new_verifier_window(code: &str, title: &str) -> VerifierWindow {
    VerifierWindow {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        starts_at: 0,
        rate: 0.0,
        quantity: 0,
        status: VerifierWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_verifier_window("code-422", "title-218");
        assert_eq!(item.status, VerifierWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_verifier_window("code-422", "title-218");
        if item.move_to(VerifierWindowStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierWindowStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_verifier_window("code-422", "title-218");
        item.status = VerifierWindowStatus::Failed;
        assert!(
            item.move_to(VerifierWindowStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
