use super::*;

use acme_kit::text;

/// Where a budget burn metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BudgetBurnMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BudgetBurnMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn metric as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMetric {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub reviewer: String,
    pub country: String,
    pub approved: bool,
    pub tags: Vec<String>,
    pub status: BudgetBurnMetricStatus,
}

impl BudgetBurnMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnMetricStatus) -> bool {
        match self.status {
            BudgetBurnMetricStatus::Requested => {
                match next {
                    BudgetBurnMetricStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnMetricStatus::Granted => {
                return next == BudgetBurnMetricStatus::Expired;
            }
            BudgetBurnMetricStatus::Revoked => {
                match next {
                    BudgetBurnMetricStatus::Expired => {
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

    pub fn move_to(&mut self, next: BudgetBurnMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnMetric {
        BudgetBurnMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            approved: self.approved,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a budget burn metric in status requested.
pub fn new_budget_burn_metric(code: &str, title: &str) -> BudgetBurnMetric {
    BudgetBurnMetric {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        ttl_seconds: 0,
        reviewer: "".to_string(),
        country: "".to_string(),
        approved: false,
        tags: Vec::new(),
        status: BudgetBurnMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_budget_burn_metric("code-843", "title-421");
        assert_eq!(item.status, BudgetBurnMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_budget_burn_metric("code-843", "title-421");
        if item.move_to(BudgetBurnMetricStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnMetricStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_budget_burn_metric("code-843", "title-421");
        item.status = BudgetBurnMetricStatus::Expired;
        assert!(
            item.move_to(BudgetBurnMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
