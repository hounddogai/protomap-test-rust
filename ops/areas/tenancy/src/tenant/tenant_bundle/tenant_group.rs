use super::*;

use acme_kit::text;

/// Where a tenant group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TenantGroupStatus {
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

impl std::fmt::Display for TenantGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantGroup {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub grace_days: i64,
    pub visible: bool,
    pub status: TenantGroupStatus,
}

impl TenantGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantGroupStatus) -> bool {
        match self.status {
            TenantGroupStatus::Idle => {
                return next == TenantGroupStatus::Ready || next == TenantGroupStatus::Draining;
            }
            TenantGroupStatus::Warming => {
                match next {
                    TenantGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantGroupStatus::Ready => {
                match next {
                    TenantGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantGroupStatus::Draining => {
                match next {
                    TenantGroupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TenantGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tenant group", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> TenantGroup {
        TenantGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            grace_days: self.grace_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tenant group in status idle.
pub fn new_tenant_group(code: &str, label: &str) -> TenantGroup {
    TenantGroup {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        grace_days: 0,
        visible: false,
        status: TenantGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tenant_group("code-540", "label-412");
        assert_eq!(item.status, TenantGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tenant_group("code-540", "label-412");
        if item.move_to(TenantGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tenant_group("code-540", "label-412");
        item.status = TenantGroupStatus::Stopped;
        assert!(item.move_to(TenantGroupStatus::Idle).is_err(), "expected move to fail");
    }
}
