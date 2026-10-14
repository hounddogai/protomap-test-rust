use super::*;

use acme_kit::text;

/// Where a weighted token schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTokenScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedTokenScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedTokenScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted token schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenSchedule {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub created_at: i64,
    pub approved: bool,
    pub status: WeightedTokenScheduleStatus,
}

impl WeightedTokenSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTokenScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTokenScheduleStatus) -> bool {
        match self.status {
            WeightedTokenScheduleStatus::Requested => {
                match next {
                    WeightedTokenScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenScheduleStatus::Granted => {
                return next == WeightedTokenScheduleStatus::Revoked || next == WeightedTokenScheduleStatus::Expired;
            }
            WeightedTokenScheduleStatus::Revoked => {
                return next == WeightedTokenScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTokenScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted token schedule {} cannot move from {} to {}",
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

    pub fn copy(&self) -> WeightedTokenSchedule {
        WeightedTokenSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            created_at: self.created_at,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted token schedule in status requested.
pub fn new_weighted_token_schedule(code: &str, name: &str) -> WeightedTokenSchedule {
    WeightedTokenSchedule {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        created_at: 0,
        approved: false,
        status: WeightedTokenScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_token_schedule("code-679", "name-644");
        assert_eq!(item.status, WeightedTokenScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_weighted_token_schedule("code-679", "name-644");
        if item.move_to(WeightedTokenScheduleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTokenScheduleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_token_schedule("code-679", "name-644");
        item.status = WeightedTokenScheduleStatus::Expired;
        assert!(
            item.move_to(WeightedTokenScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
