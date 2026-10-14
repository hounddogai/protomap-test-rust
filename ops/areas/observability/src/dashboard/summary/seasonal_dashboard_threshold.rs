use super::*;

use acme_kit::text;

/// Where a seasonal dashboard threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDashboardThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalDashboardThresholdStatus {
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

impl std::fmt::Display for SeasonalDashboardThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal dashboard threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardThreshold {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub grace_days: i64,
    pub sequence: i64,
    pub unit: String,
    pub capacity: i64,
    pub country: String,
    pub starts_at: i64,
    pub reviewer: String,
    pub source: String,
    pub status: SeasonalDashboardThresholdStatus,
}

impl SeasonalDashboardThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDashboardThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDashboardThresholdStatus) -> bool {
        match self.status {
            SeasonalDashboardThresholdStatus::Planned => {
                match next {
                    SeasonalDashboardThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDashboardThresholdStatus::Scheduled => {
                match next {
                    SeasonalDashboardThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDashboardThresholdStatus::InProgress => {
                return next == SeasonalDashboardThresholdStatus::Completed
                    || next == SeasonalDashboardThresholdStatus::Cancelled;
            }
            SeasonalDashboardThresholdStatus::Completed => {
                return next == SeasonalDashboardThresholdStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDashboardThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dashboard threshold {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.capacity as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDashboardThreshold {
        SeasonalDashboardThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            grace_days: self.grace_days,
            sequence: self.sequence,
            unit: self.unit.clone(),
            capacity: self.capacity,
            country: self.country.clone(),
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal dashboard threshold in status planned.
pub fn new_seasonal_dashboard_threshold(code: &str, name: &str) -> SeasonalDashboardThreshold {
    SeasonalDashboardThreshold {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        grace_days: 0,
        sequence: 0,
        unit: "".to_string(),
        capacity: 0,
        country: "".to_string(),
        starts_at: 0,
        reviewer: "".to_string(),
        source: "".to_string(),
        status: SeasonalDashboardThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_dashboard_threshold("code-532", "name-183");
        assert_eq!(item.status, SeasonalDashboardThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_seasonal_dashboard_threshold("code-532", "name-183");
        if item.move_to(SeasonalDashboardThresholdStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDashboardThresholdStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_dashboard_threshold("code-532", "name-183");
        item.status = SeasonalDashboardThresholdStatus::Cancelled;
        assert!(
            item.move_to(SeasonalDashboardThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
