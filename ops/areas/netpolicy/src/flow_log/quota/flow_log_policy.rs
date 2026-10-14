use super::*;

use acme_kit::text;

/// Where a flow log policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogPolicyStatus {
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

impl std::fmt::Display for FlowLogPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicy {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub ratio: f64,
    pub reviewer: String,
    pub checksum: String,
    pub locale: String,
    pub status: FlowLogPolicyStatus,
}

impl FlowLogPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogPolicyStatus) -> bool {
        match self.status {
            FlowLogPolicyStatus::Draft => {
                match next {
                    FlowLogPolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogPolicyStatus::Pending => {
                match next {
                    FlowLogPolicyStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogPolicyStatus::Active => {
                return next == FlowLogPolicyStatus::Suspended || next == FlowLogPolicyStatus::Closed;
            }
            FlowLogPolicyStatus::Suspended => {
                match next {
                    FlowLogPolicyStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log policy", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> FlowLogPolicy {
        FlowLogPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            ratio: self.ratio,
            reviewer: self.reviewer.clone(),
            checksum: self.checksum.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a flow log policy in status draft.
pub fn new_flow_log_policy(code: &str, title: &str) -> FlowLogPolicy {
    FlowLogPolicy {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        ratio: 0.0,
        reviewer: "".to_string(),
        checksum: "".to_string(),
        locale: "".to_string(),
        status: FlowLogPolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_flow_log_policy("code-832", "title-967");
        assert_eq!(item.status, FlowLogPolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_flow_log_policy("code-832", "title-967");
        if item.move_to(FlowLogPolicyStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogPolicyStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_flow_log_policy("code-832", "title-967");
        item.status = FlowLogPolicyStatus::Closed;
        assert!(
            item.move_to(FlowLogPolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
