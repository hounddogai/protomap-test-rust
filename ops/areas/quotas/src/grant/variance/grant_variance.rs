use super::*;

use acme_kit::text;

/// Where a grant variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantVarianceStatus {
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

impl std::fmt::Display for GrantVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantVariance {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub capacity: i64,
    pub amount: f64,
    pub attempts: i64,
    pub window_days: i64,
    pub status: GrantVarianceStatus,
}

impl GrantVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantVarianceStatus) -> bool {
        match self.status {
            GrantVarianceStatus::New => {
                match next {
                    GrantVarianceStatus::Running | GrantVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantVarianceStatus::Queued => {
                return next == GrantVarianceStatus::Done;
            }
            GrantVarianceStatus::Running => {
                match next {
                    GrantVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantVarianceStatus::Done => {
                match next {
                    GrantVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: GrantVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.window_days as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantVariance {
        GrantVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            capacity: self.capacity,
            amount: self.amount,
            attempts: self.attempts,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a grant variance in status new.
pub fn new_grant_variance(code: &str, title: &str) -> GrantVariance {
    GrantVariance {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        capacity: 0,
        amount: 0.0,
        attempts: 0,
        window_days: 0,
        status: GrantVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_variance("code-71", "title-621");
        assert_eq!(item.status, GrantVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_grant_variance("code-71", "title-621");
        if item.move_to(GrantVarianceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantVarianceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_variance("code-71", "title-621");
        item.status = GrantVarianceStatus::Failed;
        assert!(item.move_to(GrantVarianceStatus::New).is_err(), "expected move to fail");
    }
}
