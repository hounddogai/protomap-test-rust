use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanForecastStatus {
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

impl std::fmt::Display for PlanForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan forecast as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanForecast {
    pub code: String,
    pub label: String,
    pub country: String,
    pub title: String,
    pub version: i64,
    pub external_ref: String,
    pub region: String,
    pub rotation_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub source: String,
    pub limit: i64,
    pub status: PlanForecastStatus,
}

impl PlanForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanForecastStatus) -> bool {
        match self.status {
            PlanForecastStatus::Planned => {
                match next {
                    PlanForecastStatus::Scheduled | PlanForecastStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanForecastStatus::Scheduled => {
                match next {
                    PlanForecastStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanForecastStatus::InProgress => {
                return next == PlanForecastStatus::Completed || next == PlanForecastStatus::Cancelled;
            }
            PlanForecastStatus::Completed => {
                return next == PlanForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_total(&self) -> f64 {
        ((self.version as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanForecast {
        PlanForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            title: self.title.clone(),
            version: self.version,
            external_ref: self.external_ref.clone(),
            region: self.region.clone(),
            rotation_days: self.rotation_days,
            attributes: self.attributes.clone(),
            source: self.source.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a plan forecast in status planned.
pub fn new_plan_forecast(code: &str, label: &str) -> PlanForecast {
    PlanForecast {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        title: "".to_string(),
        version: 0,
        external_ref: "".to_string(),
        region: "".to_string(),
        rotation_days: 0,
        attributes: BTreeMap::new(),
        source: "".to_string(),
        limit: 0,
        status: PlanForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_forecast("code-493", "label-298");
        assert_eq!(item.status, PlanForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_plan_forecast("code-493", "label-298");
        if item.move_to(PlanForecastStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanForecastStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_forecast("code-493", "label-298");
        item.status = PlanForecastStatus::Cancelled;
        assert!(
            item.move_to(PlanForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
