use super::*;

use acme_kit::text;

/// Where a alert schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AlertScheduleStatus {
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

impl std::fmt::Display for AlertScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert schedule as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertSchedule {
    pub code: String,
    pub label: String,
    pub description: String,
    pub external_ref: String,
    pub rate: f64,
    pub discount: f64,
    pub title: String,
    pub ttl_seconds: i64,
    pub status: AlertScheduleStatus,
}

impl AlertSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertScheduleStatus) -> bool {
        match self.status {
            AlertScheduleStatus::Draft => {
                return next == AlertScheduleStatus::Active;
            }
            AlertScheduleStatus::Pending => {
                match next {
                    AlertScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertScheduleStatus::Active => {
                return next == AlertScheduleStatus::Suspended || next == AlertScheduleStatus::Closed;
            }
            AlertScheduleStatus::Suspended => {
                match next {
                    AlertScheduleStatus::Closed => {
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

    pub fn move_to(&mut self, next: AlertScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.discount - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertSchedule {
        AlertSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            discount: self.discount,
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert schedule in status draft.
pub fn new_alert_schedule(code: &str, label: &str) -> AlertSchedule {
    AlertSchedule {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        external_ref: "".to_string(),
        rate: 0.0,
        discount: 0.0,
        title: "".to_string(),
        ttl_seconds: 0,
        status: AlertScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_alert_schedule("code-131", "label-123");
        assert_eq!(item.status, AlertScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_alert_schedule("code-131", "label-123");
        if item.move_to(AlertScheduleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertScheduleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_alert_schedule("code-131", "label-123");
        item.status = AlertScheduleStatus::Closed;
        assert!(
            item.move_to(AlertScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
