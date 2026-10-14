use super::*;

use acme_kit::text;

/// Where a rolling allowlist exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAllowlistExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingAllowlistExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingAllowlistExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling allowlist exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistException {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub cluster: String,
    pub key_id: String,
    pub region: String,
    pub attempts: i64,
    pub budget: f64,
    pub archived: bool,
    pub duration_minutes: i64,
    pub status: RollingAllowlistExceptionStatus,
}

impl RollingAllowlistException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAllowlistExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAllowlistExceptionStatus) -> bool {
        match self.status {
            RollingAllowlistExceptionStatus::Proposed => {
                match next {
                    RollingAllowlistExceptionStatus::Rejected | RollingAllowlistExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAllowlistExceptionStatus::Approved => {
                match next {
                    RollingAllowlistExceptionStatus::Rejected | RollingAllowlistExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAllowlistExceptionStatus::Rejected => {
                return next == RollingAllowlistExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingAllowlistExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling allowlist exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling allowlist exception", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAllowlistException {
        RollingAllowlistException {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            attempts: self.attempts,
            budget: self.budget,
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling allowlist exception in status proposed.
pub fn new_rolling_allowlist_exception(code: &str, name: &str) -> RollingAllowlistException {
    RollingAllowlistException {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        cluster: "".to_string(),
        key_id: "".to_string(),
        region: "".to_string(),
        attempts: 0,
        budget: 0.0,
        archived: false,
        duration_minutes: 0,
        status: RollingAllowlistExceptionStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_allowlist_exception("code-14", "name-237");
        assert_eq!(item.status, RollingAllowlistExceptionStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rolling_allowlist_exception("code-14", "name-237");
        if item.move_to(RollingAllowlistExceptionStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAllowlistExceptionStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_allowlist_exception("code-14", "name-237");
        item.status = RollingAllowlistExceptionStatus::Withdrawn;
        assert!(
            item.move_to(RollingAllowlistExceptionStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
