use super::*;

use acme_kit::text;

/// Where a global token exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTokenExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalTokenExceptionStatus {
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

impl std::fmt::Display for GlobalTokenExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global token exception as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenException {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub description: String,
    pub attempts: i64,
    pub limit: i64,
    pub score: f64,
    pub issuer: String,
    pub status: GlobalTokenExceptionStatus,
}

impl GlobalTokenException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTokenExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTokenExceptionStatus) -> bool {
        match self.status {
            GlobalTokenExceptionStatus::Draft => {
                match next {
                    GlobalTokenExceptionStatus::Pending | GlobalTokenExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenExceptionStatus::Pending => {
                match next {
                    GlobalTokenExceptionStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenExceptionStatus::Active => {
                match next {
                    GlobalTokenExceptionStatus::Suspended | GlobalTokenExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenExceptionStatus::Suspended => {
                return next == GlobalTokenExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTokenExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global token exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global token exception", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTokenException {
        GlobalTokenException {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            description: self.description.clone(),
            attempts: self.attempts,
            limit: self.limit,
            score: self.score,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global token exception in status draft.
pub fn new_global_token_exception(code: &str, name: &str) -> GlobalTokenException {
    GlobalTokenException {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        description: "".to_string(),
        attempts: 0,
        limit: 0,
        score: 0.0,
        issuer: "".to_string(),
        status: GlobalTokenExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_token_exception("code-167", "name-741");
        assert_eq!(item.status, GlobalTokenExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_token_exception("code-167", "name-741");
        if item.move_to(GlobalTokenExceptionStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTokenExceptionStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_token_exception("code-167", "name-741");
        item.status = GlobalTokenExceptionStatus::Closed;
        assert!(
            item.move_to(GlobalTokenExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
