use super::*;

use acme_kit::text;

/// Where a default egress snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEgressSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultEgressSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultEgressSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default egress snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshot {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub retries: i64,
    pub unit: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub status: DefaultEgressSnapshotStatus,
}

impl DefaultEgressSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEgressSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEgressSnapshotStatus) -> bool {
        match self.status {
            DefaultEgressSnapshotStatus::Requested => {
                match next {
                    DefaultEgressSnapshotStatus::Granted | DefaultEgressSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEgressSnapshotStatus::Granted => {
                return next == DefaultEgressSnapshotStatus::Expired;
            }
            DefaultEgressSnapshotStatus::Revoked => {
                match next {
                    DefaultEgressSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultEgressSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default egress snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEgressSnapshot {
        DefaultEgressSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            retries: self.retries,
            unit: self.unit.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a default egress snapshot in status requested.
pub fn new_default_egress_snapshot(code: &str, title: &str) -> DefaultEgressSnapshot {
    DefaultEgressSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        retries: 0,
        unit: "".to_string(),
        label: "".to_string(),
        ttl_seconds: 0,
        status: DefaultEgressSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_egress_snapshot("code-266", "title-13");
        assert_eq!(item.status, DefaultEgressSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_egress_snapshot("code-266", "title-13");
        if item.move_to(DefaultEgressSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEgressSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_egress_snapshot("code-266", "title-13");
        item.status = DefaultEgressSnapshotStatus::Expired;
        assert!(
            item.move_to(DefaultEgressSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
