use super::*;

use acme_kit::text;

/// Where a seasonal alert record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAlertRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalAlertRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SeasonalAlertRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal alert record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertRecord {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub limit: i64,
    pub host: String,
    pub maximum: f64,
    pub batch_size: i64,
    pub contact_email: String,
    pub starts_at: i64,
    pub status: SeasonalAlertRecordStatus,
}

impl SeasonalAlertRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAlertRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAlertRecordStatus) -> bool {
        match self.status {
            SeasonalAlertRecordStatus::Open => {
                return next == SeasonalAlertRecordStatus::Reopened;
            }
            SeasonalAlertRecordStatus::InReview => {
                return next == SeasonalAlertRecordStatus::Archived;
            }
            SeasonalAlertRecordStatus::Resolved => {
                return next == SeasonalAlertRecordStatus::Reopened || next == SeasonalAlertRecordStatus::Archived;
            }
            SeasonalAlertRecordStatus::Reopened => {
                match next {
                    SeasonalAlertRecordStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalAlertRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal alert record {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAlertRecord {
        SeasonalAlertRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            limit: self.limit,
            host: self.host.clone(),
            maximum: self.maximum,
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal alert record in status open.
pub fn new_seasonal_alert_record(code: &str, name: &str) -> SeasonalAlertRecord {
    SeasonalAlertRecord {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        limit: 0,
        host: "".to_string(),
        maximum: 0.0,
        batch_size: 0,
        contact_email: "".to_string(),
        starts_at: 0,
        status: SeasonalAlertRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_alert_record("code-571", "name-98");
        assert_eq!(item.status, SeasonalAlertRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_seasonal_alert_record("code-571", "name-98");
        if item.move_to(SeasonalAlertRecordStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAlertRecordStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_alert_record("code-571", "name-98");
        item.status = SeasonalAlertRecordStatus::Archived;
        assert!(
            item.move_to(SeasonalAlertRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
