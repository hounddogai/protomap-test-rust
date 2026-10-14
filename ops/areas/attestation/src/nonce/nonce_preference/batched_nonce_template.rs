use super::*;

use acme_kit::text;

/// Where a batched nonce template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedNonceTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedNonceTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedNonceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched nonce template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceTemplate {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub fingerprint: String,
    pub source: String,
    pub label: String,
    pub name: String,
    pub attempts: i64,
    pub status: BatchedNonceTemplateStatus,
}

impl BatchedNonceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedNonceTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedNonceTemplateStatus) -> bool {
        match self.status {
            BatchedNonceTemplateStatus::Created => {
                match next {
                    BatchedNonceTemplateStatus::Validated | BatchedNonceTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedNonceTemplateStatus::Validated => {
                return next == BatchedNonceTemplateStatus::Published;
            }
            BatchedNonceTemplateStatus::Published => {
                match next {
                    BatchedNonceTemplateStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedNonceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched nonce template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched nonce template", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedNonceTemplate {
        BatchedNonceTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a batched nonce template in status created.
pub fn new_batched_nonce_template(code: &str, title: &str) -> BatchedNonceTemplate {
    BatchedNonceTemplate {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        fingerprint: "".to_string(),
        source: "".to_string(),
        label: "".to_string(),
        name: "".to_string(),
        attempts: 0,
        status: BatchedNonceTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_nonce_template("code-325", "title-810");
        assert_eq!(item.status, BatchedNonceTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_nonce_template("code-325", "title-810");
        if item.move_to(BatchedNonceTemplateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedNonceTemplateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_nonce_template("code-325", "title-810");
        item.status = BatchedNonceTemplateStatus::Retired;
        assert!(
            item.move_to(BatchedNonceTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
