use super::*;

use acme_kit::text;

/// Where a binding claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingClaimStatus {
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

impl std::fmt::Display for BindingClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingClaim {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub version: i64,
    pub window_days: i64,
    pub segment: String,
    pub sequence: i64,
    pub tenant_id: String,
    pub minimum: f64,
    pub margin: f64,
    pub contact_email: String,
    pub status: BindingClaimStatus,
}

impl BindingClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingClaimStatus) -> bool {
        match self.status {
            BindingClaimStatus::Idle => {
                match next {
                    BindingClaimStatus::Draining | BindingClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingClaimStatus::Warming => {
                return next == BindingClaimStatus::Ready || next == BindingClaimStatus::Draining;
            }
            BindingClaimStatus::Ready => {
                match next {
                    BindingClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingClaimStatus::Draining => {
                match next {
                    BindingClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BindingClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.version as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingClaim {
        BindingClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            version: self.version,
            window_days: self.window_days,
            segment: self.segment.clone(),
            sequence: self.sequence,
            tenant_id: self.tenant_id.clone(),
            minimum: self.minimum,
            margin: self.margin,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a binding claim in status idle.
pub fn new_binding_claim(code: &str, name: &str) -> BindingClaim {
    BindingClaim {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        version: 0,
        window_days: 0,
        segment: "".to_string(),
        sequence: 0,
        tenant_id: "".to_string(),
        minimum: 0.0,
        margin: 0.0,
        contact_email: "".to_string(),
        status: BindingClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_claim("code-272", "name-363");
        assert_eq!(item.status, BindingClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_binding_claim("code-272", "name-363");
        if item.move_to(BindingClaimStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingClaimStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_claim("code-272", "name-363");
        item.status = BindingClaimStatus::Stopped;
        assert!(item.move_to(BindingClaimStatus::Idle).is_err(), "expected move to fail");
    }
}
