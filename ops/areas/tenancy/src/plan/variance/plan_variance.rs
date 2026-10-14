use super::*;

use acme_kit::text;

/// Where a plan variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PlanVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PlanVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanVariance {
    pub code: String,
    pub title: String,
    pub host: String,
    pub channel: String,
    pub attempts: i64,
    pub grace_days: i64,
    pub audience: String,
    pub ceiling: f64,
    pub burst: i64,
    pub locale: String,
    pub status: PlanVarianceStatus,
}

impl PlanVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanVarianceStatus) -> bool {
        match self.status {
            PlanVarianceStatus::Requested => {
                match next {
                    PlanVarianceStatus::Revoked | PlanVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanVarianceStatus::Granted => {
                return next == PlanVarianceStatus::Revoked;
            }
            PlanVarianceStatus::Revoked => {
                match next {
                    PlanVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: PlanVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.attempts as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanVariance {
        PlanVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            channel: self.channel.clone(),
            attempts: self.attempts,
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            ceiling: self.ceiling,
            burst: self.burst,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a plan variance in status requested.
pub fn new_plan_variance(code: &str, title: &str) -> PlanVariance {
    PlanVariance {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        channel: "".to_string(),
        attempts: 0,
        grace_days: 0,
        audience: "".to_string(),
        ceiling: 0.0,
        burst: 0,
        locale: "".to_string(),
        status: PlanVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_plan_variance("code-833", "title-173");
        assert_eq!(item.status, PlanVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_plan_variance("code-833", "title-173");
        if item.move_to(PlanVarianceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanVarianceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_plan_variance("code-833", "title-173");
        item.status = PlanVarianceStatus::Expired;
        assert!(
            item.move_to(PlanVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
