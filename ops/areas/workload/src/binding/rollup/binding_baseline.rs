use super::*;

use acme_kit::text;

/// Where a binding baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingBaselineStatus {
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

impl std::fmt::Display for BindingBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBaseline {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub pinned: bool,
    pub batch_size: i64,
    pub scope: String,
    pub port: i64,
    pub status: BindingBaselineStatus,
}

impl BindingBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingBaselineStatus) -> bool {
        match self.status {
            BindingBaselineStatus::Idle => {
                match next {
                    BindingBaselineStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBaselineStatus::Warming => {
                match next {
                    BindingBaselineStatus::Ready | BindingBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBaselineStatus::Ready => {
                match next {
                    BindingBaselineStatus::Draining | BindingBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBaselineStatus::Draining => {
                return next == BindingBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingBaseline {
        BindingBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            pinned: self.pinned,
            batch_size: self.batch_size,
            scope: self.scope.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a binding baseline in status idle.
pub fn new_binding_baseline(code: &str, name: &str) -> BindingBaseline {
    BindingBaseline {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        pinned: false,
        batch_size: 0,
        scope: "".to_string(),
        port: 0,
        status: BindingBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_baseline("code-353", "name-793");
        assert_eq!(item.status, BindingBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_binding_baseline("code-353", "name-793");
        if item.move_to(BindingBaselineStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingBaselineStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_baseline("code-353", "name-793");
        item.status = BindingBaselineStatus::Stopped;
        assert!(
            item.move_to(BindingBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
