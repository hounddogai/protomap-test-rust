use super::*;

use acme_kit::text;

/// Where a silence exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SilenceExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SilenceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceException {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub name: String,
    pub maximum: f64,
    pub status: SilenceExceptionStatus,
}

impl SilenceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceExceptionStatus) -> bool {
        match self.status {
            SilenceExceptionStatus::Created => {
                match next {
                    SilenceExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceExceptionStatus::Validated => {
                match next {
                    SilenceExceptionStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceExceptionStatus::Published => {
                return next == SilenceExceptionStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceException {
        SilenceException {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            name: self.name.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence exception in status created.
pub fn new_silence_exception(code: &str, label: &str) -> SilenceException {
    SilenceException {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        name: "".to_string(),
        maximum: 0.0,
        status: SilenceExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_silence_exception("code-328", "label-422");
        assert_eq!(item.status, SilenceExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_silence_exception("code-328", "label-422");
        if item.move_to(SilenceExceptionStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceExceptionStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_silence_exception("code-328", "label-422");
        item.status = SilenceExceptionStatus::Retired;
        assert!(
            item.move_to(SilenceExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
