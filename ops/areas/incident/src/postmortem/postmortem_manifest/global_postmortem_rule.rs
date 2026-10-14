use super::*;

use acme_kit::text;

/// Where a global postmortem rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPostmortemRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalPostmortemRuleStatus {
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

impl std::fmt::Display for GlobalPostmortemRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global postmortem rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemRule {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub budget: f64,
    pub threshold: f64,
    pub cluster: String,
    pub notes: String,
    pub port: i64,
    pub approved: bool,
    pub status: GlobalPostmortemRuleStatus,
}

impl GlobalPostmortemRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPostmortemRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPostmortemRuleStatus) -> bool {
        match self.status {
            GlobalPostmortemRuleStatus::Open => {
                match next {
                    GlobalPostmortemRuleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPostmortemRuleStatus::InReview => {
                return next == GlobalPostmortemRuleStatus::Resolved;
            }
            GlobalPostmortemRuleStatus::Resolved => {
                match next {
                    GlobalPostmortemRuleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPostmortemRuleStatus::Reopened => {
                return next == GlobalPostmortemRuleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPostmortemRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global postmortem rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global postmortem rule", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPostmortemRule {
        GlobalPostmortemRule {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            budget: self.budget,
            threshold: self.threshold,
            cluster: self.cluster.clone(),
            notes: self.notes.clone(),
            port: self.port,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global postmortem rule in status open.
pub fn new_global_postmortem_rule(code: &str, name: &str) -> GlobalPostmortemRule {
    GlobalPostmortemRule {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        budget: 0.0,
        threshold: 0.0,
        cluster: "".to_string(),
        notes: "".to_string(),
        port: 0,
        approved: false,
        status: GlobalPostmortemRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_postmortem_rule("code-324", "name-850");
        assert_eq!(item.status, GlobalPostmortemRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_global_postmortem_rule("code-324", "name-850");
        if item.move_to(GlobalPostmortemRuleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPostmortemRuleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_postmortem_rule("code-324", "name-850");
        item.status = GlobalPostmortemRuleStatus::Archived;
        assert!(
            item.move_to(GlobalPostmortemRuleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
