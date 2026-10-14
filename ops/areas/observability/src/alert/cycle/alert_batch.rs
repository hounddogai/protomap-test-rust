use super::*;

use acme_kit::text;

/// Where a alert batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AlertBatchStatus {
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

impl std::fmt::Display for AlertBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatch {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub starts_at: i64,
    pub locale: String,
    pub amount: f64,
    pub visible: bool,
    pub timezone: String,
    pub retries: i64,
    pub weight: f64,
    pub status: AlertBatchStatus,
}

impl AlertBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertBatchStatus) -> bool {
        match self.status {
            AlertBatchStatus::Draft => {
                match next {
                    AlertBatchStatus::Pending | AlertBatchStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBatchStatus::Pending => {
                match next {
                    AlertBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBatchStatus::Active => {
                match next {
                    AlertBatchStatus::Suspended | AlertBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBatchStatus::Suspended => {
                match next {
                    AlertBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: AlertBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert batch {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.weight + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertBatch {
        AlertBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            amount: self.amount,
            visible: self.visible,
            timezone: self.timezone.clone(),
            retries: self.retries,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a alert batch in status draft.
pub fn new_alert_batch(code: &str, name: &str) -> AlertBatch {
    AlertBatch {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        starts_at: 0,
        locale: "".to_string(),
        amount: 0.0,
        visible: false,
        timezone: "".to_string(),
        retries: 0,
        weight: 0.0,
        status: AlertBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_alert_batch("code-328", "name-406");
        assert_eq!(item.status, AlertBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_alert_batch("code-328", "name-406");
        if item.move_to(AlertBatchStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertBatchStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_alert_batch("code-328", "name-406");
        item.status = AlertBatchStatus::Closed;
        assert!(item.move_to(AlertBatchStatus::Draft).is_err(), "expected move to fail");
    }
}
