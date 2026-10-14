use super::*;

use acme_kit::text;

/// Where a manual forecast summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualForecastSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualForecastSummaryStatus {
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

impl std::fmt::Display for ManualForecastSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual forecast summary as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastSummary {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub description: String,
    pub tenant_id: String,
    pub batch_size: i64,
    pub quota_units: i64,
    pub cluster: String,
    pub status: ManualForecastSummaryStatus,
}

impl ManualForecastSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualForecastSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualForecastSummaryStatus) -> bool {
        match self.status {
            ManualForecastSummaryStatus::Open => {
                return next == ManualForecastSummaryStatus::InReview || next == ManualForecastSummaryStatus::Archived;
            }
            ManualForecastSummaryStatus::InReview => {
                return next == ManualForecastSummaryStatus::Reopened || next == ManualForecastSummaryStatus::Archived;
            }
            ManualForecastSummaryStatus::Resolved => {
                match next {
                    ManualForecastSummaryStatus::Reopened | ManualForecastSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualForecastSummaryStatus::Reopened => {
                match next {
                    ManualForecastSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualForecastSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual forecast summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.priority as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualForecastSummary {
        ManualForecastSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual forecast summary in status open.
pub fn new_manual_forecast_summary(code: &str, name: &str) -> ManualForecastSummary {
    ManualForecastSummary {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        batch_size: 0,
        quota_units: 0,
        cluster: "".to_string(),
        status: ManualForecastSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_forecast_summary("code-641", "name-358");
        assert_eq!(item.status, ManualForecastSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_forecast_summary("code-641", "name-358");
        if item.move_to(ManualForecastSummaryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualForecastSummaryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_forecast_summary("code-641", "name-358");
        item.status = ManualForecastSummaryStatus::Archived;
        assert!(
            item.move_to(ManualForecastSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
