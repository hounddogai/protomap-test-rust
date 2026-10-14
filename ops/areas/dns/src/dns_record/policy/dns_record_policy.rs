use super::*;

use acme_kit::text;

/// Where a dns record policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DnsRecordPolicyStatus {
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

impl std::fmt::Display for DnsRecordPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicy {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub namespace: String,
    pub score: f64,
    pub owner_id: String,
    pub revision: i64,
    pub weight: f64,
    pub created_at: i64,
    pub reviewer: String,
    pub status: DnsRecordPolicyStatus,
}

impl DnsRecordPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordPolicyStatus) -> bool {
        match self.status {
            DnsRecordPolicyStatus::Planned => {
                match next {
                    DnsRecordPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordPolicyStatus::Scheduled => {
                match next {
                    DnsRecordPolicyStatus::InProgress | DnsRecordPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordPolicyStatus::InProgress => {
                return next == DnsRecordPolicyStatus::Completed || next == DnsRecordPolicyStatus::Cancelled;
            }
            DnsRecordPolicyStatus::Completed => {
                return next == DnsRecordPolicyStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dns record policy", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.created_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordPolicy {
        DnsRecordPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            owner_id: self.owner_id.clone(),
            revision: self.revision,
            weight: self.weight,
            created_at: self.created_at,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a dns record policy in status planned.
pub fn new_dns_record_policy(code: &str, name: &str) -> DnsRecordPolicy {
    DnsRecordPolicy {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        namespace: "".to_string(),
        score: 0.0,
        owner_id: "".to_string(),
        revision: 0,
        weight: 0.0,
        created_at: 0,
        reviewer: "".to_string(),
        status: DnsRecordPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dns_record_policy("code-803", "name-70");
        assert_eq!(item.status, DnsRecordPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_dns_record_policy("code-803", "name-70");
        if item.move_to(DnsRecordPolicyStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordPolicyStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dns_record_policy("code-803", "name-70");
        item.status = DnsRecordPolicyStatus::Cancelled;
        assert!(
            item.move_to(DnsRecordPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
