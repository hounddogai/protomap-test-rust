use super::*;

use acme_kit::text;

/// Where a flow log forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log forecast as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogForecast {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub currency: String,
    pub category: String,
    pub active: bool,
    pub discount: f64,
    pub port: i64,
    pub retries: i64,
    pub visible: bool,
    pub amount: f64,
    pub status: FlowLogForecastStatus,
}

impl FlowLogForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogForecastStatus) -> bool {
        match self.status {
            FlowLogForecastStatus::Proposed => {
                return next == FlowLogForecastStatus::Rejected;
            }
            FlowLogForecastStatus::Approved => {
                match next {
                    FlowLogForecastStatus::Rejected | FlowLogForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogForecastStatus::Rejected => {
                return next == FlowLogForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.discount - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogForecast {
        FlowLogForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            currency: self.currency.clone(),
            category: self.category.clone(),
            active: self.active,
            discount: self.discount,
            port: self.port,
            retries: self.retries,
            visible: self.visible,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a flow log forecast in status proposed.
pub fn new_flow_log_forecast(code: &str, title: &str) -> FlowLogForecast {
    FlowLogForecast {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        currency: "".to_string(),
        category: "".to_string(),
        active: false,
        discount: 0.0,
        port: 0,
        retries: 0,
        visible: false,
        amount: 0.0,
        status: FlowLogForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_forecast("code-842", "title-396");
        assert_eq!(item.status, FlowLogForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_flow_log_forecast("code-842", "title-396");
        if item.move_to(FlowLogForecastStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogForecastStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_forecast("code-842", "title-396");
        item.status = FlowLogForecastStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
