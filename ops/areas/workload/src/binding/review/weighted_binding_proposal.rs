use super::*;

use acme_kit::text;

/// Where a weighted binding proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedBindingProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedBindingProposalStatus {
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

impl std::fmt::Display for WeightedBindingProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted binding proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposal {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub pinned: bool,
    pub visible: bool,
    pub audience: String,
    pub expires_at: i64,
    pub quota_units: i64,
    pub category: String,
    pub attempts: i64,
    pub status: WeightedBindingProposalStatus,
}

impl WeightedBindingProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedBindingProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedBindingProposalStatus) -> bool {
        match self.status {
            WeightedBindingProposalStatus::Planned => {
                return next == WeightedBindingProposalStatus::InProgress
                    || next == WeightedBindingProposalStatus::Cancelled;
            }
            WeightedBindingProposalStatus::Scheduled => {
                match next {
                    WeightedBindingProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBindingProposalStatus::InProgress => {
                match next {
                    WeightedBindingProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBindingProposalStatus::Completed => {
                match next {
                    WeightedBindingProposalStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedBindingProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted binding proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedBindingProposal {
        WeightedBindingProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            pinned: self.pinned,
            visible: self.visible,
            audience: self.audience.clone(),
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            category: self.category.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a weighted binding proposal in status planned.
pub fn new_weighted_binding_proposal(code: &str, title: &str) -> WeightedBindingProposal {
    WeightedBindingProposal {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        ttl_seconds: 0,
        pinned: false,
        visible: false,
        audience: "".to_string(),
        expires_at: 0,
        quota_units: 0,
        category: "".to_string(),
        attempts: 0,
        status: WeightedBindingProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_binding_proposal("code-628", "title-295");
        assert_eq!(item.status, WeightedBindingProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_weighted_binding_proposal("code-628", "title-295");
        if item.move_to(WeightedBindingProposalStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedBindingProposalStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_binding_proposal("code-628", "title-295");
        item.status = WeightedBindingProposalStatus::Cancelled;
        assert!(
            item.move_to(WeightedBindingProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
