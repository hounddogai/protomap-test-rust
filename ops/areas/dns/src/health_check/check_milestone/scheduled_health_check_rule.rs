use super::*;

use acme_kit::text;

/// Where a scheduled health check rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledHealthCheckRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledHealthCheckRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledHealthCheckRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled health check rule as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckRule {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub segment: String,
    pub region: String,
    pub key_id: String,
    pub host: String,
    pub ratio: f64,
    pub version: i64,
    pub description: String,
    pub status: ScheduledHealthCheckRuleStatus,
}

impl ScheduledHealthCheckRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledHealthCheckRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledHealthCheckRuleStatus) -> bool {
        match self.status {
            ScheduledHealthCheckRuleStatus::Requested => {
                return next == ScheduledHealthCheckRuleStatus::Granted
                    || next == ScheduledHealthCheckRuleStatus::Revoked;
            }
            ScheduledHealthCheckRuleStatus::Granted => {
                return next == ScheduledHealthCheckRuleStatus::Revoked
                    || next == ScheduledHealthCheckRuleStatus::Expired;
            }
            ScheduledHealthCheckRuleStatus::Revoked => {
                match next {
                    ScheduledHealthCheckRuleStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledHealthCheckRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled health check rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled health check rule", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        (self.ratio * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledHealthCheckRule {
        ScheduledHealthCheckRule {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            segment: self.segment.clone(),
            region: self.region.clone(),
            key_id: self.key_id.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            version: self.version,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled health check rule in status requested.
pub fn new_scheduled_health_check_rule(code: &str, title: &str) -> ScheduledHealthCheckRule {
    ScheduledHealthCheckRule {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        segment: "".to_string(),
        region: "".to_string(),
        key_id: "".to_string(),
        host: "".to_string(),
        ratio: 0.0,
        version: 0,
        description: "".to_string(),
        status: ScheduledHealthCheckRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_health_check_rule("code-985", "title-321");
        assert_eq!(item.status, ScheduledHealthCheckRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_health_check_rule("code-985", "title-321");
        if item.move_to(ScheduledHealthCheckRuleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledHealthCheckRuleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_health_check_rule("code-985", "title-321");
        item.status = ScheduledHealthCheckRuleStatus::Expired;
        assert!(
            item.move_to(ScheduledHealthCheckRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
