use super::*;

use acme_kit::text;

/// Where a dashboard target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DashboardTargetStatus {
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

impl std::fmt::Display for DashboardTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dashboard target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTarget {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub audience: String,
    pub ceiling: f64,
    pub maximum: f64,
    pub status: DashboardTargetStatus,
}

impl DashboardTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardTargetStatus) -> bool {
        match self.status {
            DashboardTargetStatus::Idle => {
                match next {
                    DashboardTargetStatus::Warming | DashboardTargetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardTargetStatus::Warming => {
                match next {
                    DashboardTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardTargetStatus::Ready => {
                return next == DashboardTargetStatus::Draining;
            }
            DashboardTargetStatus::Draining => {
                match next {
                    DashboardTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DashboardTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard target", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.maximum * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardTarget {
        DashboardTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            ceiling: self.ceiling,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dashboard target in status idle.
pub fn new_dashboard_target(code: &str, title: &str) -> DashboardTarget {
    DashboardTarget {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        audience: "".to_string(),
        ceiling: 0.0,
        maximum: 0.0,
        status: DashboardTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dashboard_target("code-503", "title-230");
        assert_eq!(item.status, DashboardTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dashboard_target("code-503", "title-230");
        if item.move_to(DashboardTargetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardTargetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dashboard_target("code-503", "title-230");
        item.status = DashboardTargetStatus::Stopped;
        assert!(
            item.move_to(DashboardTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
