use super::*;

use acme_kit::text;

/// Where a dashboard adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardAdjustmentStatus {
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

impl std::fmt::Display for DashboardAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dashboard adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustment {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub revision: i64,
    pub contact_email: String,
    pub checksum: String,
    pub name: String,
    pub ends_at: i64,
    pub quantity: i64,
    pub amount: f64,
    pub description: String,
    pub status: DashboardAdjustmentStatus,
}

impl DashboardAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardAdjustmentStatus) -> bool {
        match self.status {
            DashboardAdjustmentStatus::New => {
                return next == DashboardAdjustmentStatus::Failed;
            }
            DashboardAdjustmentStatus::Queued => {
                return next == DashboardAdjustmentStatus::Running || next == DashboardAdjustmentStatus::Done;
            }
            DashboardAdjustmentStatus::Running => {
                return next == DashboardAdjustmentStatus::Done || next == DashboardAdjustmentStatus::Failed;
            }
            DashboardAdjustmentStatus::Done => {
                return next == DashboardAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard adjustment", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardAdjustment {
        DashboardAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            revision: self.revision,
            contact_email: self.contact_email.clone(),
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            quantity: self.quantity,
            amount: self.amount,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dashboard adjustment in status new.
pub fn new_dashboard_adjustment(code: &str, title: &str) -> DashboardAdjustment {
    DashboardAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        revision: 0,
        contact_email: "".to_string(),
        checksum: "".to_string(),
        name: "".to_string(),
        ends_at: 0,
        quantity: 0,
        amount: 0.0,
        description: "".to_string(),
        status: DashboardAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dashboard_adjustment("code-758", "title-585");
        assert_eq!(item.status, DashboardAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_dashboard_adjustment("code-758", "title-585");
        if item.move_to(DashboardAdjustmentStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardAdjustmentStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dashboard_adjustment("code-758", "title-585");
        item.status = DashboardAdjustmentStatus::Failed;
        assert!(
            item.move_to(DashboardAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
