use super::*;

use acme_kit::text;

/// Where a lenient query tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQueryTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientQueryTierStatus {
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

impl std::fmt::Display for LenientQueryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient query tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryTier {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub timezone: String,
    pub created_at: i64,
    pub attempts: i64,
    pub audience: String,
    pub amount: f64,
    pub status: LenientQueryTierStatus,
}

impl LenientQueryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQueryTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQueryTierStatus) -> bool {
        match self.status {
            LenientQueryTierStatus::Draft => {
                return next == LenientQueryTierStatus::Pending || next == LenientQueryTierStatus::Suspended;
            }
            LenientQueryTierStatus::Pending => {
                match next {
                    LenientQueryTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQueryTierStatus::Active => {
                return next == LenientQueryTierStatus::Closed;
            }
            LenientQueryTierStatus::Suspended => {
                match next {
                    LenientQueryTierStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientQueryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient query tier {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rate + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientQueryTier {
        LenientQueryTier {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            timezone: self.timezone.clone(),
            created_at: self.created_at,
            attempts: self.attempts,
            audience: self.audience.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient query tier in status draft.
pub fn new_lenient_query_tier(code: &str, name: &str) -> LenientQueryTier {
    LenientQueryTier {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        timezone: "".to_string(),
        created_at: 0,
        attempts: 0,
        audience: "".to_string(),
        amount: 0.0,
        status: LenientQueryTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_query_tier("code-686", "name-894");
        assert_eq!(item.status, LenientQueryTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_query_tier("code-686", "name-894");
        if item.move_to(LenientQueryTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientQueryTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_query_tier("code-686", "name-894");
        item.status = LenientQueryTierStatus::Closed;
        assert!(
            item.move_to(LenientQueryTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
