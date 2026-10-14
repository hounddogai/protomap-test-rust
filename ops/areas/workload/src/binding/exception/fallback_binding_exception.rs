use super::*;

use acme_kit::text;

/// Where a fallback binding exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBindingExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackBindingExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackBindingExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback binding exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingException {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub grace_days: i64,
    pub checksum: String,
    pub phone: String,
    pub status: FallbackBindingExceptionStatus,
}

impl FallbackBindingException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBindingExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBindingExceptionStatus) -> bool {
        match self.status {
            FallbackBindingExceptionStatus::Planned => {
                match next {
                    FallbackBindingExceptionStatus::InProgress | FallbackBindingExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingExceptionStatus::Scheduled => {
                return next == FallbackBindingExceptionStatus::InProgress
                    || next == FallbackBindingExceptionStatus::Completed;
            }
            FallbackBindingExceptionStatus::InProgress => {
                match next {
                    FallbackBindingExceptionStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingExceptionStatus::Completed => {
                match next {
                    FallbackBindingExceptionStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackBindingExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback binding exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback binding exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> FallbackBindingException {
        FallbackBindingException {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback binding exception in status planned.
pub fn new_fallback_binding_exception(code: &str, title: &str) -> FallbackBindingException {
    FallbackBindingException {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        grace_days: 0,
        checksum: "".to_string(),
        phone: "".to_string(),
        status: FallbackBindingExceptionStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_binding_exception("code-192", "title-357");
        assert_eq!(item.status, FallbackBindingExceptionStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_fallback_binding_exception("code-192", "title-357");
        if item.move_to(FallbackBindingExceptionStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBindingExceptionStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_binding_exception("code-192", "title-357");
        item.status = FallbackBindingExceptionStatus::Cancelled;
        assert!(
            item.move_to(FallbackBindingExceptionStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
