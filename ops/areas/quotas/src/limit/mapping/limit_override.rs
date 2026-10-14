use super::*;

use acme_kit::text;

/// Where a limit override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitOverrideStatus {
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

impl std::fmt::Display for LimitOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitOverride {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub retries: i64,
    pub title: String,
    pub description: String,
    pub label: String,
    pub status: LimitOverrideStatus,
}

impl LimitOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitOverrideStatus) -> bool {
        match self.status {
            LimitOverrideStatus::Idle => {
                match next {
                    LimitOverrideStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitOverrideStatus::Warming => {
                match next {
                    LimitOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitOverrideStatus::Ready => {
                match next {
                    LimitOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitOverrideStatus::Draining => {
                match next {
                    LimitOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LimitOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "limit override", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitOverride {
        LimitOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            retries: self.retries,
            title: self.title.clone(),
            description: self.description.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a limit override in status idle.
pub fn new_limit_override(code: &str, name: &str) -> LimitOverride {
    LimitOverride {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        retries: 0,
        title: "".to_string(),
        description: "".to_string(),
        label: "".to_string(),
        status: LimitOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_limit_override("code-768", "name-679");
        assert_eq!(item.status, LimitOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_limit_override("code-768", "name-679");
        if item.move_to(LimitOverrideStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitOverrideStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_limit_override("code-768", "name-679");
        item.status = LimitOverrideStatus::Stopped;
        assert!(
            item.move_to(LimitOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
