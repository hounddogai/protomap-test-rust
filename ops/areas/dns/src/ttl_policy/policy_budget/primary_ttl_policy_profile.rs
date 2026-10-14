use super::*;

use acme_kit::text;

/// Where a primary ttl policy profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTtlPolicyProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryTtlPolicyProfileStatus {
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

impl std::fmt::Display for PrimaryTtlPolicyProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary ttl policy profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfile {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub segment: String,
    pub country: String,
    pub weight: f64,
    pub issuer: String,
    pub status: PrimaryTtlPolicyProfileStatus,
}

impl PrimaryTtlPolicyProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTtlPolicyProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTtlPolicyProfileStatus) -> bool {
        match self.status {
            PrimaryTtlPolicyProfileStatus::Planned => {
                match next {
                    PrimaryTtlPolicyProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyProfileStatus::Scheduled => {
                return next == PrimaryTtlPolicyProfileStatus::InProgress
                    || next == PrimaryTtlPolicyProfileStatus::Cancelled;
            }
            PrimaryTtlPolicyProfileStatus::InProgress => {
                return next == PrimaryTtlPolicyProfileStatus::Completed
                    || next == PrimaryTtlPolicyProfileStatus::Cancelled;
            }
            PrimaryTtlPolicyProfileStatus::Completed => {
                match next {
                    PrimaryTtlPolicyProfileStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryTtlPolicyProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary ttl policy profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> PrimaryTtlPolicyProfile {
        PrimaryTtlPolicyProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            category: self.category.clone(),
            segment: self.segment.clone(),
            country: self.country.clone(),
            weight: self.weight,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary ttl policy profile in status planned.
pub fn new_primary_ttl_policy_profile(code: &str, name: &str) -> PrimaryTtlPolicyProfile {
    PrimaryTtlPolicyProfile {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        category: "".to_string(),
        segment: "".to_string(),
        country: "".to_string(),
        weight: 0.0,
        issuer: "".to_string(),
        status: PrimaryTtlPolicyProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_ttl_policy_profile("code-622", "name-379");
        assert_eq!(item.status, PrimaryTtlPolicyProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_primary_ttl_policy_profile("code-622", "name-379");
        if item.move_to(PrimaryTtlPolicyProfileStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTtlPolicyProfileStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_ttl_policy_profile("code-622", "name-379");
        item.status = PrimaryTtlPolicyProfileStatus::Cancelled;
        assert!(
            item.move_to(PrimaryTtlPolicyProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
