use super::*;

use acme_kit::text;

/// Where a weighted budget burn window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedBudgetBurnWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedBudgetBurnWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedBudgetBurnWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted budget burn window as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnWindow {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub cluster: String,
    pub country: String,
    pub starts_at: i64,
    pub ratio: f64,
    pub limit: i64,
    pub expires_at: i64,
    pub unit: String,
    pub principal: String,
    pub status: WeightedBudgetBurnWindowStatus,
}

impl WeightedBudgetBurnWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedBudgetBurnWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedBudgetBurnWindowStatus) -> bool {
        match self.status {
            WeightedBudgetBurnWindowStatus::Proposed => {
                match next {
                    WeightedBudgetBurnWindowStatus::Approved | WeightedBudgetBurnWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnWindowStatus::Approved => {
                match next {
                    WeightedBudgetBurnWindowStatus::Rejected | WeightedBudgetBurnWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnWindowStatus::Rejected => {
                match next {
                    WeightedBudgetBurnWindowStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: WeightedBudgetBurnWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted budget burn window {} cannot move from {} to {}",
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
        format!("{} {}", "weighted budget burn window", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.expires_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedBudgetBurnWindow {
        WeightedBudgetBurnWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            country: self.country.clone(),
            starts_at: self.starts_at,
            ratio: self.ratio,
            limit: self.limit,
            expires_at: self.expires_at,
            unit: self.unit.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a weighted budget burn window in status proposed.
pub fn new_weighted_budget_burn_window(code: &str, title: &str) -> WeightedBudgetBurnWindow {
    WeightedBudgetBurnWindow {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        cluster: "".to_string(),
        country: "".to_string(),
        starts_at: 0,
        ratio: 0.0,
        limit: 0,
        expires_at: 0,
        unit: "".to_string(),
        principal: "".to_string(),
        status: WeightedBudgetBurnWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_budget_burn_window("code-678", "title-628");
        assert_eq!(item.status, WeightedBudgetBurnWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_weighted_budget_burn_window("code-678", "title-628");
        if item.move_to(WeightedBudgetBurnWindowStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedBudgetBurnWindowStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_budget_burn_window("code-678", "title-628");
        item.status = WeightedBudgetBurnWindowStatus::Withdrawn;
        assert!(
            item.move_to(WeightedBudgetBurnWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
