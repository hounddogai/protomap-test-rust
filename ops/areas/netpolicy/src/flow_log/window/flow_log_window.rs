use super::*;

use acme_kit::text;

/// Where a flow log window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FlowLogWindowStatus {
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

impl std::fmt::Display for FlowLogWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log window as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindow {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub phone: String,
    pub grace_days: i64,
    pub capacity: i64,
    pub host: String,
    pub starts_at: i64,
    pub audience: String,
    pub priority: i64,
    pub status: FlowLogWindowStatus,
}

impl FlowLogWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogWindowStatus) -> bool {
        match self.status {
            FlowLogWindowStatus::Idle => {
                match next {
                    FlowLogWindowStatus::Warming | FlowLogWindowStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogWindowStatus::Warming => {
                return next == FlowLogWindowStatus::Draining;
            }
            FlowLogWindowStatus::Ready => {
                return next == FlowLogWindowStatus::Draining;
            }
            FlowLogWindowStatus::Draining => {
                match next {
                    FlowLogWindowStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FlowLogWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log window {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.starts_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogWindow {
        FlowLogWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            capacity: self.capacity,
            host: self.host.clone(),
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a flow log window in status idle.
pub fn new_flow_log_window(code: &str, name: &str) -> FlowLogWindow {
    FlowLogWindow {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        phone: "".to_string(),
        grace_days: 0,
        capacity: 0,
        host: "".to_string(),
        starts_at: 0,
        audience: "".to_string(),
        priority: 0,
        status: FlowLogWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_flow_log_window("code-335", "name-538");
        assert_eq!(item.status, FlowLogWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_flow_log_window("code-335", "name-538");
        if item.move_to(FlowLogWindowStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogWindowStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_flow_log_window("code-335", "name-538");
        item.status = FlowLogWindowStatus::Stopped;
        assert!(
            item.move_to(FlowLogWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
