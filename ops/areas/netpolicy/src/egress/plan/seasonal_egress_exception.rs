use super::*;

use acme_kit::text;

/// Where a seasonal egress exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEgressExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalEgressExceptionStatus {
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

impl std::fmt::Display for SeasonalEgressExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal egress exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressException {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub limit: i64,
    pub port: i64,
    pub expires_at: i64,
    pub source: String,
    pub maximum: f64,
    pub retries: i64,
    pub status: SeasonalEgressExceptionStatus,
}

impl SeasonalEgressException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEgressExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEgressExceptionStatus) -> bool {
        match self.status {
            SeasonalEgressExceptionStatus::Draft => {
                return next == SeasonalEgressExceptionStatus::Active
                    || next == SeasonalEgressExceptionStatus::Suspended;
            }
            SeasonalEgressExceptionStatus::Pending => {
                match next {
                    SeasonalEgressExceptionStatus::Active | SeasonalEgressExceptionStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEgressExceptionStatus::Active => {
                return next == SeasonalEgressExceptionStatus::Suspended
                    || next == SeasonalEgressExceptionStatus::Closed;
            }
            SeasonalEgressExceptionStatus::Suspended => {
                return next == SeasonalEgressExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalEgressExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal egress exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.expires_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEgressException {
        SeasonalEgressException {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            limit: self.limit,
            port: self.port,
            expires_at: self.expires_at,
            source: self.source.clone(),
            maximum: self.maximum,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal egress exception in status draft.
pub fn new_seasonal_egress_exception(code: &str, name: &str) -> SeasonalEgressException {
    SeasonalEgressException {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        limit: 0,
        port: 0,
        expires_at: 0,
        source: "".to_string(),
        maximum: 0.0,
        retries: 0,
        status: SeasonalEgressExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_egress_exception("code-71", "name-996");
        assert_eq!(item.status, SeasonalEgressExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_seasonal_egress_exception("code-71", "name-996");
        if item.move_to(SeasonalEgressExceptionStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEgressExceptionStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_egress_exception("code-71", "name-996");
        item.status = SeasonalEgressExceptionStatus::Closed;
        assert!(
            item.move_to(SeasonalEgressExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
