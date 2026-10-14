use super::*;

use acme_kit::text;

/// Where a nonce template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NonceTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NonceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplate {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub maximum: f64,
    pub budget: f64,
    pub duration_minutes: i64,
    pub status: NonceTemplateStatus,
}

impl NonceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceTemplateStatus) -> bool {
        match self.status {
            NonceTemplateStatus::Requested => {
                match next {
                    NonceTemplateStatus::Granted | NonceTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceTemplateStatus::Granted => {
                match next {
                    NonceTemplateStatus::Revoked | NonceTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceTemplateStatus::Revoked => {
                return next == NonceTemplateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        (self.budget * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceTemplate {
        NonceTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            maximum: self.maximum,
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce template in status requested.
pub fn new_nonce_template(code: &str, name: &str) -> NonceTemplate {
    NonceTemplate {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        maximum: 0.0,
        budget: 0.0,
        duration_minutes: 0,
        status: NonceTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_nonce_template("code-814", "name-276");
        assert_eq!(item.status, NonceTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_nonce_template("code-814", "name-276");
        if item.move_to(NonceTemplateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceTemplateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_nonce_template("code-814", "name-276");
        item.status = NonceTemplateStatus::Expired;
        assert!(
            item.move_to(NonceTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
