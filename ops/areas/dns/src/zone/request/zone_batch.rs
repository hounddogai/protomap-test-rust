use super::*;

use acme_kit::text;

/// Where a zone batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone batch as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatch {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub phone: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub revision: i64,
    pub pinned: bool,
    pub status: ZoneBatchStatus,
}

impl ZoneBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneBatchStatus) -> bool {
        match self.status {
            ZoneBatchStatus::Proposed => {
                return next == ZoneBatchStatus::Approved || next == ZoneBatchStatus::Withdrawn;
            }
            ZoneBatchStatus::Approved => {
                match next {
                    ZoneBatchStatus::Rejected | ZoneBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneBatchStatus::Rejected => {
                match next {
                    ZoneBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ZoneBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneBatch {
        ZoneBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            phone: self.phone.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a zone batch in status proposed.
pub fn new_zone_batch(code: &str, name: &str) -> ZoneBatch {
    ZoneBatch {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        phone: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        revision: 0,
        pinned: false,
        status: ZoneBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_batch("code-722", "name-778");
        assert_eq!(item.status, ZoneBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_zone_batch("code-722", "name-778");
        if item.move_to(ZoneBatchStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneBatchStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_batch("code-722", "name-778");
        item.status = ZoneBatchStatus::Withdrawn;
        assert!(
            item.move_to(ZoneBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
