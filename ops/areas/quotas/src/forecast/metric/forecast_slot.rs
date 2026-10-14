use super::*;

use acme_kit::text;

/// Where a forecast slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastSlotStatus {
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

impl std::fmt::Display for ForecastSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSlot {
    pub code: String,
    pub title: String,
    pub country: String,
    pub floor_value: f64,
    pub budget: f64,
    pub sequence: i64,
    pub region: String,
    pub ratio: f64,
    pub channel: String,
    pub contact_email: String,
    pub status: ForecastSlotStatus,
}

impl ForecastSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastSlotStatus) -> bool {
        match self.status {
            ForecastSlotStatus::Draft => {
                match next {
                    ForecastSlotStatus::Pending | ForecastSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSlotStatus::Pending => {
                match next {
                    ForecastSlotStatus::Suspended | ForecastSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSlotStatus::Active => {
                match next {
                    ForecastSlotStatus::Suspended | ForecastSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSlotStatus::Suspended => {
                return next == ForecastSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.floor_value + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastSlot {
        ForecastSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            floor_value: self.floor_value,
            budget: self.budget,
            sequence: self.sequence,
            region: self.region.clone(),
            ratio: self.ratio,
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a forecast slot in status draft.
pub fn new_forecast_slot(code: &str, title: &str) -> ForecastSlot {
    ForecastSlot {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        floor_value: 0.0,
        budget: 0.0,
        sequence: 0,
        region: "".to_string(),
        ratio: 0.0,
        channel: "".to_string(),
        contact_email: "".to_string(),
        status: ForecastSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_slot("code-79", "title-537");
        assert_eq!(item.status, ForecastSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_forecast_slot("code-79", "title-537");
        if item.move_to(ForecastSlotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastSlotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_slot("code-79", "title-537");
        item.status = ForecastSlotStatus::Closed;
        assert!(
            item.move_to(ForecastSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
