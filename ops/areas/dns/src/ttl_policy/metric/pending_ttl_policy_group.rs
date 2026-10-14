use super::*;

use acme_kit::text;

/// Where a pending ttl policy group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTtlPolicyGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingTtlPolicyGroupStatus {
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

impl std::fmt::Display for PendingTtlPolicyGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending ttl policy group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroup {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub ratio: f64,
    pub external_ref: String,
    pub phone: String,
    pub score: f64,
    pub discount: f64,
    pub burst: i64,
    pub namespace: String,
    pub status: PendingTtlPolicyGroupStatus,
}

impl PendingTtlPolicyGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTtlPolicyGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTtlPolicyGroupStatus) -> bool {
        match self.status {
            PendingTtlPolicyGroupStatus::Planned => {
                return next == PendingTtlPolicyGroupStatus::Cancelled;
            }
            PendingTtlPolicyGroupStatus::Scheduled => {
                match next {
                    PendingTtlPolicyGroupStatus::Completed | PendingTtlPolicyGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTtlPolicyGroupStatus::InProgress => {
                match next {
                    PendingTtlPolicyGroupStatus::Completed | PendingTtlPolicyGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTtlPolicyGroupStatus::Completed => {
                match next {
                    PendingTtlPolicyGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingTtlPolicyGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending ttl policy group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.discount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTtlPolicyGroup {
        PendingTtlPolicyGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            ratio: self.ratio,
            external_ref: self.external_ref.clone(),
            phone: self.phone.clone(),
            score: self.score,
            discount: self.discount,
            burst: self.burst,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending ttl policy group in status planned.
pub fn new_pending_ttl_policy_group(code: &str, title: &str) -> PendingTtlPolicyGroup {
    PendingTtlPolicyGroup {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        ratio: 0.0,
        external_ref: "".to_string(),
        phone: "".to_string(),
        score: 0.0,
        discount: 0.0,
        burst: 0,
        namespace: "".to_string(),
        status: PendingTtlPolicyGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_ttl_policy_group("code-501", "title-855");
        assert_eq!(item.status, PendingTtlPolicyGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_pending_ttl_policy_group("code-501", "title-855");
        if item.move_to(PendingTtlPolicyGroupStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTtlPolicyGroupStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_ttl_policy_group("code-501", "title-855");
        item.status = PendingTtlPolicyGroupStatus::Cancelled;
        assert!(
            item.move_to(PendingTtlPolicyGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
