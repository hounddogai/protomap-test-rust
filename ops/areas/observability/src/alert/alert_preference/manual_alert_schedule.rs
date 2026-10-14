use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual alert schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAlertScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualAlertScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualAlertScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual alert schedule as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAlertSchedule {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub principal: String,
    pub region: String,
    pub unit: String,
    pub window_days: i64,
    pub fingerprint: String,
    pub priority: i64,
    pub ends_at: i64,
    pub status: ManualAlertScheduleStatus,
}

impl ManualAlertSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAlertScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAlertScheduleStatus) -> bool {
        match self.status {
            ManualAlertScheduleStatus::Requested => {
                return next == ManualAlertScheduleStatus::Expired;
            }
            ManualAlertScheduleStatus::Granted => {
                match next {
                    ManualAlertScheduleStatus::Revoked | ManualAlertScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAlertScheduleStatus::Revoked => {
                return next == ManualAlertScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualAlertScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual alert schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualAlertSchedule {
        ManualAlertSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            unit: self.unit.clone(),
            window_days: self.window_days,
            fingerprint: self.fingerprint.clone(),
            priority: self.priority,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual alert schedule in status requested.
pub fn new_manual_alert_schedule(code: &str, title: &str) -> ManualAlertSchedule {
    ManualAlertSchedule {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        principal: "".to_string(),
        region: "".to_string(),
        unit: "".to_string(),
        window_days: 0,
        fingerprint: "".to_string(),
        priority: 0,
        ends_at: 0,
        status: ManualAlertScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_alert_schedule("code-148", "title-693");
        assert_eq!(item.status, ManualAlertScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_manual_alert_schedule("code-148", "title-693");
        if item.move_to(ManualAlertScheduleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualAlertScheduleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_alert_schedule("code-148", "title-693");
        item.status = ManualAlertScheduleStatus::Expired;
        assert!(
            item.move_to(ManualAlertScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
