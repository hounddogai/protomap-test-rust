use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a forecast milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastMilestoneStatus {
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

impl std::fmt::Display for ForecastMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMilestone {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub external_ref: String,
    pub reviewer: String,
    pub discount: f64,
    pub status: ForecastMilestoneStatus,
}

impl ForecastMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastMilestoneStatus) -> bool {
        match self.status {
            ForecastMilestoneStatus::Draft => {
                match next {
                    ForecastMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastMilestoneStatus::Pending => {
                return next == ForecastMilestoneStatus::Closed;
            }
            ForecastMilestoneStatus::Active => {
                return next == ForecastMilestoneStatus::Suspended || next == ForecastMilestoneStatus::Closed;
            }
            ForecastMilestoneStatus::Suspended => {
                return next == ForecastMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "forecast milestone", self.code)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.discount - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastMilestone {
        ForecastMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            external_ref: self.external_ref.clone(),
            reviewer: self.reviewer.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast milestone in status draft.
pub fn new_forecast_milestone(code: &str, label: &str) -> ForecastMilestone {
    ForecastMilestone {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        score: 0.0,
        external_ref: "".to_string(),
        reviewer: "".to_string(),
        discount: 0.0,
        status: ForecastMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_milestone("code-274", "label-250");
        assert_eq!(item.status, ForecastMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_forecast_milestone("code-274", "label-250");
        if item.move_to(ForecastMilestoneStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastMilestoneStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_milestone("code-274", "label-250");
        item.status = ForecastMilestoneStatus::Closed;
        assert!(
            item.move_to(ForecastMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
