use super::*;

use acme_kit::text;

/// Where a pending zone record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingZoneRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingZoneRecordStatus {
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

impl std::fmt::Display for PendingZoneRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending zone record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneRecord {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub sequence: i64,
    pub fingerprint: String,
    pub archived: bool,
    pub tenant_id: String,
    pub window_days: i64,
    pub scope: String,
    pub notes: String,
    pub active: bool,
    pub status: PendingZoneRecordStatus,
}

impl PendingZoneRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingZoneRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingZoneRecordStatus) -> bool {
        match self.status {
            PendingZoneRecordStatus::Draft => {
                return next == PendingZoneRecordStatus::Suspended || next == PendingZoneRecordStatus::Closed;
            }
            PendingZoneRecordStatus::Pending => {
                match next {
                    PendingZoneRecordStatus::Active | PendingZoneRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingZoneRecordStatus::Active => {
                match next {
                    PendingZoneRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingZoneRecordStatus::Suspended => {
                return next == PendingZoneRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingZoneRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending zone record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingZoneRecord {
        PendingZoneRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending zone record in status draft.
pub fn new_pending_zone_record(code: &str, name: &str) -> PendingZoneRecord {
    PendingZoneRecord {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        sequence: 0,
        fingerprint: "".to_string(),
        archived: false,
        tenant_id: "".to_string(),
        window_days: 0,
        scope: "".to_string(),
        notes: "".to_string(),
        active: false,
        status: PendingZoneRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_zone_record("code-355", "name-725");
        assert_eq!(item.status, PendingZoneRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_pending_zone_record("code-355", "name-725");
        if item.move_to(PendingZoneRecordStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingZoneRecordStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_zone_record("code-355", "name-725");
        item.status = PendingZoneRecordStatus::Closed;
        assert!(
            item.move_to(PendingZoneRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
