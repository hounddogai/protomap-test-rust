use super::*;

use acme_kit::text;

/// Where a regional budget burn forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalBudgetBurnForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalBudgetBurnForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalBudgetBurnForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional budget burn forecast as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecast {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub contact_email: String,
    pub quantity: i64,
    pub active: bool,
    pub checksum: String,
    pub priority: i64,
    pub segment: String,
    pub status: RegionalBudgetBurnForecastStatus,
}

impl RegionalBudgetBurnForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalBudgetBurnForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalBudgetBurnForecastStatus) -> bool {
        match self.status {
            RegionalBudgetBurnForecastStatus::Proposed => {
                return next == RegionalBudgetBurnForecastStatus::Withdrawn;
            }
            RegionalBudgetBurnForecastStatus::Approved => {
                return next == RegionalBudgetBurnForecastStatus::Rejected;
            }
            RegionalBudgetBurnForecastStatus::Rejected => {
                match next {
                    RegionalBudgetBurnForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalBudgetBurnForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional budget burn forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalBudgetBurnForecast {
        RegionalBudgetBurnForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            contact_email: self.contact_email.clone(),
            quantity: self.quantity,
            active: self.active,
            checksum: self.checksum.clone(),
            priority: self.priority,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional budget burn forecast in status proposed.
pub fn new_regional_budget_burn_forecast(code: &str, title: &str) -> RegionalBudgetBurnForecast {
    RegionalBudgetBurnForecast {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        cipher: "".to_string(),
        contact_email: "".to_string(),
        quantity: 0,
        active: false,
        checksum: "".to_string(),
        priority: 0,
        segment: "".to_string(),
        status: RegionalBudgetBurnForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_budget_burn_forecast("code-402", "title-806");
        assert_eq!(item.status, RegionalBudgetBurnForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_regional_budget_burn_forecast("code-402", "title-806");
        if item.move_to(RegionalBudgetBurnForecastStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalBudgetBurnForecastStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_budget_burn_forecast("code-402", "title-806");
        item.status = RegionalBudgetBurnForecastStatus::Withdrawn;
        assert!(
            item.move_to(RegionalBudgetBurnForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
