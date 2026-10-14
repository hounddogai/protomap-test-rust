use super::*;

use acme_kit::text;

/// Where a egress milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressMilestoneStatus {
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

impl std::fmt::Display for EgressMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress milestone as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestone {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub country: String,
    pub namespace: String,
    pub ratio: f64,
    pub sequence: i64,
    pub status: EgressMilestoneStatus,
}

impl EgressMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressMilestoneStatus) -> bool {
        match self.status {
            EgressMilestoneStatus::Draft => {
                match next {
                    EgressMilestoneStatus::Pending | EgressMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressMilestoneStatus::Pending => {
                return next == EgressMilestoneStatus::Suspended;
            }
            EgressMilestoneStatus::Active => {
                match next {
                    EgressMilestoneStatus::Suspended | EgressMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressMilestoneStatus::Suspended => {
                match next {
                    EgressMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.sequence as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressMilestone {
        EgressMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a egress milestone in status draft.
pub fn new_egress_milestone(code: &str, title: &str) -> EgressMilestone {
    EgressMilestone {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        country: "".to_string(),
        namespace: "".to_string(),
        ratio: 0.0,
        sequence: 0,
        status: EgressMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_milestone("code-541", "title-219");
        assert_eq!(item.status, EgressMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_milestone("code-541", "title-219");
        if item.move_to(EgressMilestoneStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressMilestoneStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_milestone("code-541", "title-219");
        item.status = EgressMilestoneStatus::Closed;
        assert!(
            item.move_to(EgressMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
