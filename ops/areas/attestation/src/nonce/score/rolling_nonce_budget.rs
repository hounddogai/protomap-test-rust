use super::*;

use acme_kit::text;

/// Where a rolling nonce budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingNonceBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingNonceBudgetStatus {
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

impl std::fmt::Display for RollingNonceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling nonce budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBudget {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub country: String,
    pub retries: i64,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub version: i64,
    pub cipher: String,
    pub status: RollingNonceBudgetStatus,
}

impl RollingNonceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingNonceBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingNonceBudgetStatus) -> bool {
        match self.status {
            RollingNonceBudgetStatus::New => {
                match next {
                    RollingNonceBudgetStatus::Queued | RollingNonceBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingNonceBudgetStatus::Queued => {
                return next == RollingNonceBudgetStatus::Running;
            }
            RollingNonceBudgetStatus::Running => {
                return next == RollingNonceBudgetStatus::Done || next == RollingNonceBudgetStatus::Failed;
            }
            RollingNonceBudgetStatus::Done => {
                return next == RollingNonceBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingNonceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling nonce budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingNonceBudget {
        RollingNonceBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            country: self.country.clone(),
            retries: self.retries,
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling nonce budget in status new.
pub fn new_rolling_nonce_budget(code: &str, name: &str) -> RollingNonceBudget {
    RollingNonceBudget {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        country: "".to_string(),
        retries: 0,
        maximum: 0.0,
        duration_minutes: 0,
        tags: Vec::new(),
        version: 0,
        cipher: "".to_string(),
        status: RollingNonceBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_nonce_budget("code-400", "name-751");
        assert_eq!(item.status, RollingNonceBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rolling_nonce_budget("code-400", "name-751");
        if item.move_to(RollingNonceBudgetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingNonceBudgetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_nonce_budget("code-400", "name-751");
        item.status = RollingNonceBudgetStatus::Failed;
        assert!(
            item.move_to(RollingNonceBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
