use super::*;

use acme_kit::text;

/// Where a adaptive ttl policy agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTtlPolicyAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveTtlPolicyAgreementStatus {
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

impl std::fmt::Display for AdaptiveTtlPolicyAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive ttl policy agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyAgreement {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub cost: f64,
    pub burst: i64,
    pub batch_size: i64,
    pub version: i64,
    pub reviewer: String,
    pub segment: String,
    pub owner_id: String,
    pub status: AdaptiveTtlPolicyAgreementStatus,
}

impl AdaptiveTtlPolicyAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTtlPolicyAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTtlPolicyAgreementStatus) -> bool {
        match self.status {
            AdaptiveTtlPolicyAgreementStatus::Planned => {
                return next == AdaptiveTtlPolicyAgreementStatus::InProgress
                    || next == AdaptiveTtlPolicyAgreementStatus::Cancelled;
            }
            AdaptiveTtlPolicyAgreementStatus::Scheduled => {
                match next {
                    AdaptiveTtlPolicyAgreementStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTtlPolicyAgreementStatus::InProgress => {
                match next {
                    AdaptiveTtlPolicyAgreementStatus::Completed | AdaptiveTtlPolicyAgreementStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTtlPolicyAgreementStatus::Completed => {
                return next == AdaptiveTtlPolicyAgreementStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveTtlPolicyAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive ttl policy agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.cost - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTtlPolicyAgreement {
        AdaptiveTtlPolicyAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            cost: self.cost,
            burst: self.burst,
            batch_size: self.batch_size,
            version: self.version,
            reviewer: self.reviewer.clone(),
            segment: self.segment.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive ttl policy agreement in status planned.
pub fn new_adaptive_ttl_policy_agreement(code: &str, name: &str) -> AdaptiveTtlPolicyAgreement {
    AdaptiveTtlPolicyAgreement {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        cost: 0.0,
        burst: 0,
        batch_size: 0,
        version: 0,
        reviewer: "".to_string(),
        segment: "".to_string(),
        owner_id: "".to_string(),
        status: AdaptiveTtlPolicyAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_ttl_policy_agreement("code-843", "name-765");
        assert_eq!(item.status, AdaptiveTtlPolicyAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_adaptive_ttl_policy_agreement("code-843", "name-765");
        if item.move_to(AdaptiveTtlPolicyAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTtlPolicyAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_ttl_policy_agreement("code-843", "name-765");
        item.status = AdaptiveTtlPolicyAgreementStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveTtlPolicyAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
