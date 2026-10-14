use super::*;

use acme_kit::text;

/// Where a silence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilenceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBudget {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub attempts: i64,
    pub limit: i64,
    pub score: f64,
    pub starts_at: i64,
    pub batch_size: i64,
    pub retries: i64,
    pub tags: Vec<String>,
    pub status: SilenceBudgetStatus,
}

impl SilenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceBudgetStatus) -> bool {
        match self.status {
            SilenceBudgetStatus::Proposed => {
                return next == SilenceBudgetStatus::Approved || next == SilenceBudgetStatus::Rejected;
            }
            SilenceBudgetStatus::Approved => {
                return next == SilenceBudgetStatus::Withdrawn;
            }
            SilenceBudgetStatus::Rejected => {
                return next == SilenceBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence budget {} cannot move from {} to {}",
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
        format!("{} {}", "silence budget", self.code)
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
        ((self.retries as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceBudget {
        SilenceBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            attempts: self.attempts,
            limit: self.limit,
            score: self.score,
            starts_at: self.starts_at,
            batch_size: self.batch_size,
            retries: self.retries,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a silence budget in status proposed.
pub fn new_silence_budget(code: &str, title: &str) -> SilenceBudget {
    SilenceBudget {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        attempts: 0,
        limit: 0,
        score: 0.0,
        starts_at: 0,
        batch_size: 0,
        retries: 0,
        tags: Vec::new(),
        status: SilenceBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_silence_budget("code-225", "title-789");
        assert_eq!(item.status, SilenceBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_silence_budget("code-225", "title-789");
        if item.move_to(SilenceBudgetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceBudgetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_silence_budget("code-225", "title-789");
        item.status = SilenceBudgetStatus::Withdrawn;
        assert!(
            item.move_to(SilenceBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
