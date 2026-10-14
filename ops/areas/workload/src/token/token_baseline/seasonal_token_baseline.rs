use super::*;

use acme_kit::text;

/// Where a seasonal token baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTokenBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalTokenBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalTokenBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal token baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenBaseline {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub cipher: String,
    pub ceiling: f64,
    pub retries: i64,
    pub status: SeasonalTokenBaselineStatus,
}

impl SeasonalTokenBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTokenBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTokenBaselineStatus) -> bool {
        match self.status {
            SeasonalTokenBaselineStatus::Planned => {
                match next {
                    SeasonalTokenBaselineStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenBaselineStatus::Scheduled => {
                match next {
                    SeasonalTokenBaselineStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenBaselineStatus::InProgress => {
                match next {
                    SeasonalTokenBaselineStatus::Completed | SeasonalTokenBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenBaselineStatus::Completed => {
                return next == SeasonalTokenBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTokenBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal token baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal token baseline", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ceiling - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTokenBaseline {
        SeasonalTokenBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            cipher: self.cipher.clone(),
            ceiling: self.ceiling,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal token baseline in status planned.
pub fn new_seasonal_token_baseline(code: &str, name: &str) -> SeasonalTokenBaseline {
    SeasonalTokenBaseline {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        cipher: "".to_string(),
        ceiling: 0.0,
        retries: 0,
        status: SeasonalTokenBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_token_baseline("code-787", "name-502");
        assert_eq!(item.status, SeasonalTokenBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_seasonal_token_baseline("code-787", "name-502");
        if item.move_to(SeasonalTokenBaselineStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTokenBaselineStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_token_baseline("code-787", "name-502");
        item.status = SeasonalTokenBaselineStatus::Cancelled;
        assert!(
            item.move_to(SeasonalTokenBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
