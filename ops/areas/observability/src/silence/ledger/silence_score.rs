use super::*;

use acme_kit::text;

/// Where a silence score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SilenceScoreStatus {
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

impl std::fmt::Display for SilenceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceScore {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub duration_minutes: i64,
    pub budget: f64,
    pub status: SilenceScoreStatus,
}

impl SilenceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceScoreStatus) -> bool {
        match self.status {
            SilenceScoreStatus::New => {
                match next {
                    SilenceScoreStatus::Running | SilenceScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceScoreStatus::Queued => {
                match next {
                    SilenceScoreStatus::Running | SilenceScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceScoreStatus::Running => {
                return next == SilenceScoreStatus::Done || next == SilenceScoreStatus::Failed;
            }
            SilenceScoreStatus::Done => {
                match next {
                    SilenceScoreStatus::Failed => {
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

    pub fn move_to(&mut self, next: SilenceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence score", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.window_days as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceScore {
        SilenceScore {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            duration_minutes: self.duration_minutes,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a silence score in status new.
pub fn new_silence_score(code: &str, name: &str) -> SilenceScore {
    SilenceScore {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        duration_minutes: 0,
        budget: 0.0,
        status: SilenceScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_silence_score("code-580", "name-498");
        assert_eq!(item.status, SilenceScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_silence_score("code-580", "name-498");
        if item.move_to(SilenceScoreStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceScoreStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_silence_score("code-580", "name-498");
        item.status = SilenceScoreStatus::Failed;
        assert!(item.move_to(SilenceScoreStatus::New).is_err(), "expected move to fail");
    }
}
