use super::*;

use acme_kit::text;

/// Where a default grant quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultGrantQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultGrantQuotaStatus {
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

impl std::fmt::Display for DefaultGrantQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default grant quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuota {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub threshold: f64,
    pub principal: String,
    pub country: String,
    pub status: DefaultGrantQuotaStatus,
}

impl DefaultGrantQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultGrantQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultGrantQuotaStatus) -> bool {
        match self.status {
            DefaultGrantQuotaStatus::Idle => {
                match next {
                    DefaultGrantQuotaStatus::Warming | DefaultGrantQuotaStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultGrantQuotaStatus::Warming => {
                return next == DefaultGrantQuotaStatus::Ready || next == DefaultGrantQuotaStatus::Stopped;
            }
            DefaultGrantQuotaStatus::Ready => {
                return next == DefaultGrantQuotaStatus::Draining;
            }
            DefaultGrantQuotaStatus::Draining => {
                return next == DefaultGrantQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultGrantQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default grant quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.threshold - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultGrantQuota {
        DefaultGrantQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            threshold: self.threshold,
            principal: self.principal.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default grant quota in status idle.
pub fn new_default_grant_quota(code: &str, name: &str) -> DefaultGrantQuota {
    DefaultGrantQuota {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        threshold: 0.0,
        principal: "".to_string(),
        country: "".to_string(),
        status: DefaultGrantQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_grant_quota("code-608", "name-620");
        assert_eq!(item.status, DefaultGrantQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_grant_quota("code-608", "name-620");
        if item.move_to(DefaultGrantQuotaStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultGrantQuotaStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_grant_quota("code-608", "name-620");
        item.status = DefaultGrantQuotaStatus::Stopped;
        assert!(
            item.move_to(DefaultGrantQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
