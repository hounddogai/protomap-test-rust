use super::*;

use acme_kit::text;

/// Where a canary cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CanaryCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CanaryCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary cycle as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCycle {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub phone: String,
    pub notes: String,
    pub tenant_id: String,
    pub port: i64,
    pub fingerprint: String,
    pub status: CanaryCycleStatus,
}

impl CanaryCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryCycleStatus) -> bool {
        match self.status {
            CanaryCycleStatus::Requested => {
                match next {
                    CanaryCycleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryCycleStatus::Granted => {
                match next {
                    CanaryCycleStatus::Revoked | CanaryCycleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryCycleStatus::Revoked => {
                return next == CanaryCycleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.port as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryCycle {
        CanaryCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary cycle in status requested.
pub fn new_canary_cycle(code: &str, name: &str) -> CanaryCycle {
    CanaryCycle {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        phone: "".to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        port: 0,
        fingerprint: "".to_string(),
        status: CanaryCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_canary_cycle("code-791", "name-329");
        assert_eq!(item.status, CanaryCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_canary_cycle("code-791", "name-329");
        if item.move_to(CanaryCycleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryCycleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_canary_cycle("code-791", "name-329");
        item.status = CanaryCycleStatus::Expired;
        assert!(
            item.move_to(CanaryCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
