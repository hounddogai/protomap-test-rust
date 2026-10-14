use super::*;

use acme_kit::text;

/// Where a dashboard group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DashboardGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DashboardGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard group as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroup {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub principal: String,
    pub burst: i64,
    pub locale: String,
    pub active: bool,
    pub status: DashboardGroupStatus,
}

impl DashboardGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardGroupStatus) -> bool {
        match self.status {
            DashboardGroupStatus::Idle => {
                match next {
                    DashboardGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardGroupStatus::Warming => {
                match next {
                    DashboardGroupStatus::Draining | DashboardGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardGroupStatus::Ready => {
                match next {
                    DashboardGroupStatus::Draining | DashboardGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardGroupStatus::Draining => {
                return next == DashboardGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> DashboardGroup {
        DashboardGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            principal: self.principal.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a dashboard group in status idle.
pub fn new_dashboard_group(code: &str, title: &str) -> DashboardGroup {
    DashboardGroup {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        principal: "".to_string(),
        burst: 0,
        locale: "".to_string(),
        active: false,
        status: DashboardGroupStatus::Idle,
    }
}
