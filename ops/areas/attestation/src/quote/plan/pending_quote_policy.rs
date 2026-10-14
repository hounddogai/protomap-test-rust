use super::*;

use acme_kit::text;

/// Where a pending quote policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQuotePolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingQuotePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingQuotePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending quote policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuotePolicy {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub channel: String,
    pub country: String,
    pub weight: f64,
    pub capacity: i64,
    pub tenant_id: String,
    pub status: PendingQuotePolicyStatus,
}

impl PendingQuotePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQuotePolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQuotePolicyStatus) -> bool {
        match self.status {
            PendingQuotePolicyStatus::Proposed => {
                return next == PendingQuotePolicyStatus::Rejected;
            }
            PendingQuotePolicyStatus::Approved => {
                return next == PendingQuotePolicyStatus::Rejected || next == PendingQuotePolicyStatus::Withdrawn;
            }
            PendingQuotePolicyStatus::Rejected => {
                match next {
                    PendingQuotePolicyStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingQuotePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending quote policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending quote policy", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.capacity as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingQuotePolicy {
        PendingQuotePolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            channel: self.channel.clone(),
            country: self.country.clone(),
            weight: self.weight,
            capacity: self.capacity,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending quote policy in status proposed.
pub fn new_pending_quote_policy(code: &str, title: &str) -> PendingQuotePolicy {
    PendingQuotePolicy {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        channel: "".to_string(),
        country: "".to_string(),
        weight: 0.0,
        capacity: 0,
        tenant_id: "".to_string(),
        status: PendingQuotePolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_quote_policy("code-892", "title-492");
        assert_eq!(item.status, PendingQuotePolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_pending_quote_policy("code-892", "title-492");
        if item.move_to(PendingQuotePolicyStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingQuotePolicyStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_quote_policy("code-892", "title-492");
        item.status = PendingQuotePolicyStatus::Withdrawn;
        assert!(
            item.move_to(PendingQuotePolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
