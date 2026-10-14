use super::*;

use acme_kit::text;

/// Where a nonce record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl NonceRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for NonceRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRecord {
    pub code: String,
    pub name: String,
    pub country: String,
    pub source: String,
    pub phone: String,
    pub principal: String,
    pub batch_size: i64,
    pub description: String,
    pub owner_id: String,
    pub status: NonceRecordStatus,
}

impl NonceRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceRecordStatus) -> bool {
        match self.status {
            NonceRecordStatus::Proposed => {
                return next == NonceRecordStatus::Withdrawn;
            }
            NonceRecordStatus::Approved => {
                return next == NonceRecordStatus::Rejected || next == NonceRecordStatus::Withdrawn;
            }
            NonceRecordStatus::Rejected => {
                match next {
                    NonceRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: NonceRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> NonceRecord {
        NonceRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            source: self.source.clone(),
            phone: self.phone.clone(),
            principal: self.principal.clone(),
            batch_size: self.batch_size,
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a nonce record in status proposed.
pub fn new_nonce_record(code: &str, name: &str) -> NonceRecord {
    NonceRecord {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        source: "".to_string(),
        phone: "".to_string(),
        principal: "".to_string(),
        batch_size: 0,
        description: "".to_string(),
        owner_id: "".to_string(),
        status: NonceRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_nonce_record("code-454", "name-983");
        assert_eq!(item.status, NonceRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_nonce_record("code-454", "name-983");
        if item.move_to(NonceRecordStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceRecordStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_nonce_record("code-454", "name-983");
        item.status = NonceRecordStatus::Withdrawn;
        assert!(
            item.move_to(NonceRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
