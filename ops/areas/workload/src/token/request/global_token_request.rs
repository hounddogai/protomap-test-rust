use super::*;

use acme_kit::text;

/// Where a global token request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTokenRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalTokenRequestStatus {
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

impl std::fmt::Display for GlobalTokenRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global token request as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequest {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub amount: f64,
    pub grace_days: i64,
    pub visible: bool,
    pub created_at: i64,
    pub ttl_seconds: i64,
    pub contact_email: String,
    pub label: String,
    pub status: GlobalTokenRequestStatus,
}

impl GlobalTokenRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTokenRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTokenRequestStatus) -> bool {
        match self.status {
            GlobalTokenRequestStatus::Draft => {
                match next {
                    GlobalTokenRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenRequestStatus::Pending => {
                match next {
                    GlobalTokenRequestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenRequestStatus::Active => {
                match next {
                    GlobalTokenRequestStatus::Suspended | GlobalTokenRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenRequestStatus::Suspended => {
                return next == GlobalTokenRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTokenRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global token request {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.amount + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTokenRequest {
        GlobalTokenRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            amount: self.amount,
            grace_days: self.grace_days,
            visible: self.visible,
            created_at: self.created_at,
            ttl_seconds: self.ttl_seconds,
            contact_email: self.contact_email.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global token request in status draft.
pub fn new_global_token_request(code: &str, title: &str) -> GlobalTokenRequest {
    GlobalTokenRequest {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        amount: 0.0,
        grace_days: 0,
        visible: false,
        created_at: 0,
        ttl_seconds: 0,
        contact_email: "".to_string(),
        label: "".to_string(),
        status: GlobalTokenRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_token_request("code-483", "title-231");
        assert_eq!(item.status, GlobalTokenRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_global_token_request("code-483", "title-231");
        if item.move_to(GlobalTokenRequestStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTokenRequestStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_token_request("code-483", "title-231");
        item.status = GlobalTokenRequestStatus::Closed;
        assert!(
            item.move_to(GlobalTokenRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
