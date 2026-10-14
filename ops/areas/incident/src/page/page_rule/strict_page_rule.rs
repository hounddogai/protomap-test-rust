use super::*;

use acme_kit::text;

/// Where a strict page rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPageRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictPageRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictPageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict page rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRule {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub unit: String,
    pub budget: f64,
    pub region: String,
    pub notes: String,
    pub grace_days: i64,
    pub created_at: i64,
    pub status: StrictPageRuleStatus,
}

impl StrictPageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPageRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPageRuleStatus) -> bool {
        match self.status {
            StrictPageRuleStatus::Created => {
                match next {
                    StrictPageRuleStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPageRuleStatus::Validated => {
                match next {
                    StrictPageRuleStatus::Published | StrictPageRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPageRuleStatus::Published => {
                match next {
                    StrictPageRuleStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictPageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict page rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.created_at as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPageRule {
        StrictPageRule {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            unit: self.unit.clone(),
            budget: self.budget,
            region: self.region.clone(),
            notes: self.notes.clone(),
            grace_days: self.grace_days,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict page rule in status created.
pub fn new_strict_page_rule(code: &str, title: &str) -> StrictPageRule {
    StrictPageRule {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        unit: "".to_string(),
        budget: 0.0,
        region: "".to_string(),
        notes: "".to_string(),
        grace_days: 0,
        created_at: 0,
        status: StrictPageRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_page_rule("code-958", "title-635");
        assert_eq!(item.status, StrictPageRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_page_rule("code-958", "title-635");
        if item.move_to(StrictPageRuleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPageRuleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_page_rule("code-958", "title-635");
        item.status = StrictPageRuleStatus::Retired;
        assert!(
            item.move_to(StrictPageRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
