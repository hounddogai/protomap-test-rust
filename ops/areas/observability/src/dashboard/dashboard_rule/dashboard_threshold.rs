use super::*;

use acme_kit::text;

/// Where a dashboard threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DashboardThresholdStatus {
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

impl std::fmt::Display for DashboardThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardThreshold {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub cluster: String,
    pub threshold: f64,
    pub key_id: String,
    pub description: String,
    pub contact_email: String,
    pub limit: i64,
    pub principal: String,
    pub status: DashboardThresholdStatus,
}

impl DashboardThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardThresholdStatus) -> bool {
        match self.status {
            DashboardThresholdStatus::Planned => {
                return next == DashboardThresholdStatus::Completed;
            }
            DashboardThresholdStatus::Scheduled => {
                match next {
                    DashboardThresholdStatus::Completed | DashboardThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardThresholdStatus::InProgress => {
                match next {
                    DashboardThresholdStatus::Completed | DashboardThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardThresholdStatus::Completed => {
                return next == DashboardThresholdStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.threshold * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardThreshold {
        DashboardThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            cluster: self.cluster.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            description: self.description.clone(),
            contact_email: self.contact_email.clone(),
            limit: self.limit,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a dashboard threshold in status planned.
pub fn new_dashboard_threshold(code: &str, name: &str) -> DashboardThreshold {
    DashboardThreshold {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        cluster: "".to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        description: "".to_string(),
        contact_email: "".to_string(),
        limit: 0,
        principal: "".to_string(),
        status: DashboardThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dashboard_threshold("code-26", "name-153");
        assert_eq!(item.status, DashboardThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_dashboard_threshold("code-26", "name-153");
        if item.move_to(DashboardThresholdStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardThresholdStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dashboard_threshold("code-26", "name-153");
        item.status = DashboardThresholdStatus::Cancelled;
        assert!(
            item.move_to(DashboardThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
