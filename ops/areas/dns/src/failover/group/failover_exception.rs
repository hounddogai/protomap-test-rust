use super::*;

use acme_kit::text;

/// Where a failover exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FailoverExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FailoverExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverException {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub margin: f64,
    pub audience: String,
    pub namespace: String,
    pub external_ref: String,
    pub cost: f64,
    pub retries: i64,
    pub expires_at: i64,
    pub status: FailoverExceptionStatus,
}

impl FailoverException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverExceptionStatus) -> bool {
        match self.status {
            FailoverExceptionStatus::Requested => {
                return next == FailoverExceptionStatus::Granted || next == FailoverExceptionStatus::Revoked;
            }
            FailoverExceptionStatus::Granted => {
                match next {
                    FailoverExceptionStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverExceptionStatus::Revoked => {
                return next == FailoverExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover exception {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.cost + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverException {
        FailoverException {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            margin: self.margin,
            audience: self.audience.clone(),
            namespace: self.namespace.clone(),
            external_ref: self.external_ref.clone(),
            cost: self.cost,
            retries: self.retries,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a failover exception in status requested.
pub fn new_failover_exception(code: &str, title: &str) -> FailoverException {
    FailoverException {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        margin: 0.0,
        audience: "".to_string(),
        namespace: "".to_string(),
        external_ref: "".to_string(),
        cost: 0.0,
        retries: 0,
        expires_at: 0,
        status: FailoverExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_failover_exception("code-146", "title-277");
        assert_eq!(item.status, FailoverExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_failover_exception("code-146", "title-277");
        if item.move_to(FailoverExceptionStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverExceptionStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_failover_exception("code-146", "title-277");
        item.status = FailoverExceptionStatus::Expired;
        assert!(
            item.move_to(FailoverExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
