use super::*;

use acme_kit::text;

/// Where a action item forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemForecastStatus {
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

impl std::fmt::Display for ActionItemForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item forecast as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecast {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub owner_id: String,
    pub capacity: i64,
    pub currency: String,
    pub amount: f64,
    pub pinned: bool,
    pub rate: f64,
    pub threshold: f64,
    pub contact_email: String,
    pub status: ActionItemForecastStatus,
}

impl ActionItemForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemForecastStatus) -> bool {
        match self.status {
            ActionItemForecastStatus::Draft => {
                return next == ActionItemForecastStatus::Active;
            }
            ActionItemForecastStatus::Pending => {
                match next {
                    ActionItemForecastStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemForecastStatus::Active => {
                return next == ActionItemForecastStatus::Suspended;
            }
            ActionItemForecastStatus::Suspended => {
                match next {
                    ActionItemForecastStatus::Closed => {
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

    pub fn move_to(&mut self, next: ActionItemForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.amount * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemForecast {
        ActionItemForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            owner_id: self.owner_id.clone(),
            capacity: self.capacity,
            currency: self.currency.clone(),
            amount: self.amount,
            pinned: self.pinned,
            rate: self.rate,
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a action item forecast in status draft.
pub fn new_action_item_forecast(code: &str, label: &str) -> ActionItemForecast {
    ActionItemForecast {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        owner_id: "".to_string(),
        capacity: 0,
        currency: "".to_string(),
        amount: 0.0,
        pinned: false,
        rate: 0.0,
        threshold: 0.0,
        contact_email: "".to_string(),
        status: ActionItemForecastStatus::Draft,
    }
}
