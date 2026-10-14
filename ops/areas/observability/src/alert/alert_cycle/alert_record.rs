use super::*;

use acme_kit::text;

/// Where a alert record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AlertRecordStatus {
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

impl std::fmt::Display for AlertRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecord {
    pub code: String,
    pub name: String,
    pub source: String,
    pub ttl_seconds: i64,
    pub timezone: String,
    pub external_ref: String,
    pub owner_id: String,
    pub segment: String,
    pub approved: bool,
    pub reviewer: String,
    pub amount: f64,
    pub status: AlertRecordStatus,
}

impl AlertRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertRecordStatus) -> bool {
        match self.status {
            AlertRecordStatus::Planned => {
                return next == AlertRecordStatus::InProgress;
            }
            AlertRecordStatus::Scheduled => {
                return next == AlertRecordStatus::Completed;
            }
            AlertRecordStatus::InProgress => {
                match next {
                    AlertRecordStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertRecordStatus::Completed => {
                match next {
                    AlertRecordStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AlertRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertRecord {
        AlertRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            timezone: self.timezone.clone(),
            external_ref: self.external_ref.clone(),
            owner_id: self.owner_id.clone(),
            segment: self.segment.clone(),
            approved: self.approved,
            reviewer: self.reviewer.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a alert record in status planned.
pub fn new_alert_record(code: &str, name: &str) -> AlertRecord {
    AlertRecord {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        ttl_seconds: 0,
        timezone: "".to_string(),
        external_ref: "".to_string(),
        owner_id: "".to_string(),
        segment: "".to_string(),
        approved: false,
        reviewer: "".to_string(),
        amount: 0.0,
        status: AlertRecordStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_alert_record("code-713", "name-221");
        assert_eq!(item.status, AlertRecordStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_alert_record("code-713", "name-221");
        if item.move_to(AlertRecordStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertRecordStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_alert_record("code-713", "name-221");
        item.status = AlertRecordStatus::Cancelled;
        assert!(
            item.move_to(AlertRecordStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
