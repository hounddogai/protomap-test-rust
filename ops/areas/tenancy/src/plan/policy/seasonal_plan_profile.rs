use super::*;

use acme_kit::text;

/// Where a seasonal plan profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPlanProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalPlanProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalPlanProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal plan profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfile {
    pub code: String,
    pub name: String,
    pub category: String,
    pub retries: i64,
    pub maximum: f64,
    pub country: String,
    pub limit: i64,
    pub channel: String,
    pub status: SeasonalPlanProfileStatus,
}

impl SeasonalPlanProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPlanProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPlanProfileStatus) -> bool {
        match self.status {
            SeasonalPlanProfileStatus::Planned => {
                match next {
                    SeasonalPlanProfileStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPlanProfileStatus::Scheduled => {
                return next == SeasonalPlanProfileStatus::InProgress || next == SeasonalPlanProfileStatus::Cancelled;
            }
            SeasonalPlanProfileStatus::InProgress => {
                match next {
                    SeasonalPlanProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPlanProfileStatus::Completed => {
                return next == SeasonalPlanProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalPlanProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal plan profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.retries as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPlanProfile {
        SeasonalPlanProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            retries: self.retries,
            maximum: self.maximum,
            country: self.country.clone(),
            limit: self.limit,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal plan profile in status planned.
pub fn new_seasonal_plan_profile(code: &str, name: &str) -> SeasonalPlanProfile {
    SeasonalPlanProfile {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        retries: 0,
        maximum: 0.0,
        country: "".to_string(),
        limit: 0,
        channel: "".to_string(),
        status: SeasonalPlanProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_plan_profile("code-826", "name-563");
        assert_eq!(item.status, SeasonalPlanProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_seasonal_plan_profile("code-826", "name-563");
        if item.move_to(SeasonalPlanProfileStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalPlanProfileStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_plan_profile("code-826", "name-563");
        item.status = SeasonalPlanProfileStatus::Cancelled;
        assert!(
            item.move_to(SeasonalPlanProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
