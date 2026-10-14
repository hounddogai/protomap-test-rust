use super::*;

use acme_kit::text;

/// Where a global dashboard window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDashboardWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalDashboardWindowStatus {
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

impl std::fmt::Display for GlobalDashboardWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global dashboard window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardWindow {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub channel: String,
    pub capacity: i64,
    pub attempts: i64,
    pub revision: i64,
    pub status: GlobalDashboardWindowStatus,
}

impl GlobalDashboardWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDashboardWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDashboardWindowStatus) -> bool {
        match self.status {
            GlobalDashboardWindowStatus::Draft => {
                return next == GlobalDashboardWindowStatus::Suspended || next == GlobalDashboardWindowStatus::Closed;
            }
            GlobalDashboardWindowStatus::Pending => {
                match next {
                    GlobalDashboardWindowStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDashboardWindowStatus::Active => {
                return next == GlobalDashboardWindowStatus::Suspended || next == GlobalDashboardWindowStatus::Closed;
            }
            GlobalDashboardWindowStatus::Suspended => {
                match next {
                    GlobalDashboardWindowStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalDashboardWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global dashboard window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global dashboard window", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDashboardWindow {
        GlobalDashboardWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            channel: self.channel.clone(),
            capacity: self.capacity,
            attempts: self.attempts,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a global dashboard window in status draft.
pub fn new_global_dashboard_window(code: &str, title: &str) -> GlobalDashboardWindow {
    GlobalDashboardWindow {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        channel: "".to_string(),
        capacity: 0,
        attempts: 0,
        revision: 0,
        status: GlobalDashboardWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_dashboard_window("code-420", "title-304");
        assert_eq!(item.status, GlobalDashboardWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_global_dashboard_window("code-420", "title-304");
        if item.move_to(GlobalDashboardWindowStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalDashboardWindowStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_dashboard_window("code-420", "title-304");
        item.status = GlobalDashboardWindowStatus::Closed;
        assert!(
            item.move_to(GlobalDashboardWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
