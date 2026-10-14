use super::*;

use acme_kit::text;

/// Where a ttl policy schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyScheduleStatus {
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

impl std::fmt::Display for TtlPolicyScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySchedule {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub currency: String,
    pub contact_email: String,
    pub name: String,
    pub maximum: f64,
    pub port: i64,
    pub scope: String,
    pub attempts: i64,
    pub status: TtlPolicyScheduleStatus,
}

impl TtlPolicySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyScheduleStatus) -> bool {
        match self.status {
            TtlPolicyScheduleStatus::Draft => {
                match next {
                    TtlPolicyScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyScheduleStatus::Pending => {
                match next {
                    TtlPolicyScheduleStatus::Suspended | TtlPolicyScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyScheduleStatus::Active => {
                match next {
                    TtlPolicyScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyScheduleStatus::Suspended => {
                return next == TtlPolicyScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicySchedule {
        TtlPolicySchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            contact_email: self.contact_email.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            port: self.port,
            scope: self.scope.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a ttl policy schedule in status draft.
pub fn new_ttl_policy_schedule(code: &str, title: &str) -> TtlPolicySchedule {
    TtlPolicySchedule {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        currency: "".to_string(),
        contact_email: "".to_string(),
        name: "".to_string(),
        maximum: 0.0,
        port: 0,
        scope: "".to_string(),
        attempts: 0,
        status: TtlPolicyScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_schedule("code-362", "title-891");
        assert_eq!(item.status, TtlPolicyScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_ttl_policy_schedule("code-362", "title-891");
        if item.move_to(TtlPolicyScheduleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyScheduleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_schedule("code-362", "title-891");
        item.status = TtlPolicyScheduleStatus::Closed;
        assert!(
            item.move_to(TtlPolicyScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
