use super::*;

use acme_kit::text;

/// Where a trust domain group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TrustDomainGroupStatus {
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

impl std::fmt::Display for TrustDomainGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainGroup {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub reviewer: String,
    pub contact_email: String,
    pub tenant_id: String,
    pub quantity: i64,
    pub status: TrustDomainGroupStatus,
}

impl TrustDomainGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainGroupStatus) -> bool {
        match self.status {
            TrustDomainGroupStatus::Planned => {
                return next == TrustDomainGroupStatus::Scheduled;
            }
            TrustDomainGroupStatus::Scheduled => {
                match next {
                    TrustDomainGroupStatus::InProgress | TrustDomainGroupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainGroupStatus::InProgress => {
                return next == TrustDomainGroupStatus::Completed || next == TrustDomainGroupStatus::Cancelled;
            }
            TrustDomainGroupStatus::Completed => {
                match next {
                    TrustDomainGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TrustDomainGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain group {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quantity as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainGroup {
        TrustDomainGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            reviewer: self.reviewer.clone(),
            contact_email: self.contact_email.clone(),
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a trust domain group in status planned.
pub fn new_trust_domain_group(code: &str, name: &str) -> TrustDomainGroup {
    TrustDomainGroup {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        reviewer: "".to_string(),
        contact_email: "".to_string(),
        tenant_id: "".to_string(),
        quantity: 0,
        status: TrustDomainGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_trust_domain_group("code-678", "name-758");
        assert_eq!(item.status, TrustDomainGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_trust_domain_group("code-678", "name-758");
        if item.move_to(TrustDomainGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_trust_domain_group("code-678", "name-758");
        item.status = TrustDomainGroupStatus::Cancelled;
        assert!(
            item.move_to(TrustDomainGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
