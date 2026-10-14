use super::*;

use acme_kit::text;

/// Where a dns record rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DnsRecordRuleStatus {
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

impl std::fmt::Display for DnsRecordRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRule {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub rotation_days: i64,
    pub floor_value: f64,
    pub created_at: i64,
    pub threshold: f64,
    pub status: DnsRecordRuleStatus,
}

impl DnsRecordRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordRuleStatus) -> bool {
        match self.status {
            DnsRecordRuleStatus::Planned => {
                return next == DnsRecordRuleStatus::Scheduled || next == DnsRecordRuleStatus::InProgress;
            }
            DnsRecordRuleStatus::Scheduled => {
                match next {
                    DnsRecordRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRuleStatus::InProgress => {
                match next {
                    DnsRecordRuleStatus::Completed | DnsRecordRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRuleStatus::Completed => {
                return next == DnsRecordRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record rule {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.limit as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordRule {
        DnsRecordRule {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            rotation_days: self.rotation_days,
            floor_value: self.floor_value,
            created_at: self.created_at,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record rule in status planned.
pub fn new_dns_record_rule(code: &str, name: &str) -> DnsRecordRule {
    DnsRecordRule {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        limit: 0,
        rotation_days: 0,
        floor_value: 0.0,
        created_at: 0,
        threshold: 0.0,
        status: DnsRecordRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dns_record_rule("code-110", "name-863");
        assert_eq!(item.status, DnsRecordRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_dns_record_rule("code-110", "name-863");
        if item.move_to(DnsRecordRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dns_record_rule("code-110", "name-863");
        item.status = DnsRecordRuleStatus::Cancelled;
        assert!(
            item.move_to(DnsRecordRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
