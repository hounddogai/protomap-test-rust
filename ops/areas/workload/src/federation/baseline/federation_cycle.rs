use super::*;

use acme_kit::text;

/// Where a federation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FederationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FederationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationCycle {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub active: bool,
    pub sequence: i64,
    pub status: FederationCycleStatus,
}

impl FederationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationCycleStatus) -> bool {
        match self.status {
            FederationCycleStatus::Draft => {
                match next {
                    FederationCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationCycleStatus::Pending => {
                return next == FederationCycleStatus::Closed;
            }
            FederationCycleStatus::Active => {
                return next == FederationCycleStatus::Closed;
            }
            FederationCycleStatus::Suspended => {
                match next {
                    FederationCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: FederationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation cycle {} cannot move from {} to {}",
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
        format!("{} {}", "federation cycle", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationCycle {
        FederationCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            active: self.active,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation cycle in status draft.
pub fn new_federation_cycle(code: &str, name: &str) -> FederationCycle {
    FederationCycle {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        active: false,
        sequence: 0,
        status: FederationCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_federation_cycle("code-545", "name-216");
        assert_eq!(item.status, FederationCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_federation_cycle("code-545", "name-216");
        if item.move_to(FederationCycleStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationCycleStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_federation_cycle("code-545", "name-216");
        item.status = FederationCycleStatus::Closed;
        assert!(
            item.move_to(FederationCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
