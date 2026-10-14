use super::*;

use acme_kit::text;

/// Where a binding override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingOverrideStatus {
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

impl std::fmt::Display for BindingOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding override as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverride {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub phone: String,
    pub ends_at: i64,
    pub reviewer: String,
    pub audience: String,
    pub pinned: bool,
    pub status: BindingOverrideStatus,
}

impl BindingOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingOverrideStatus) -> bool {
        match self.status {
            BindingOverrideStatus::Idle => {
                match next {
                    BindingOverrideStatus::Draining | BindingOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingOverrideStatus::Warming => {
                return next == BindingOverrideStatus::Ready || next == BindingOverrideStatus::Stopped;
            }
            BindingOverrideStatus::Ready => {
                return next == BindingOverrideStatus::Draining || next == BindingOverrideStatus::Stopped;
            }
            BindingOverrideStatus::Draining => {
                return next == BindingOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> BindingOverride {
        BindingOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            phone: self.phone.clone(),
            ends_at: self.ends_at,
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a binding override in status idle.
pub fn new_binding_override(code: &str, label: &str) -> BindingOverride {
    BindingOverride {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        phone: "".to_string(),
        ends_at: 0,
        reviewer: "".to_string(),
        audience: "".to_string(),
        pinned: false,
        status: BindingOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_override("code-605", "label-190");
        assert_eq!(item.status, BindingOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_binding_override("code-605", "label-190");
        if item.move_to(BindingOverrideStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingOverrideStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_override("code-605", "label-190");
        item.status = BindingOverrideStatus::Stopped;
        assert!(
            item.move_to(BindingOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
