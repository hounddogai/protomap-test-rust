use super::*;

use acme_kit::text;

/// Where a health check exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl HealthCheckExceptionStatus {
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

impl std::fmt::Display for HealthCheckExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckException {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub segment: String,
    pub capacity: i64,
    pub label: String,
    pub cipher: String,
    pub attempts: i64,
    pub channel: String,
    pub currency: String,
    pub revision: i64,
    pub status: HealthCheckExceptionStatus,
}

impl HealthCheckException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckExceptionStatus) -> bool {
        match self.status {
            HealthCheckExceptionStatus::Draft => {
                return next == HealthCheckExceptionStatus::Pending || next == HealthCheckExceptionStatus::Active;
            }
            HealthCheckExceptionStatus::Pending => {
                match next {
                    HealthCheckExceptionStatus::Active | HealthCheckExceptionStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckExceptionStatus::Active => {
                return next == HealthCheckExceptionStatus::Closed;
            }
            HealthCheckExceptionStatus::Suspended => {
                return next == HealthCheckExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckException {
        HealthCheckException {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            capacity: self.capacity,
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a health check exception in status draft.
pub fn new_health_check_exception(code: &str, title: &str) -> HealthCheckException {
    HealthCheckException {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        segment: "".to_string(),
        capacity: 0,
        label: "".to_string(),
        cipher: "".to_string(),
        attempts: 0,
        channel: "".to_string(),
        currency: "".to_string(),
        revision: 0,
        status: HealthCheckExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_health_check_exception("code-738", "title-901");
        assert_eq!(item.status, HealthCheckExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_health_check_exception("code-738", "title-901");
        if item.move_to(HealthCheckExceptionStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckExceptionStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_health_check_exception("code-738", "title-901");
        item.status = HealthCheckExceptionStatus::Closed;
        assert!(
            item.move_to(HealthCheckExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
