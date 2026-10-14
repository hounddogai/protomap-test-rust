use super::*;

use acme_kit::text;

/// Where a fallback quote schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackQuoteScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackQuoteScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackQuoteScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback quote schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteSchedule {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub starts_at: i64,
    pub active: bool,
    pub rotation_days: i64,
    pub budget: f64,
    pub principal: String,
    pub margin: f64,
    pub status: FallbackQuoteScheduleStatus,
}

impl FallbackQuoteSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackQuoteScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackQuoteScheduleStatus) -> bool {
        match self.status {
            FallbackQuoteScheduleStatus::Requested => {
                return next == FallbackQuoteScheduleStatus::Revoked || next == FallbackQuoteScheduleStatus::Expired;
            }
            FallbackQuoteScheduleStatus::Granted => {
                return next == FallbackQuoteScheduleStatus::Expired;
            }
            FallbackQuoteScheduleStatus::Revoked => {
                return next == FallbackQuoteScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackQuoteScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback quote schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rotation_days as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackQuoteSchedule {
        FallbackQuoteSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            starts_at: self.starts_at,
            active: self.active,
            rotation_days: self.rotation_days,
            budget: self.budget,
            principal: self.principal.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback quote schedule in status requested.
pub fn new_fallback_quote_schedule(code: &str, name: &str) -> FallbackQuoteSchedule {
    FallbackQuoteSchedule {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        starts_at: 0,
        active: false,
        rotation_days: 0,
        budget: 0.0,
        principal: "".to_string(),
        margin: 0.0,
        status: FallbackQuoteScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_quote_schedule("code-833", "name-563");
        assert_eq!(item.status, FallbackQuoteScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_fallback_quote_schedule("code-833", "name-563");
        if item.move_to(FallbackQuoteScheduleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackQuoteScheduleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_quote_schedule("code-833", "name-563");
        item.status = FallbackQuoteScheduleStatus::Expired;
        assert!(
            item.move_to(FallbackQuoteScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
