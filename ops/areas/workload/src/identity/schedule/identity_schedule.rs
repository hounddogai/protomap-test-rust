use super::*;

use acme_kit::text;

/// Where a identity schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity schedule as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySchedule {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub country: String,
    pub created_at: i64,
    pub status: IdentityScheduleStatus,
}

impl IdentitySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityScheduleStatus) -> bool {
        match self.status {
            IdentityScheduleStatus::Requested => {
                return next == IdentityScheduleStatus::Granted || next == IdentityScheduleStatus::Revoked;
            }
            IdentityScheduleStatus::Granted => {
                return next == IdentityScheduleStatus::Revoked || next == IdentityScheduleStatus::Expired;
            }
            IdentityScheduleStatus::Revoked => {
                match next {
                    IdentityScheduleStatus::Expired => {
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

    pub fn move_to(&mut self, next: IdentityScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> IdentitySchedule {
        IdentitySchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            country: self.country.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a identity schedule in status requested.
pub fn new_identity_schedule(code: &str, name: &str) -> IdentitySchedule {
    IdentitySchedule {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        country: "".to_string(),
        created_at: 0,
        status: IdentityScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_identity_schedule("code-752", "name-672");
        assert_eq!(item.status, IdentityScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_identity_schedule("code-752", "name-672");
        if item.move_to(IdentityScheduleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityScheduleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_identity_schedule("code-752", "name-672");
        item.status = IdentityScheduleStatus::Expired;
        assert!(
            item.move_to(IdentityScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
