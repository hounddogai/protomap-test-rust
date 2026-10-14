use super::*;

use acme_kit::text;

/// Where a approval exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalException {
    pub code: String,
    pub title: String,
    pub country: String,
    pub host: String,
    pub attempts: i64,
    pub timezone: String,
    pub discount: f64,
    pub status: ApprovalExceptionStatus,
}

impl ApprovalException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalExceptionStatus) -> bool {
        match self.status {
            ApprovalExceptionStatus::Created => {
                match next {
                    ApprovalExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalExceptionStatus::Validated => {
                return next == ApprovalExceptionStatus::Published;
            }
            ApprovalExceptionStatus::Published => {
                return next == ApprovalExceptionStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval exception", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.discount - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalException {
        ApprovalException {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            host: self.host.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval exception in status created.
pub fn new_approval_exception(code: &str, title: &str) -> ApprovalException {
    ApprovalException {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        host: "".to_string(),
        attempts: 0,
        timezone: "".to_string(),
        discount: 0.0,
        status: ApprovalExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_approval_exception("code-987", "title-253");
        assert_eq!(item.status, ApprovalExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_approval_exception("code-987", "title-253");
        if item.move_to(ApprovalExceptionStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalExceptionStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_approval_exception("code-987", "title-253");
        item.status = ApprovalExceptionStatus::Retired;
        assert!(
            item.move_to(ApprovalExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
