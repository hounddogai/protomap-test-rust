use super::*;

use acme_kit::text;

/// Where a weighted alert entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedAlertEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedAlertEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedAlertEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted alert entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertEntry {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub tenant_id: String,
    pub attempts: i64,
    pub country: String,
    pub status: WeightedAlertEntryStatus,
}

impl WeightedAlertEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedAlertEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedAlertEntryStatus) -> bool {
        match self.status {
            WeightedAlertEntryStatus::Created => {
                return next == WeightedAlertEntryStatus::Retired;
            }
            WeightedAlertEntryStatus::Validated => {
                match next {
                    WeightedAlertEntryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertEntryStatus::Published => {
                match next {
                    WeightedAlertEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: WeightedAlertEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted alert entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted alert entry", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.attempts as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedAlertEntry {
        WeightedAlertEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted alert entry in status created.
pub fn new_weighted_alert_entry(code: &str, title: &str) -> WeightedAlertEntry {
    WeightedAlertEntry {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        tenant_id: "".to_string(),
        attempts: 0,
        country: "".to_string(),
        status: WeightedAlertEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_alert_entry("code-599", "title-666");
        assert_eq!(item.status, WeightedAlertEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_weighted_alert_entry("code-599", "title-666");
        if item.move_to(WeightedAlertEntryStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedAlertEntryStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_alert_entry("code-599", "title-666");
        item.status = WeightedAlertEntryStatus::Retired;
        assert!(
            item.move_to(WeightedAlertEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
