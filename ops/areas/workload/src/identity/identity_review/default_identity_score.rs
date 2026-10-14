use super::*;

use acme_kit::text;

/// Where a default identity score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIdentityScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultIdentityScoreStatus {
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

impl std::fmt::Display for DefaultIdentityScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default identity score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScore {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub quota_units: i64,
    pub window_days: i64,
    pub locale: String,
    pub attempts: i64,
    pub status: DefaultIdentityScoreStatus,
}

impl DefaultIdentityScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIdentityScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIdentityScoreStatus) -> bool {
        match self.status {
            DefaultIdentityScoreStatus::Draft => {
                match next {
                    DefaultIdentityScoreStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIdentityScoreStatus::Pending => {
                match next {
                    DefaultIdentityScoreStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIdentityScoreStatus::Active => {
                return next == DefaultIdentityScoreStatus::Closed;
            }
            DefaultIdentityScoreStatus::Suspended => {
                return next == DefaultIdentityScoreStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultIdentityScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default identity score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default identity score", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIdentityScore {
        DefaultIdentityScore {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            quota_units: self.quota_units,
            window_days: self.window_days,
            locale: self.locale.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default identity score in status draft.
pub fn new_default_identity_score(code: &str, title: &str) -> DefaultIdentityScore {
    DefaultIdentityScore {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        quota_units: 0,
        window_days: 0,
        locale: "".to_string(),
        attempts: 0,
        status: DefaultIdentityScoreStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_identity_score("code-878", "title-77");
        assert_eq!(item.status, DefaultIdentityScoreStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_default_identity_score("code-878", "title-77");
        if item.move_to(DefaultIdentityScoreStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultIdentityScoreStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_identity_score("code-878", "title-77");
        item.status = DefaultIdentityScoreStatus::Closed;
        assert!(
            item.move_to(DefaultIdentityScoreStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
