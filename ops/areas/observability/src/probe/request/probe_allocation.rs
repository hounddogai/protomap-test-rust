use super::*;

use acme_kit::text;

/// Where a probe allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ProbeAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocation {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub title: String,
    pub attempts: i64,
    pub fingerprint: String,
    pub ttl_seconds: i64,
    pub scope: String,
    pub segment: String,
    pub grace_days: i64,
    pub description: String,
    pub status: ProbeAllocationStatus,
}

impl ProbeAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeAllocationStatus) -> bool {
        match self.status {
            ProbeAllocationStatus::Draft => {
                match next {
                    ProbeAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeAllocationStatus::Pending => {
                match next {
                    ProbeAllocationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeAllocationStatus::Active => {
                match next {
                    ProbeAllocationStatus::Suspended | ProbeAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeAllocationStatus::Suspended => {
                match next {
                    ProbeAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: ProbeAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe allocation", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.budget * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeAllocation {
        ProbeAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            title: self.title.clone(),
            attempts: self.attempts,
            fingerprint: self.fingerprint.clone(),
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a probe allocation in status draft.
pub fn new_probe_allocation(code: &str, label: &str) -> ProbeAllocation {
    ProbeAllocation {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        title: "".to_string(),
        attempts: 0,
        fingerprint: "".to_string(),
        ttl_seconds: 0,
        scope: "".to_string(),
        segment: "".to_string(),
        grace_days: 0,
        description: "".to_string(),
        status: ProbeAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_allocation("code-414", "label-236");
        assert_eq!(item.status, ProbeAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_probe_allocation("code-414", "label-236");
        if item.move_to(ProbeAllocationStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeAllocationStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_allocation("code-414", "label-236");
        item.status = ProbeAllocationStatus::Closed;
        assert!(
            item.move_to(ProbeAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
