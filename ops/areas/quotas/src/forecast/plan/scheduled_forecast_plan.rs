use super::*;

use acme_kit::text;

/// Where a scheduled forecast plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledForecastPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledForecastPlanStatus {
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

impl std::fmt::Display for ScheduledForecastPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled forecast plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPlan {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub namespace: String,
    pub starts_at: i64,
    pub score: f64,
    pub status: ScheduledForecastPlanStatus,
}

impl ScheduledForecastPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledForecastPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledForecastPlanStatus) -> bool {
        match self.status {
            ScheduledForecastPlanStatus::Open => {
                return next == ScheduledForecastPlanStatus::Archived;
            }
            ScheduledForecastPlanStatus::InReview => {
                return next == ScheduledForecastPlanStatus::Resolved;
            }
            ScheduledForecastPlanStatus::Resolved => {
                return next == ScheduledForecastPlanStatus::Archived;
            }
            ScheduledForecastPlanStatus::Reopened => {
                match next {
                    ScheduledForecastPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledForecastPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled forecast plan {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.starts_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledForecastPlan {
        ScheduledForecastPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            starts_at: self.starts_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled forecast plan in status open.
pub fn new_scheduled_forecast_plan(code: &str, name: &str) -> ScheduledForecastPlan {
    ScheduledForecastPlan {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        namespace: "".to_string(),
        starts_at: 0,
        score: 0.0,
        status: ScheduledForecastPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_forecast_plan("code-74", "name-815");
        assert_eq!(item.status, ScheduledForecastPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_scheduled_forecast_plan("code-74", "name-815");
        if item.move_to(ScheduledForecastPlanStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledForecastPlanStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_forecast_plan("code-74", "name-815");
        item.status = ScheduledForecastPlanStatus::Archived;
        assert!(
            item.move_to(ScheduledForecastPlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
