use super::*;

use acme_kit::text;

/// Where a approval metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ApprovalMetricStatus {
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

impl std::fmt::Display for ApprovalMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMetric {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub revision: i64,
    pub reviewer: String,
    pub currency: String,
    pub threshold: f64,
    pub tenant_id: String,
    pub active: bool,
    pub ceiling: f64,
    pub status: ApprovalMetricStatus,
}

impl ApprovalMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalMetricStatus) -> bool {
        match self.status {
            ApprovalMetricStatus::Draft => {
                match next {
                    ApprovalMetricStatus::Suspended | ApprovalMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalMetricStatus::Pending => {
                match next {
                    ApprovalMetricStatus::Active | ApprovalMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalMetricStatus::Active => {
                match next {
                    ApprovalMetricStatus::Suspended | ApprovalMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalMetricStatus::Suspended => {
                match next {
                    ApprovalMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: ApprovalMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval metric", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.threshold * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalMetric {
        ApprovalMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            revision: self.revision,
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            threshold: self.threshold,
            tenant_id: self.tenant_id.clone(),
            active: self.active,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a approval metric in status draft.
pub fn new_approval_metric(code: &str, name: &str) -> ApprovalMetric {
    ApprovalMetric {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        revision: 0,
        reviewer: "".to_string(),
        currency: "".to_string(),
        threshold: 0.0,
        tenant_id: "".to_string(),
        active: false,
        ceiling: 0.0,
        status: ApprovalMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_approval_metric("code-572", "name-331");
        assert_eq!(item.status, ApprovalMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_approval_metric("code-572", "name-331");
        if item.move_to(ApprovalMetricStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalMetricStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_approval_metric("code-572", "name-331");
        item.status = ApprovalMetricStatus::Closed;
        assert!(
            item.move_to(ApprovalMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
