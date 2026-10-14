use super::*;

use acme_kit::text;

/// Where a regional silence exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSilenceExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalSilenceExceptionStatus {
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

impl std::fmt::Display for RegionalSilenceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional silence exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceException {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub currency: String,
    pub unit: String,
    pub namespace: String,
    pub attempts: i64,
    pub label: String,
    pub active: bool,
    pub expires_at: i64,
    pub status: RegionalSilenceExceptionStatus,
}

impl RegionalSilenceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSilenceExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSilenceExceptionStatus) -> bool {
        match self.status {
            RegionalSilenceExceptionStatus::Draft => {
                return next == RegionalSilenceExceptionStatus::Pending
                    || next == RegionalSilenceExceptionStatus::Closed;
            }
            RegionalSilenceExceptionStatus::Pending => {
                return next == RegionalSilenceExceptionStatus::Active
                    || next == RegionalSilenceExceptionStatus::Suspended;
            }
            RegionalSilenceExceptionStatus::Active => {
                match next {
                    RegionalSilenceExceptionStatus::Suspended | RegionalSilenceExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSilenceExceptionStatus::Suspended => {
                return next == RegionalSilenceExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSilenceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional silence exception {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSilenceException {
        RegionalSilenceException {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            currency: self.currency.clone(),
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            attempts: self.attempts,
            label: self.label.clone(),
            active: self.active,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a regional silence exception in status draft.
pub fn new_regional_silence_exception(code: &str, name: &str) -> RegionalSilenceException {
    RegionalSilenceException {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        currency: "".to_string(),
        unit: "".to_string(),
        namespace: "".to_string(),
        attempts: 0,
        label: "".to_string(),
        active: false,
        expires_at: 0,
        status: RegionalSilenceExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_silence_exception("code-233", "name-97");
        assert_eq!(item.status, RegionalSilenceExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_regional_silence_exception("code-233", "name-97");
        if item.move_to(RegionalSilenceExceptionStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSilenceExceptionStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_silence_exception("code-233", "name-97");
        item.status = RegionalSilenceExceptionStatus::Closed;
        assert!(
            item.move_to(RegionalSilenceExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
