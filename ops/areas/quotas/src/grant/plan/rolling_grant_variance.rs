use super::*;

use acme_kit::text;

/// Where a rolling grant variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingGrantVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingGrantVarianceStatus {
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

impl std::fmt::Display for RollingGrantVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling grant variance as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVariance {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub port: i64,
    pub grace_days: i64,
    pub status: RollingGrantVarianceStatus,
}

impl RollingGrantVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingGrantVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingGrantVarianceStatus) -> bool {
        match self.status {
            RollingGrantVarianceStatus::Draft => {
                return next == RollingGrantVarianceStatus::Suspended;
            }
            RollingGrantVarianceStatus::Pending => {
                match next {
                    RollingGrantVarianceStatus::Suspended | RollingGrantVarianceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingGrantVarianceStatus::Active => {
                return next == RollingGrantVarianceStatus::Suspended;
            }
            RollingGrantVarianceStatus::Suspended => {
                match next {
                    RollingGrantVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingGrantVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling grant variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingGrantVariance {
        RollingGrantVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            port: self.port,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling grant variance in status draft.
pub fn new_rolling_grant_variance(code: &str, title: &str) -> RollingGrantVariance {
    RollingGrantVariance {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        port: 0,
        grace_days: 0,
        status: RollingGrantVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_grant_variance("code-678", "title-542");
        assert_eq!(item.status, RollingGrantVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_grant_variance("code-678", "title-542");
        if item.move_to(RollingGrantVarianceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingGrantVarianceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_grant_variance("code-678", "title-542");
        item.status = RollingGrantVarianceStatus::Closed;
        assert!(
            item.move_to(RollingGrantVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
