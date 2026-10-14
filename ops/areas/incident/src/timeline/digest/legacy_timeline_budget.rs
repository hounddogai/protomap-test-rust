use super::*;

use acme_kit::text;

/// Where a legacy timeline budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTimelineBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyTimelineBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyTimelineBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy timeline budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineBudget {
    pub code: String,
    pub title: String,
    pub source: String,
    pub host: String,
    pub quota_units: i64,
    pub revision: i64,
    pub channel: String,
    pub approved: bool,
    pub issuer: String,
    pub category: String,
    pub duration_minutes: i64,
    pub status: LegacyTimelineBudgetStatus,
}

impl LegacyTimelineBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTimelineBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTimelineBudgetStatus) -> bool {
        match self.status {
            LegacyTimelineBudgetStatus::Created => {
                return next == LegacyTimelineBudgetStatus::Published || next == LegacyTimelineBudgetStatus::Retired;
            }
            LegacyTimelineBudgetStatus::Validated => {
                return next == LegacyTimelineBudgetStatus::Published;
            }
            LegacyTimelineBudgetStatus::Published => {
                return next == LegacyTimelineBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyTimelineBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy timeline budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTimelineBudget {
        LegacyTimelineBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            host: self.host.clone(),
            quota_units: self.quota_units,
            revision: self.revision,
            channel: self.channel.clone(),
            approved: self.approved,
            issuer: self.issuer.clone(),
            category: self.category.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy timeline budget in status created.
pub fn new_legacy_timeline_budget(code: &str, title: &str) -> LegacyTimelineBudget {
    LegacyTimelineBudget {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        host: "".to_string(),
        quota_units: 0,
        revision: 0,
        channel: "".to_string(),
        approved: false,
        issuer: "".to_string(),
        category: "".to_string(),
        duration_minutes: 0,
        status: LegacyTimelineBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_timeline_budget("code-908", "title-284");
        assert_eq!(item.status, LegacyTimelineBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_legacy_timeline_budget("code-908", "title-284");
        if item.move_to(LegacyTimelineBudgetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTimelineBudgetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_timeline_budget("code-908", "title-284");
        item.status = LegacyTimelineBudgetStatus::Retired;
        assert!(
            item.move_to(LegacyTimelineBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
