use super::*;

use acme_kit::text;

/// Where a scheduled measurement policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledMeasurementPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledMeasurementPolicyStatus {
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

impl std::fmt::Display for ScheduledMeasurementPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled measurement policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementPolicy {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub quota_units: i64,
    pub expires_at: i64,
    pub tenant_id: String,
    pub segment: String,
    pub visible: bool,
    pub locale: String,
    pub maximum: f64,
    pub status: ScheduledMeasurementPolicyStatus,
}

impl ScheduledMeasurementPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledMeasurementPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledMeasurementPolicyStatus) -> bool {
        match self.status {
            ScheduledMeasurementPolicyStatus::Planned => {
                return next == ScheduledMeasurementPolicyStatus::Completed
                    || next == ScheduledMeasurementPolicyStatus::Cancelled;
            }
            ScheduledMeasurementPolicyStatus::Scheduled => {
                match next {
                    ScheduledMeasurementPolicyStatus::Completed | ScheduledMeasurementPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledMeasurementPolicyStatus::InProgress => {
                match next {
                    ScheduledMeasurementPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledMeasurementPolicyStatus::Completed => {
                return next == ScheduledMeasurementPolicyStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledMeasurementPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled measurement policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledMeasurementPolicy {
        ScheduledMeasurementPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            segment: self.segment.clone(),
            visible: self.visible,
            locale: self.locale.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled measurement policy in status planned.
pub fn new_scheduled_measurement_policy(code: &str, name: &str) -> ScheduledMeasurementPolicy {
    ScheduledMeasurementPolicy {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        expires_at: 0,
        tenant_id: "".to_string(),
        segment: "".to_string(),
        visible: false,
        locale: "".to_string(),
        maximum: 0.0,
        status: ScheduledMeasurementPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_measurement_policy("code-525", "name-542");
        assert_eq!(item.status, ScheduledMeasurementPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_scheduled_measurement_policy("code-525", "name-542");
        if item.move_to(ScheduledMeasurementPolicyStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledMeasurementPolicyStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_measurement_policy("code-525", "name-542");
        item.status = ScheduledMeasurementPolicyStatus::Cancelled;
        assert!(
            item.move_to(ScheduledMeasurementPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
