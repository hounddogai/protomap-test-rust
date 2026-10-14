use super::*;

use acme_kit::text;

/// Where a manual probe milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualProbeMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualProbeMilestoneStatus {
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

impl std::fmt::Display for ManualProbeMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual probe milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestone {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub updated_at: i64,
    pub rate: f64,
    pub phone: String,
    pub status: ManualProbeMilestoneStatus,
}

impl ManualProbeMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualProbeMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualProbeMilestoneStatus) -> bool {
        match self.status {
            ManualProbeMilestoneStatus::Draft => {
                match next {
                    ManualProbeMilestoneStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualProbeMilestoneStatus::Pending => {
                return next == ManualProbeMilestoneStatus::Suspended;
            }
            ManualProbeMilestoneStatus::Active => {
                return next == ManualProbeMilestoneStatus::Closed;
            }
            ManualProbeMilestoneStatus::Suspended => {
                match next {
                    ManualProbeMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualProbeMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual probe milestone {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.maximum - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualProbeMilestone {
        ManualProbeMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            updated_at: self.updated_at,
            rate: self.rate,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual probe milestone in status draft.
pub fn new_manual_probe_milestone(code: &str, title: &str) -> ManualProbeMilestone {
    ManualProbeMilestone {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        updated_at: 0,
        rate: 0.0,
        phone: "".to_string(),
        status: ManualProbeMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_probe_milestone("code-220", "title-422");
        assert_eq!(item.status, ManualProbeMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_manual_probe_milestone("code-220", "title-422");
        if item.move_to(ManualProbeMilestoneStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualProbeMilestoneStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_probe_milestone("code-220", "title-422");
        item.status = ManualProbeMilestoneStatus::Closed;
        assert!(
            item.move_to(ManualProbeMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
