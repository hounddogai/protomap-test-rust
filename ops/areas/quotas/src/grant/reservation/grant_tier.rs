use super::*;

use acme_kit::text;

/// Where a grant tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GrantTierStatus {
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

impl std::fmt::Display for GrantTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantTier {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub phone: String,
    pub window_days: i64,
    pub status: GrantTierStatus,
}

impl GrantTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantTierStatus) -> bool {
        match self.status {
            GrantTierStatus::Draft => {
                match next {
                    GrantTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantTierStatus::Pending => {
                match next {
                    GrantTierStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantTierStatus::Active => {
                return next == GrantTierStatus::Suspended || next == GrantTierStatus::Closed;
            }
            GrantTierStatus::Suspended => {
                match next {
                    GrantTierStatus::Closed => {
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

    pub fn move_to(&mut self, next: GrantTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantTier {
        GrantTier {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            phone: self.phone.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a grant tier in status draft.
pub fn new_grant_tier(code: &str, title: &str) -> GrantTier {
    GrantTier {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        phone: "".to_string(),
        window_days: 0,
        status: GrantTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_grant_tier("code-996", "title-661");
        assert_eq!(item.status, GrantTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_grant_tier("code-996", "title-661");
        if item.move_to(GrantTierStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantTierStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_grant_tier("code-996", "title-661");
        item.status = GrantTierStatus::Closed;
        assert!(item.move_to(GrantTierStatus::Draft).is_err(), "expected move to fail");
    }
}
