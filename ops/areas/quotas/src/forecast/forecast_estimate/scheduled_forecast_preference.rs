use super::*;

use acme_kit::text;

/// Where a scheduled forecast preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledForecastPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledForecastPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledForecastPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled forecast preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreference {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub unit: String,
    pub description: String,
    pub budget: f64,
    pub checksum: String,
    pub timezone: String,
    pub principal: String,
    pub key_id: String,
    pub amount: f64,
    pub status: ScheduledForecastPreferenceStatus,
}

impl ScheduledForecastPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledForecastPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledForecastPreferenceStatus) -> bool {
        match self.status {
            ScheduledForecastPreferenceStatus::Proposed => {
                return next == ScheduledForecastPreferenceStatus::Rejected;
            }
            ScheduledForecastPreferenceStatus::Approved => {
                return next == ScheduledForecastPreferenceStatus::Rejected
                    || next == ScheduledForecastPreferenceStatus::Withdrawn;
            }
            ScheduledForecastPreferenceStatus::Rejected => {
                return next == ScheduledForecastPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledForecastPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled forecast preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.floor_value * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledForecastPreference {
        ScheduledForecastPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            unit: self.unit.clone(),
            description: self.description.clone(),
            budget: self.budget,
            checksum: self.checksum.clone(),
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            key_id: self.key_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled forecast preference in status proposed.
pub fn new_scheduled_forecast_preference(code: &str, label: &str) -> ScheduledForecastPreference {
    ScheduledForecastPreference {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        unit: "".to_string(),
        description: "".to_string(),
        budget: 0.0,
        checksum: "".to_string(),
        timezone: "".to_string(),
        principal: "".to_string(),
        key_id: "".to_string(),
        amount: 0.0,
        status: ScheduledForecastPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_forecast_preference("code-833", "label-80");
        assert_eq!(item.status, ScheduledForecastPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_scheduled_forecast_preference("code-833", "label-80");
        if item.move_to(ScheduledForecastPreferenceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledForecastPreferenceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_forecast_preference("code-833", "label-80");
        item.status = ScheduledForecastPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledForecastPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
