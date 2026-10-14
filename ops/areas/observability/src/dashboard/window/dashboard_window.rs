use super::*;

use acme_kit::text;

/// Where a dashboard window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DashboardWindowStatus {
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

impl std::fmt::Display for DashboardWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardWindow {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub locale: String,
    pub margin: f64,
    pub expires_at: i64,
    pub key_id: String,
    pub status: DashboardWindowStatus,
}

impl DashboardWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardWindowStatus) -> bool {
        match self.status {
            DashboardWindowStatus::Idle => {
                return next == DashboardWindowStatus::Draining || next == DashboardWindowStatus::Stopped;
            }
            DashboardWindowStatus::Warming => {
                match next {
                    DashboardWindowStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardWindowStatus::Ready => {
                match next {
                    DashboardWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardWindowStatus::Draining => {
                return next == DashboardWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.margin - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardWindow {
        DashboardWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            margin: self.margin,
            expires_at: self.expires_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a dashboard window in status idle.
pub fn new_dashboard_window(code: &str, name: &str) -> DashboardWindow {
    DashboardWindow {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        locale: "".to_string(),
        margin: 0.0,
        expires_at: 0,
        key_id: "".to_string(),
        status: DashboardWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dashboard_window("code-660", "name-660");
        assert_eq!(item.status, DashboardWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_dashboard_window("code-660", "name-660");
        if item.move_to(DashboardWindowStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardWindowStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dashboard_window("code-660", "name-660");
        item.status = DashboardWindowStatus::Stopped;
        assert!(
            item.move_to(DashboardWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
