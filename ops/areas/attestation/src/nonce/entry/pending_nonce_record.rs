use super::*;

use acme_kit::text;

/// Where a pending nonce record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingNonceRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingNonceRecordStatus {
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

impl std::fmt::Display for PendingNonceRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending nonce record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecord {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub limit: i64,
    pub margin: f64,
    pub owner_id: String,
    pub status: PendingNonceRecordStatus,
}

impl PendingNonceRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingNonceRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingNonceRecordStatus) -> bool {
        match self.status {
            PendingNonceRecordStatus::Draft => {
                return next == PendingNonceRecordStatus::Pending;
            }
            PendingNonceRecordStatus::Pending => {
                match next {
                    PendingNonceRecordStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceRecordStatus::Active => {
                match next {
                    PendingNonceRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceRecordStatus::Suspended => {
                return next == PendingNonceRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingNonceRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending nonce record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending nonce record", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.floor_value + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingNonceRecord {
        PendingNonceRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            margin: self.margin,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending nonce record in status draft.
pub fn new_pending_nonce_record(code: &str, name: &str) -> PendingNonceRecord {
    PendingNonceRecord {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        limit: 0,
        margin: 0.0,
        owner_id: "".to_string(),
        status: PendingNonceRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_nonce_record("code-891", "name-600");
        assert_eq!(item.status, PendingNonceRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_nonce_record("code-891", "name-600");
        if item.move_to(PendingNonceRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingNonceRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_nonce_record("code-891", "name-600");
        item.status = PendingNonceRecordStatus::Closed;
        assert!(
            item.move_to(PendingNonceRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
