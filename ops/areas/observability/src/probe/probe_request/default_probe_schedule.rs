use super::*;

use acme_kit::text;

/// Where a default probe schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultProbeScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultProbeScheduleStatus {
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

impl std::fmt::Display for DefaultProbeScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default probe schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeSchedule {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub capacity: i64,
    pub checksum: String,
    pub fingerprint: String,
    pub quantity: i64,
    pub region: String,
    pub tenant_id: String,
    pub ends_at: i64,
    pub discount: f64,
    pub status: DefaultProbeScheduleStatus,
}

impl DefaultProbeSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultProbeScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultProbeScheduleStatus) -> bool {
        match self.status {
            DefaultProbeScheduleStatus::Open => {
                match next {
                    DefaultProbeScheduleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultProbeScheduleStatus::InReview => {
                return next == DefaultProbeScheduleStatus::Archived;
            }
            DefaultProbeScheduleStatus::Resolved => {
                match next {
                    DefaultProbeScheduleStatus::Reopened | DefaultProbeScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultProbeScheduleStatus::Reopened => {
                match next {
                    DefaultProbeScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultProbeScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default probe schedule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "default probe schedule", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.discount - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultProbeSchedule {
        DefaultProbeSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            capacity: self.capacity,
            checksum: self.checksum.clone(),
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            ends_at: self.ends_at,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default probe schedule in status open.
pub fn new_default_probe_schedule(code: &str, label: &str) -> DefaultProbeSchedule {
    DefaultProbeSchedule {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        capacity: 0,
        checksum: "".to_string(),
        fingerprint: "".to_string(),
        quantity: 0,
        region: "".to_string(),
        tenant_id: "".to_string(),
        ends_at: 0,
        discount: 0.0,
        status: DefaultProbeScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_probe_schedule("code-200", "label-982");
        assert_eq!(item.status, DefaultProbeScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_default_probe_schedule("code-200", "label-982");
        if item.move_to(DefaultProbeScheduleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultProbeScheduleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_probe_schedule("code-200", "label-982");
        item.status = DefaultProbeScheduleStatus::Archived;
        assert!(
            item.move_to(DefaultProbeScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
