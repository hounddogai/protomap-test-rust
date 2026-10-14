use super::*;

use acme_kit::text;

/// Where a strict health check summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictHealthCheckSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictHealthCheckSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictHealthCheckSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict health check summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckSummary {
    pub code: String,
    pub name: String,
    pub country: String,
    pub fingerprint: String,
    pub floor_value: f64,
    pub description: String,
    pub timezone: String,
    pub namespace: String,
    pub visible: bool,
    pub summary: String,
    pub status: StrictHealthCheckSummaryStatus,
}

impl StrictHealthCheckSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictHealthCheckSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictHealthCheckSummaryStatus) -> bool {
        match self.status {
            StrictHealthCheckSummaryStatus::Requested => {
                match next {
                    StrictHealthCheckSummaryStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckSummaryStatus::Granted => {
                match next {
                    StrictHealthCheckSummaryStatus::Revoked | StrictHealthCheckSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckSummaryStatus::Revoked => {
                match next {
                    StrictHealthCheckSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: StrictHealthCheckSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict health check summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> StrictHealthCheckSummary {
        StrictHealthCheckSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            fingerprint: self.fingerprint.clone(),
            floor_value: self.floor_value,
            description: self.description.clone(),
            timezone: self.timezone.clone(),
            namespace: self.namespace.clone(),
            visible: self.visible,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict health check summary in status requested.
pub fn new_strict_health_check_summary(code: &str, name: &str) -> StrictHealthCheckSummary {
    StrictHealthCheckSummary {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        fingerprint: "".to_string(),
        floor_value: 0.0,
        description: "".to_string(),
        timezone: "".to_string(),
        namespace: "".to_string(),
        visible: false,
        summary: "".to_string(),
        status: StrictHealthCheckSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_health_check_summary("code-589", "name-744");
        assert_eq!(item.status, StrictHealthCheckSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_strict_health_check_summary("code-589", "name-744");
        if item.move_to(StrictHealthCheckSummaryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictHealthCheckSummaryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_health_check_summary("code-589", "name-744");
        item.status = StrictHealthCheckSummaryStatus::Expired;
        assert!(
            item.move_to(StrictHealthCheckSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
