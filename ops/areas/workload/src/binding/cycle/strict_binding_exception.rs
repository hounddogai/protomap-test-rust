use super::*;

use acme_kit::text;

/// Where a strict binding exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictBindingExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictBindingExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictBindingExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict binding exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingException {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub retries: i64,
    pub duration_minutes: i64,
    pub visible: bool,
    pub contact_email: String,
    pub grace_days: i64,
    pub locale: String,
    pub ends_at: i64,
    pub status: StrictBindingExceptionStatus,
}

impl StrictBindingException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictBindingExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictBindingExceptionStatus) -> bool {
        match self.status {
            StrictBindingExceptionStatus::Proposed => {
                return next == StrictBindingExceptionStatus::Rejected;
            }
            StrictBindingExceptionStatus::Approved => {
                match next {
                    StrictBindingExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBindingExceptionStatus::Rejected => {
                return next == StrictBindingExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictBindingExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict binding exception {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictBindingException {
        StrictBindingException {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            retries: self.retries,
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict binding exception in status proposed.
pub fn new_strict_binding_exception(code: &str, name: &str) -> StrictBindingException {
    StrictBindingException {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        retries: 0,
        duration_minutes: 0,
        visible: false,
        contact_email: "".to_string(),
        grace_days: 0,
        locale: "".to_string(),
        ends_at: 0,
        status: StrictBindingExceptionStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_binding_exception("code-925", "name-770");
        assert_eq!(item.status, StrictBindingExceptionStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_strict_binding_exception("code-925", "name-770");
        if item.move_to(StrictBindingExceptionStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictBindingExceptionStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_binding_exception("code-925", "name-770");
        item.status = StrictBindingExceptionStatus::Withdrawn;
        assert!(
            item.move_to(StrictBindingExceptionStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
