use super::*;

use acme_kit::text;

/// Where a seasonal release rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalReleaseRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalReleaseRuleStatus {
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

impl std::fmt::Display for SeasonalReleaseRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal release rule as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRule {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub quota_units: i64,
    pub limit: i64,
    pub currency: String,
    pub country: String,
    pub status: SeasonalReleaseRuleStatus,
}

impl SeasonalReleaseRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalReleaseRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalReleaseRuleStatus) -> bool {
        match self.status {
            SeasonalReleaseRuleStatus::New => {
                match next {
                    SeasonalReleaseRuleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReleaseRuleStatus::Queued => {
                return next == SeasonalReleaseRuleStatus::Done;
            }
            SeasonalReleaseRuleStatus::Running => {
                return next == SeasonalReleaseRuleStatus::Done || next == SeasonalReleaseRuleStatus::Failed;
            }
            SeasonalReleaseRuleStatus::Done => {
                return next == SeasonalReleaseRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalReleaseRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal release rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.quota_units as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalReleaseRule {
        SeasonalReleaseRule {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            quota_units: self.quota_units,
            limit: self.limit,
            currency: self.currency.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal release rule in status new.
pub fn new_seasonal_release_rule(code: &str, title: &str) -> SeasonalReleaseRule {
    SeasonalReleaseRule {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        quota_units: 0,
        limit: 0,
        currency: "".to_string(),
        country: "".to_string(),
        status: SeasonalReleaseRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_release_rule("code-78", "title-787");
        assert_eq!(item.status, SeasonalReleaseRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_seasonal_release_rule("code-78", "title-787");
        if item.move_to(SeasonalReleaseRuleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalReleaseRuleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_release_rule("code-78", "title-787");
        item.status = SeasonalReleaseRuleStatus::Failed;
        assert!(
            item.move_to(SeasonalReleaseRuleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
