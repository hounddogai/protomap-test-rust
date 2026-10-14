use super::*;

use acme_kit::text;

/// Where a scheduled freeze override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFreezeOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledFreezeOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledFreezeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled freeze override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeOverride {
    pub code: String,
    pub label: String,
    pub title: String,
    pub grace_days: i64,
    pub window_days: i64,
    pub external_ref: String,
    pub checksum: String,
    pub floor_value: f64,
    pub starts_at: i64,
    pub rate: f64,
    pub amount: f64,
    pub status: ScheduledFreezeOverrideStatus,
}

impl ScheduledFreezeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFreezeOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFreezeOverrideStatus) -> bool {
        match self.status {
            ScheduledFreezeOverrideStatus::Requested => {
                return next == ScheduledFreezeOverrideStatus::Expired;
            }
            ScheduledFreezeOverrideStatus::Granted => {
                match next {
                    ScheduledFreezeOverrideStatus::Revoked | ScheduledFreezeOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFreezeOverrideStatus::Revoked => {
                return next == ScheduledFreezeOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFreezeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled freeze override {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.floor_value + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFreezeOverride {
        ScheduledFreezeOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            window_days: self.window_days,
            external_ref: self.external_ref.clone(),
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            starts_at: self.starts_at,
            rate: self.rate,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled freeze override in status requested.
pub fn new_scheduled_freeze_override(code: &str, label: &str) -> ScheduledFreezeOverride {
    ScheduledFreezeOverride {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        grace_days: 0,
        window_days: 0,
        external_ref: "".to_string(),
        checksum: "".to_string(),
        floor_value: 0.0,
        starts_at: 0,
        rate: 0.0,
        amount: 0.0,
        status: ScheduledFreezeOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_freeze_override("code-684", "label-417");
        assert_eq!(item.status, ScheduledFreezeOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_scheduled_freeze_override("code-684", "label-417");
        if item.move_to(ScheduledFreezeOverrideStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFreezeOverrideStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_freeze_override("code-684", "label-417");
        item.status = ScheduledFreezeOverrideStatus::Expired;
        assert!(
            item.move_to(ScheduledFreezeOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
