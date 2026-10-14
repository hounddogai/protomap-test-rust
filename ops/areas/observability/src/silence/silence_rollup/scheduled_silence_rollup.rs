use super::*;

use acme_kit::text;

/// Where a scheduled silence rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSilenceRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledSilenceRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledSilenceRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled silence rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollup {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub title: String,
    pub checksum: String,
    pub fingerprint: String,
    pub minimum: f64,
    pub rate: f64,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub priority: i64,
    pub status: ScheduledSilenceRollupStatus,
}

impl ScheduledSilenceRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSilenceRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSilenceRollupStatus) -> bool {
        match self.status {
            ScheduledSilenceRollupStatus::Requested => {
                return next == ScheduledSilenceRollupStatus::Expired;
            }
            ScheduledSilenceRollupStatus::Granted => {
                return next == ScheduledSilenceRollupStatus::Revoked || next == ScheduledSilenceRollupStatus::Expired;
            }
            ScheduledSilenceRollupStatus::Revoked => {
                return next == ScheduledSilenceRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSilenceRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled silence rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSilenceRollup {
        ScheduledSilenceRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            fingerprint: self.fingerprint.clone(),
            minimum: self.minimum,
            rate: self.rate,
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled silence rollup in status requested.
pub fn new_scheduled_silence_rollup(code: &str, name: &str) -> ScheduledSilenceRollup {
    ScheduledSilenceRollup {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        title: "".to_string(),
        checksum: "".to_string(),
        fingerprint: "".to_string(),
        minimum: 0.0,
        rate: 0.0,
        rotation_days: 0,
        duration_minutes: 0,
        priority: 0,
        status: ScheduledSilenceRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_silence_rollup("code-89", "name-509");
        assert_eq!(item.status, ScheduledSilenceRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_scheduled_silence_rollup("code-89", "name-509");
        if item.move_to(ScheduledSilenceRollupStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSilenceRollupStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_silence_rollup("code-89", "name-509");
        item.status = ScheduledSilenceRollupStatus::Expired;
        assert!(
            item.move_to(ScheduledSilenceRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
