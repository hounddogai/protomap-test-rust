use super::*;

use acme_kit::text;

/// Where a primary rule budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRuleBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRuleBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRuleBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rule budget as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleBudget {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub grace_days: i64,
    pub ratio: f64,
    pub window_days: i64,
    pub approved: bool,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub capacity: i64,
    pub country: String,
    pub status: PrimaryRuleBudgetStatus,
}

impl PrimaryRuleBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRuleBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRuleBudgetStatus) -> bool {
        match self.status {
            PrimaryRuleBudgetStatus::Requested => {
                match next {
                    PrimaryRuleBudgetStatus::Granted | PrimaryRuleBudgetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRuleBudgetStatus::Granted => {
                return next == PrimaryRuleBudgetStatus::Revoked || next == PrimaryRuleBudgetStatus::Expired;
            }
            PrimaryRuleBudgetStatus::Revoked => {
                match next {
                    PrimaryRuleBudgetStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryRuleBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rule budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary rule budget", self.code)
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
        (self.floor_value * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRuleBudget {
        PrimaryRuleBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            grace_days: self.grace_days,
            ratio: self.ratio,
            window_days: self.window_days,
            approved: self.approved,
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            capacity: self.capacity,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary rule budget in status requested.
pub fn new_primary_rule_budget(code: &str, name: &str) -> PrimaryRuleBudget {
    PrimaryRuleBudget {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        grace_days: 0,
        ratio: 0.0,
        window_days: 0,
        approved: false,
        floor_value: 0.0,
        tags: Vec::new(),
        capacity: 0,
        country: "".to_string(),
        status: PrimaryRuleBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_rule_budget("code-643", "name-299");
        assert_eq!(item.status, PrimaryRuleBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_rule_budget("code-643", "name-299");
        if item.move_to(PrimaryRuleBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRuleBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_rule_budget("code-643", "name-299");
        item.status = PrimaryRuleBudgetStatus::Expired;
        assert!(
            item.move_to(PrimaryRuleBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
