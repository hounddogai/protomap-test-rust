use super::*;

use acme_kit::text;

/// Where a fallback budget burn metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBudgetBurnMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackBudgetBurnMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackBudgetBurnMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback budget burn metric as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetric {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub quota_units: i64,
    pub channel: String,
    pub pinned: bool,
    pub notes: String,
    pub country: String,
    pub visible: bool,
    pub checksum: String,
    pub cluster: String,
    pub status: FallbackBudgetBurnMetricStatus,
}

impl FallbackBudgetBurnMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBudgetBurnMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBudgetBurnMetricStatus) -> bool {
        match self.status {
            FallbackBudgetBurnMetricStatus::Created => {
                return next == FallbackBudgetBurnMetricStatus::Published;
            }
            FallbackBudgetBurnMetricStatus::Validated => {
                match next {
                    FallbackBudgetBurnMetricStatus::Published | FallbackBudgetBurnMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBudgetBurnMetricStatus::Published => {
                match next {
                    FallbackBudgetBurnMetricStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackBudgetBurnMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback budget burn metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.burst as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBudgetBurnMetric {
        FallbackBudgetBurnMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            pinned: self.pinned,
            notes: self.notes.clone(),
            country: self.country.clone(),
            visible: self.visible,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback budget burn metric in status created.
pub fn new_fallback_budget_burn_metric(code: &str, name: &str) -> FallbackBudgetBurnMetric {
    FallbackBudgetBurnMetric {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        quota_units: 0,
        channel: "".to_string(),
        pinned: false,
        notes: "".to_string(),
        country: "".to_string(),
        visible: false,
        checksum: "".to_string(),
        cluster: "".to_string(),
        status: FallbackBudgetBurnMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_budget_burn_metric("code-362", "name-977");
        assert_eq!(item.status, FallbackBudgetBurnMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_fallback_budget_burn_metric("code-362", "name-977");
        if item.move_to(FallbackBudgetBurnMetricStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBudgetBurnMetricStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_budget_burn_metric("code-362", "name-977");
        item.status = FallbackBudgetBurnMetricStatus::Retired;
        assert!(
            item.move_to(FallbackBudgetBurnMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
