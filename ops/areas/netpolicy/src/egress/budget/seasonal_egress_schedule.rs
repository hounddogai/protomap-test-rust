use super::*;

use acme_kit::text;

/// Where a seasonal egress schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEgressScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalEgressScheduleStatus {
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

impl std::fmt::Display for SeasonalEgressScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal egress schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressSchedule {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub minimum: f64,
    pub approved: bool,
    pub timezone: String,
    pub source: String,
    pub sequence: i64,
    pub currency: String,
    pub summary: String,
    pub status: SeasonalEgressScheduleStatus,
}

impl SeasonalEgressSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEgressScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEgressScheduleStatus) -> bool {
        match self.status {
            SeasonalEgressScheduleStatus::Draft => {
                match next {
                    SeasonalEgressScheduleStatus::Active | SeasonalEgressScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEgressScheduleStatus::Pending => {
                match next {
                    SeasonalEgressScheduleStatus::Active | SeasonalEgressScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEgressScheduleStatus::Active => {
                return next == SeasonalEgressScheduleStatus::Closed;
            }
            SeasonalEgressScheduleStatus::Suspended => {
                match next {
                    SeasonalEgressScheduleStatus::Closed => {
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

    pub fn move_to(&mut self, next: SeasonalEgressScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal egress schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal egress schedule", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.sequence as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEgressSchedule {
        SeasonalEgressSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            minimum: self.minimum,
            approved: self.approved,
            timezone: self.timezone.clone(),
            source: self.source.clone(),
            sequence: self.sequence,
            currency: self.currency.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal egress schedule in status draft.
pub fn new_seasonal_egress_schedule(code: &str, name: &str) -> SeasonalEgressSchedule {
    SeasonalEgressSchedule {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        minimum: 0.0,
        approved: false,
        timezone: "".to_string(),
        source: "".to_string(),
        sequence: 0,
        currency: "".to_string(),
        summary: "".to_string(),
        status: SeasonalEgressScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_egress_schedule("code-683", "name-115");
        assert_eq!(item.status, SeasonalEgressScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_seasonal_egress_schedule("code-683", "name-115");
        if item.move_to(SeasonalEgressScheduleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEgressScheduleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_egress_schedule("code-683", "name-115");
        item.status = SeasonalEgressScheduleStatus::Closed;
        assert!(
            item.move_to(SeasonalEgressScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
