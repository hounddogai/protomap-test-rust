use super::*;

use acme_kit::text;

/// Where a lenient egress record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientEgressRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientEgressRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient egress record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecord {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub issuer: String,
    pub category: String,
    pub unit: String,
    pub batch_size: i64,
    pub reviewer: String,
    pub grace_days: i64,
    pub status: LenientEgressRecordStatus,
}

impl LenientEgressRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressRecordStatus) -> bool {
        match self.status {
            LenientEgressRecordStatus::Requested => {
                match next {
                    LenientEgressRecordStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressRecordStatus::Granted => {
                match next {
                    LenientEgressRecordStatus::Revoked | LenientEgressRecordStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressRecordStatus::Revoked => {
                match next {
                    LenientEgressRecordStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientEgressRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientEgressRecord {
        LenientEgressRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            category: self.category.clone(),
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient egress record in status requested.
pub fn new_lenient_egress_record(code: &str, name: &str) -> LenientEgressRecord {
    LenientEgressRecord {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        issuer: "".to_string(),
        category: "".to_string(),
        unit: "".to_string(),
        batch_size: 0,
        reviewer: "".to_string(),
        grace_days: 0,
        status: LenientEgressRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_egress_record("code-225", "name-8");
        assert_eq!(item.status, LenientEgressRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_egress_record("code-225", "name-8");
        if item.move_to(LenientEgressRecordStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEgressRecordStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_egress_record("code-225", "name-8");
        item.status = LenientEgressRecordStatus::Expired;
        assert!(
            item.move_to(LenientEgressRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
