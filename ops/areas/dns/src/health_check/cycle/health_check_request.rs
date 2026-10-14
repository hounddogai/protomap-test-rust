use super::*;

use acme_kit::text;

/// Where a health check request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl HealthCheckRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for HealthCheckRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequest {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub key_id: String,
    pub currency: String,
    pub margin: f64,
    pub starts_at: i64,
    pub ceiling: f64,
    pub fingerprint: String,
    pub audience: String,
    pub floor_value: f64,
    pub status: HealthCheckRequestStatus,
}

impl HealthCheckRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckRequestStatus) -> bool {
        match self.status {
            HealthCheckRequestStatus::Open => {
                match next {
                    HealthCheckRequestStatus::InReview | HealthCheckRequestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckRequestStatus::InReview => {
                match next {
                    HealthCheckRequestStatus::Resolved | HealthCheckRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckRequestStatus::Resolved => {
                match next {
                    HealthCheckRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckRequestStatus::Reopened => {
                match next {
                    HealthCheckRequestStatus::Archived => {
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

    pub fn move_to(&mut self, next: HealthCheckRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check request {} cannot move from {} to {}",
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
        format!("{} {}", "health check request", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.margin - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckRequest {
        HealthCheckRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            key_id: self.key_id.clone(),
            currency: self.currency.clone(),
            margin: self.margin,
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check request in status open.
pub fn new_health_check_request(code: &str, title: &str) -> HealthCheckRequest {
    HealthCheckRequest {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        key_id: "".to_string(),
        currency: "".to_string(),
        margin: 0.0,
        starts_at: 0,
        ceiling: 0.0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        floor_value: 0.0,
        status: HealthCheckRequestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_health_check_request("code-704", "title-529");
        assert_eq!(item.status, HealthCheckRequestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_health_check_request("code-704", "title-529");
        if item.move_to(HealthCheckRequestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckRequestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_health_check_request("code-704", "title-529");
        item.status = HealthCheckRequestStatus::Archived;
        assert!(
            item.move_to(HealthCheckRequestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
