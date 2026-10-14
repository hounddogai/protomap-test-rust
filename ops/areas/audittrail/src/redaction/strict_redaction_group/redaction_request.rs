use super::*;

use acme_kit::text;

/// Where a redaction request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionRequestStatus {
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

impl std::fmt::Display for RedactionRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequest {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub host: String,
    pub cost: f64,
    pub timezone: String,
    pub currency: String,
    pub status: RedactionRequestStatus,
}

impl RedactionRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionRequestStatus) -> bool {
        match self.status {
            RedactionRequestStatus::Draft => {
                match next {
                    RedactionRequestStatus::Pending | RedactionRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionRequestStatus::Pending => {
                return next == RedactionRequestStatus::Suspended;
            }
            RedactionRequestStatus::Active => {
                return next == RedactionRequestStatus::Suspended || next == RedactionRequestStatus::Closed;
            }
            RedactionRequestStatus::Suspended => {
                return next == RedactionRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> RedactionRequest {
        RedactionRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            host: self.host.clone(),
            cost: self.cost,
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a redaction request in status draft.
pub fn new_redaction_request(code: &str, name: &str) -> RedactionRequest {
    RedactionRequest {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        host: "".to_string(),
        cost: 0.0,
        timezone: "".to_string(),
        currency: "".to_string(),
        status: RedactionRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_redaction_request("code-154", "name-310");
        assert_eq!(item.status, RedactionRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_redaction_request("code-154", "name-310");
        if item.move_to(RedactionRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_redaction_request("code-154", "name-310");
        item.status = RedactionRequestStatus::Closed;
        assert!(
            item.move_to(RedactionRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
