use super::*;

use acme_kit::text;

/// Where a capped rollout milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRolloutMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedRolloutMilestoneStatus {
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

impl std::fmt::Display for CappedRolloutMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped rollout milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestone {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub updated_at: i64,
    pub country: String,
    pub amount: f64,
    pub duration_minutes: i64,
    pub cluster: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub ratio: f64,
    pub status: CappedRolloutMilestoneStatus,
}

impl CappedRolloutMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRolloutMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRolloutMilestoneStatus) -> bool {
        match self.status {
            CappedRolloutMilestoneStatus::Draft => {
                return next == CappedRolloutMilestoneStatus::Active || next == CappedRolloutMilestoneStatus::Suspended;
            }
            CappedRolloutMilestoneStatus::Pending => {
                match next {
                    CappedRolloutMilestoneStatus::Active | CappedRolloutMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRolloutMilestoneStatus::Active => {
                match next {
                    CappedRolloutMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRolloutMilestoneStatus::Suspended => {
                match next {
                    CappedRolloutMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedRolloutMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped rollout milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ratio + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRolloutMilestone {
        CappedRolloutMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            updated_at: self.updated_at,
            country: self.country.clone(),
            amount: self.amount,
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a capped rollout milestone in status draft.
pub fn new_capped_rollout_milestone(code: &str, label: &str) -> CappedRolloutMilestone {
    CappedRolloutMilestone {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        updated_at: 0,
        country: "".to_string(),
        amount: 0.0,
        duration_minutes: 0,
        cluster: "".to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        ratio: 0.0,
        status: CappedRolloutMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_rollout_milestone("code-825", "label-318");
        assert_eq!(item.status, CappedRolloutMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_capped_rollout_milestone("code-825", "label-318");
        if item.move_to(CappedRolloutMilestoneStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRolloutMilestoneStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_rollout_milestone("code-825", "label-318");
        item.status = CappedRolloutMilestoneStatus::Closed;
        assert!(
            item.move_to(CappedRolloutMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
