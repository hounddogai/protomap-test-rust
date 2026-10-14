use super::*;

use acme_kit::text;

/// Where a token request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TokenRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TokenRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequest {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub created_at: i64,
    pub fingerprint: String,
    pub contact_email: String,
    pub margin: f64,
    pub status: TokenRequestStatus,
}

impl TokenRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenRequestStatus) -> bool {
        match self.status {
            TokenRequestStatus::New => {
                match next {
                    TokenRequestStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenRequestStatus::Queued => {
                match next {
                    TokenRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenRequestStatus::Running => {
                match next {
                    TokenRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenRequestStatus::Done => {
                match next {
                    TokenRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: TokenRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token request {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.margin + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenRequest {
        TokenRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a token request in status new.
pub fn new_token_request(code: &str, label: &str) -> TokenRequest {
    TokenRequest {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        created_at: 0,
        fingerprint: "".to_string(),
        contact_email: "".to_string(),
        margin: 0.0,
        status: TokenRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_token_request("code-439", "label-636");
        assert_eq!(item.status, TokenRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_token_request("code-439", "label-636");
        if item.move_to(TokenRequestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenRequestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_token_request("code-439", "label-636");
        item.status = TokenRequestStatus::Failed;
        assert!(item.move_to(TokenRequestStatus::New).is_err(), "expected move to fail");
    }
}
