use super::*;

use acme_kit::text;

/// Where a grant rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GrantRollupStatus {
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

impl std::fmt::Display for GrantRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollup {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub grace_days: i64,
    pub quantity: i64,
    pub locale: String,
    pub status: GrantRollupStatus,
}

impl GrantRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantRollupStatus) -> bool {
        match self.status {
            GrantRollupStatus::Idle => {
                match next {
                    GrantRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantRollupStatus::Warming => {
                return next == GrantRollupStatus::Ready || next == GrantRollupStatus::Stopped;
            }
            GrantRollupStatus::Ready => {
                return next == GrantRollupStatus::Draining || next == GrantRollupStatus::Stopped;
            }
            GrantRollupStatus::Draining => {
                return next == GrantRollupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantRollup {
        GrantRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            quantity: self.quantity,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a grant rollup in status idle.
pub fn new_grant_rollup(code: &str, name: &str) -> GrantRollup {
    GrantRollup {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        grace_days: 0,
        quantity: 0,
        locale: "".to_string(),
        status: GrantRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_grant_rollup("code-686", "name-571");
        assert_eq!(item.status, GrantRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_grant_rollup("code-686", "name-571");
        if item.move_to(GrantRollupStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantRollupStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_grant_rollup("code-686", "name-571");
        item.status = GrantRollupStatus::Stopped;
        assert!(item.move_to(GrantRollupStatus::Idle).is_err(), "expected move to fail");
    }
}
