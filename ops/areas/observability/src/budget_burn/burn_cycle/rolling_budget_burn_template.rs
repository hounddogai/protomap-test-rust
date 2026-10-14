use super::*;

use acme_kit::text;

/// Where a rolling budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBudgetBurnTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingBudgetBurnTemplateStatus {
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

impl std::fmt::Display for RollingBudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling budget burn template as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnTemplate {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub tags: Vec<String>,
    pub host: String,
    pub rotation_days: i64,
    pub status: RollingBudgetBurnTemplateStatus,
}

impl RollingBudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBudgetBurnTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBudgetBurnTemplateStatus) -> bool {
        match self.status {
            RollingBudgetBurnTemplateStatus::Open => {
                match next {
                    RollingBudgetBurnTemplateStatus::InReview | RollingBudgetBurnTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnTemplateStatus::InReview => {
                return next == RollingBudgetBurnTemplateStatus::Reopened;
            }
            RollingBudgetBurnTemplateStatus::Resolved => {
                return next == RollingBudgetBurnTemplateStatus::Reopened
                    || next == RollingBudgetBurnTemplateStatus::Archived;
            }
            RollingBudgetBurnTemplateStatus::Reopened => {
                match next {
                    RollingBudgetBurnTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingBudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling budget burn template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling budget burn template", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.version as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingBudgetBurnTemplate {
        RollingBudgetBurnTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            tags: self.tags.clone(),
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling budget burn template in status open.
pub fn new_rolling_budget_burn_template(code: &str, title: &str) -> RollingBudgetBurnTemplate {
    RollingBudgetBurnTemplate {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        tags: Vec::new(),
        host: "".to_string(),
        rotation_days: 0,
        status: RollingBudgetBurnTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_budget_burn_template("code-629", "title-911");
        assert_eq!(item.status, RollingBudgetBurnTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_rolling_budget_burn_template("code-629", "title-911");
        if item.move_to(RollingBudgetBurnTemplateStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingBudgetBurnTemplateStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_budget_burn_template("code-629", "title-911");
        item.status = RollingBudgetBurnTemplateStatus::Archived;
        assert!(
            item.move_to(RollingBudgetBurnTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
