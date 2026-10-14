use super::*;

use acme_kit::text;

/// Where a adaptive health check quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveHealthCheckQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveHealthCheckQuotaStatus {
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

impl std::fmt::Display for AdaptiveHealthCheckQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive health check quota as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckQuota {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub scope: String,
    pub external_ref: String,
    pub segment: String,
    pub description: String,
    pub status: AdaptiveHealthCheckQuotaStatus,
}

impl AdaptiveHealthCheckQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveHealthCheckQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveHealthCheckQuotaStatus) -> bool {
        match self.status {
            AdaptiveHealthCheckQuotaStatus::Open => {
                match next {
                    AdaptiveHealthCheckQuotaStatus::InReview | AdaptiveHealthCheckQuotaStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveHealthCheckQuotaStatus::InReview => {
                return next == AdaptiveHealthCheckQuotaStatus::Reopened
                    || next == AdaptiveHealthCheckQuotaStatus::Archived;
            }
            AdaptiveHealthCheckQuotaStatus::Resolved => {
                match next {
                    AdaptiveHealthCheckQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveHealthCheckQuotaStatus::Reopened => {
                return next == AdaptiveHealthCheckQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveHealthCheckQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive health check quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> AdaptiveHealthCheckQuota {
        AdaptiveHealthCheckQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            segment: self.segment.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive health check quota in status open.
pub fn new_adaptive_health_check_quota(code: &str, title: &str) -> AdaptiveHealthCheckQuota {
    AdaptiveHealthCheckQuota {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        scope: "".to_string(),
        external_ref: "".to_string(),
        segment: "".to_string(),
        description: "".to_string(),
        status: AdaptiveHealthCheckQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_health_check_quota("code-610", "title-156");
        assert_eq!(item.status, AdaptiveHealthCheckQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_health_check_quota("code-610", "title-156");
        if item.move_to(AdaptiveHealthCheckQuotaStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveHealthCheckQuotaStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_health_check_quota("code-610", "title-156");
        item.status = AdaptiveHealthCheckQuotaStatus::Archived;
        assert!(
            item.move_to(AdaptiveHealthCheckQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
