use super::*;

use acme_kit::text;

/// Where a lenient freeze record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFreezeRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientFreezeRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientFreezeRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient freeze record as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecord {
    pub code: String,
    pub name: String,
    pub category: String,
    pub checksum: String,
    pub minimum: f64,
    pub sequence: i64,
    pub cluster: String,
    pub timezone: String,
    pub status: LenientFreezeRecordStatus,
}

impl LenientFreezeRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFreezeRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFreezeRecordStatus) -> bool {
        match self.status {
            LenientFreezeRecordStatus::Requested => {
                return next == LenientFreezeRecordStatus::Granted || next == LenientFreezeRecordStatus::Expired;
            }
            LenientFreezeRecordStatus::Granted => {
                return next == LenientFreezeRecordStatus::Revoked || next == LenientFreezeRecordStatus::Expired;
            }
            LenientFreezeRecordStatus::Revoked => {
                return next == LenientFreezeRecordStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFreezeRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient freeze record {} cannot move from {} to {}",
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
        ((self.sequence as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFreezeRecord {
        LenientFreezeRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            checksum: self.checksum.clone(),
            minimum: self.minimum,
            sequence: self.sequence,
            cluster: self.cluster.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient freeze record in status requested.
pub fn new_lenient_freeze_record(code: &str, name: &str) -> LenientFreezeRecord {
    LenientFreezeRecord {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        checksum: "".to_string(),
        minimum: 0.0,
        sequence: 0,
        cluster: "".to_string(),
        timezone: "".to_string(),
        status: LenientFreezeRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_freeze_record("code-346", "name-774");
        assert_eq!(item.status, LenientFreezeRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_lenient_freeze_record("code-346", "name-774");
        if item.move_to(LenientFreezeRecordStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFreezeRecordStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_freeze_record("code-346", "name-774");
        item.status = LenientFreezeRecordStatus::Expired;
        assert!(
            item.move_to(LenientFreezeRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
