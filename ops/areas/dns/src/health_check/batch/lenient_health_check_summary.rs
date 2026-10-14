use super::*;

use acme_kit::text;

/// Where a lenient health check summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientHealthCheckSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientHealthCheckSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientHealthCheckSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient health check summary as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummary {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub fingerprint: String,
    pub archived: bool,
    pub created_at: i64,
    pub visible: bool,
    pub expires_at: i64,
    pub weight: f64,
    pub country: String,
    pub rotation_days: i64,
    pub status: LenientHealthCheckSummaryStatus,
}

impl LenientHealthCheckSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientHealthCheckSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientHealthCheckSummaryStatus) -> bool {
        match self.status {
            LenientHealthCheckSummaryStatus::Requested => {
                return next == LenientHealthCheckSummaryStatus::Expired;
            }
            LenientHealthCheckSummaryStatus::Granted => {
                match next {
                    LenientHealthCheckSummaryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientHealthCheckSummaryStatus::Revoked => {
                match next {
                    LenientHealthCheckSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientHealthCheckSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient health check summary {} cannot move from {} to {}",
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientHealthCheckSummary {
        LenientHealthCheckSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            created_at: self.created_at,
            visible: self.visible,
            expires_at: self.expires_at,
            weight: self.weight,
            country: self.country.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient health check summary in status requested.
pub fn new_lenient_health_check_summary(code: &str, name: &str) -> LenientHealthCheckSummary {
    LenientHealthCheckSummary {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        fingerprint: "".to_string(),
        archived: false,
        created_at: 0,
        visible: false,
        expires_at: 0,
        weight: 0.0,
        country: "".to_string(),
        rotation_days: 0,
        status: LenientHealthCheckSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_health_check_summary("code-361", "name-338");
        assert_eq!(item.status, LenientHealthCheckSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_lenient_health_check_summary("code-361", "name-338");
        if item.move_to(LenientHealthCheckSummaryStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientHealthCheckSummaryStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_health_check_summary("code-361", "name-338");
        item.status = LenientHealthCheckSummaryStatus::Expired;
        assert!(
            item.move_to(LenientHealthCheckSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
