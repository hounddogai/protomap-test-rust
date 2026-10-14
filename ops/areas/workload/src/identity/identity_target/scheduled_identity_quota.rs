use super::*;

use acme_kit::text;

/// Where a scheduled identity quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledIdentityQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledIdentityQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledIdentityQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled identity quota as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityQuota {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub checksum: String,
    pub quota_units: i64,
    pub contact_email: String,
    pub visible: bool,
    pub key_id: String,
    pub status: ScheduledIdentityQuotaStatus,
}

impl ScheduledIdentityQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledIdentityQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledIdentityQuotaStatus) -> bool {
        match self.status {
            ScheduledIdentityQuotaStatus::Idle => {
                return next == ScheduledIdentityQuotaStatus::Warming || next == ScheduledIdentityQuotaStatus::Stopped;
            }
            ScheduledIdentityQuotaStatus::Warming => {
                match next {
                    ScheduledIdentityQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIdentityQuotaStatus::Ready => {
                return next == ScheduledIdentityQuotaStatus::Draining;
            }
            ScheduledIdentityQuotaStatus::Draining => {
                match next {
                    ScheduledIdentityQuotaStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledIdentityQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled identity quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> ScheduledIdentityQuota {
        ScheduledIdentityQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            contact_email: self.contact_email.clone(),
            visible: self.visible,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled identity quota in status idle.
pub fn new_scheduled_identity_quota(code: &str, title: &str) -> ScheduledIdentityQuota {
    ScheduledIdentityQuota {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        contact_email: "".to_string(),
        visible: false,
        key_id: "".to_string(),
        status: ScheduledIdentityQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_identity_quota("code-646", "title-11");
        assert_eq!(item.status, ScheduledIdentityQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_identity_quota("code-646", "title-11");
        if item.move_to(ScheduledIdentityQuotaStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledIdentityQuotaStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_identity_quota("code-646", "title-11");
        item.status = ScheduledIdentityQuotaStatus::Stopped;
        assert!(
            item.move_to(ScheduledIdentityQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
