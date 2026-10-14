use super::*;

use acme_kit::text;

/// Where a lenient zone snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientZoneSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientZoneSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientZoneSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient zone snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshot {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub currency: String,
    pub port: i64,
    pub tenant_id: String,
    pub ends_at: i64,
    pub sequence: i64,
    pub pinned: bool,
    pub phone: String,
    pub status: LenientZoneSnapshotStatus,
}

impl LenientZoneSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientZoneSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientZoneSnapshotStatus) -> bool {
        match self.status {
            LenientZoneSnapshotStatus::Requested => {
                match next {
                    LenientZoneSnapshotStatus::Granted | LenientZoneSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientZoneSnapshotStatus::Granted => {
                return next == LenientZoneSnapshotStatus::Revoked || next == LenientZoneSnapshotStatus::Expired;
            }
            LenientZoneSnapshotStatus::Revoked => {
                return next == LenientZoneSnapshotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientZoneSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient zone snapshot {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.port as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientZoneSnapshot {
        LenientZoneSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            currency: self.currency.clone(),
            port: self.port,
            tenant_id: self.tenant_id.clone(),
            ends_at: self.ends_at,
            sequence: self.sequence,
            pinned: self.pinned,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient zone snapshot in status requested.
pub fn new_lenient_zone_snapshot(code: &str, label: &str) -> LenientZoneSnapshot {
    LenientZoneSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        currency: "".to_string(),
        port: 0,
        tenant_id: "".to_string(),
        ends_at: 0,
        sequence: 0,
        pinned: false,
        phone: "".to_string(),
        status: LenientZoneSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_zone_snapshot("code-202", "label-413");
        assert_eq!(item.status, LenientZoneSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_lenient_zone_snapshot("code-202", "label-413");
        if item.move_to(LenientZoneSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientZoneSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_zone_snapshot("code-202", "label-413");
        item.status = LenientZoneSnapshotStatus::Expired;
        assert!(
            item.move_to(LenientZoneSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
