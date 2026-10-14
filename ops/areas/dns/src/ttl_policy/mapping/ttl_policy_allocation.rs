use super::*;

use acme_kit::text;

/// Where a ttl policy allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TtlPolicyAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TtlPolicyAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocation {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub threshold: f64,
    pub region: String,
    pub name: String,
    pub principal: String,
    pub status: TtlPolicyAllocationStatus,
}

impl TtlPolicyAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyAllocationStatus) -> bool {
        match self.status {
            TtlPolicyAllocationStatus::New => {
                match next {
                    TtlPolicyAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyAllocationStatus::Queued => {
                match next {
                    TtlPolicyAllocationStatus::Running | TtlPolicyAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyAllocationStatus::Running => {
                match next {
                    TtlPolicyAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyAllocationStatus::Done => {
                match next {
                    TtlPolicyAllocationStatus::Failed => {
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

    pub fn move_to(&mut self, next: TtlPolicyAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> TtlPolicyAllocation {
        TtlPolicyAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            threshold: self.threshold,
            region: self.region.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a ttl policy allocation in status new.
pub fn new_ttl_policy_allocation(code: &str, label: &str) -> TtlPolicyAllocation {
    TtlPolicyAllocation {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        threshold: 0.0,
        region: "".to_string(),
        name: "".to_string(),
        principal: "".to_string(),
        status: TtlPolicyAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_ttl_policy_allocation("code-355", "label-565");
        assert_eq!(item.status, TtlPolicyAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_ttl_policy_allocation("code-355", "label-565");
        if item.move_to(TtlPolicyAllocationStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyAllocationStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_ttl_policy_allocation("code-355", "label-565");
        item.status = TtlPolicyAllocationStatus::Failed;
        assert!(
            item.move_to(TtlPolicyAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
