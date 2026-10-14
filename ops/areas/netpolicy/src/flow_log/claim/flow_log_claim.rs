use super::*;

use acme_kit::text;

/// Where a flow log claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaim {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub burst: i64,
    pub contact_email: String,
    pub country: String,
    pub namespace: String,
    pub approved: bool,
    pub budget: f64,
    pub status: FlowLogClaimStatus,
}

impl FlowLogClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogClaimStatus) -> bool {
        match self.status {
            FlowLogClaimStatus::Proposed => {
                return next == FlowLogClaimStatus::Rejected || next == FlowLogClaimStatus::Withdrawn;
            }
            FlowLogClaimStatus::Approved => {
                return next == FlowLogClaimStatus::Rejected;
            }
            FlowLogClaimStatus::Rejected => {
                match next {
                    FlowLogClaimStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FlowLogClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log claim {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.burst as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogClaim {
        FlowLogClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            burst: self.burst,
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            namespace: self.namespace.clone(),
            approved: self.approved,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a flow log claim in status proposed.
pub fn new_flow_log_claim(code: &str, label: &str) -> FlowLogClaim {
    FlowLogClaim {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        burst: 0,
        contact_email: "".to_string(),
        country: "".to_string(),
        namespace: "".to_string(),
        approved: false,
        budget: 0.0,
        status: FlowLogClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_claim("code-26", "label-582");
        assert_eq!(item.status, FlowLogClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_flow_log_claim("code-26", "label-582");
        if item.move_to(FlowLogClaimStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogClaimStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_claim("code-26", "label-582");
        item.status = FlowLogClaimStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
