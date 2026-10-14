use super::*;

use acme_kit::text;

/// Where a default trust domain profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTrustDomainProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultTrustDomainProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultTrustDomainProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default trust domain profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfile {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub channel: String,
    pub created_at: i64,
    pub host: String,
    pub source: String,
    pub burst: i64,
    pub budget: f64,
    pub scope: String,
    pub status: DefaultTrustDomainProfileStatus,
}

impl DefaultTrustDomainProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTrustDomainProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTrustDomainProfileStatus) -> bool {
        match self.status {
            DefaultTrustDomainProfileStatus::Planned => {
                return next == DefaultTrustDomainProfileStatus::InProgress
                    || next == DefaultTrustDomainProfileStatus::Cancelled;
            }
            DefaultTrustDomainProfileStatus::Scheduled => {
                match next {
                    DefaultTrustDomainProfileStatus::Completed | DefaultTrustDomainProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTrustDomainProfileStatus::InProgress => {
                match next {
                    DefaultTrustDomainProfileStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTrustDomainProfileStatus::Completed => {
                match next {
                    DefaultTrustDomainProfileStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultTrustDomainProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default trust domain profile {} cannot move from {} to {}",
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
        format!("{} {}", "default trust domain profile", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.starts_at as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTrustDomainProfile {
        DefaultTrustDomainProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            created_at: self.created_at,
            host: self.host.clone(),
            source: self.source.clone(),
            burst: self.burst,
            budget: self.budget,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default trust domain profile in status planned.
pub fn new_default_trust_domain_profile(code: &str, title: &str) -> DefaultTrustDomainProfile {
    DefaultTrustDomainProfile {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        channel: "".to_string(),
        created_at: 0,
        host: "".to_string(),
        source: "".to_string(),
        burst: 0,
        budget: 0.0,
        scope: "".to_string(),
        status: DefaultTrustDomainProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_trust_domain_profile("code-492", "title-15");
        assert_eq!(item.status, DefaultTrustDomainProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_trust_domain_profile("code-492", "title-15");
        if item.move_to(DefaultTrustDomainProfileStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTrustDomainProfileStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_trust_domain_profile("code-492", "title-15");
        item.status = DefaultTrustDomainProfileStatus::Cancelled;
        assert!(
            item.move_to(DefaultTrustDomainProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
