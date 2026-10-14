use super::*;

use acme_kit::text;

/// Where a forecast rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ForecastRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRule {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub currency: String,
    pub created_at: i64,
    pub discount: f64,
    pub budget: f64,
    pub window_days: i64,
    pub segment: String,
    pub pinned: bool,
    pub quota_units: i64,
    pub status: ForecastRuleStatus,
}

impl ForecastRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastRuleStatus) -> bool {
        match self.status {
            ForecastRuleStatus::Draft => {
                return next == ForecastRuleStatus::Pending || next == ForecastRuleStatus::Closed;
            }
            ForecastRuleStatus::Pending => {
                return next == ForecastRuleStatus::Closed;
            }
            ForecastRuleStatus::Active => {
                return next == ForecastRuleStatus::Suspended;
            }
            ForecastRuleStatus::Suspended => {
                match next {
                    ForecastRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: ForecastRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast rule {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.discount - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastRule {
        ForecastRule {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            currency: self.currency.clone(),
            created_at: self.created_at,
            discount: self.discount,
            budget: self.budget,
            window_days: self.window_days,
            segment: self.segment.clone(),
            pinned: self.pinned,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a forecast rule in status draft.
pub fn new_forecast_rule(code: &str, name: &str) -> ForecastRule {
    ForecastRule {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        currency: "".to_string(),
        created_at: 0,
        discount: 0.0,
        budget: 0.0,
        window_days: 0,
        segment: "".to_string(),
        pinned: false,
        quota_units: 0,
        status: ForecastRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_rule("code-383", "name-524");
        assert_eq!(item.status, ForecastRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_forecast_rule("code-383", "name-524");
        if item.move_to(ForecastRuleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastRuleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_rule("code-383", "name-524");
        item.status = ForecastRuleStatus::Closed;
        assert!(
            item.move_to(ForecastRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
