use super::*;

use acme_kit::text;

/// Where a capped budget burn metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedBudgetBurnMetricStatus {
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

impl std::fmt::Display for CappedBudgetBurnMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped budget burn metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnMetric {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub margin: f64,
    pub quantity: i64,
    pub source: String,
    pub ceiling: f64,
    pub revision: i64,
    pub phone: String,
    pub expires_at: i64,
    pub summary: String,
    pub status: CappedBudgetBurnMetricStatus,
}

impl CappedBudgetBurnMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnMetricStatus) -> bool {
        match self.status {
            CappedBudgetBurnMetricStatus::Draft => {
                match next {
                    CappedBudgetBurnMetricStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnMetricStatus::Pending => {
                match next {
                    CappedBudgetBurnMetricStatus::Active | CappedBudgetBurnMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnMetricStatus::Active => {
                return next == CappedBudgetBurnMetricStatus::Suspended || next == CappedBudgetBurnMetricStatus::Closed;
            }
            CappedBudgetBurnMetricStatus::Suspended => {
                match next {
                    CappedBudgetBurnMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedBudgetBurnMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.revision as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnMetric {
        CappedBudgetBurnMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            margin: self.margin,
            quantity: self.quantity,
            source: self.source.clone(),
            ceiling: self.ceiling,
            revision: self.revision,
            phone: self.phone.clone(),
            expires_at: self.expires_at,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped budget burn metric in status draft.
pub fn new_capped_budget_burn_metric(code: &str, title: &str) -> CappedBudgetBurnMetric {
    CappedBudgetBurnMetric {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        margin: 0.0,
        quantity: 0,
        source: "".to_string(),
        ceiling: 0.0,
        revision: 0,
        phone: "".to_string(),
        expires_at: 0,
        summary: "".to_string(),
        status: CappedBudgetBurnMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_budget_burn_metric("code-104", "title-583");
        assert_eq!(item.status, CappedBudgetBurnMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_capped_budget_burn_metric("code-104", "title-583");
        if item.move_to(CappedBudgetBurnMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedBudgetBurnMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_budget_burn_metric("code-104", "title-583");
        item.status = CappedBudgetBurnMetricStatus::Closed;
        assert!(
            item.move_to(CappedBudgetBurnMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
