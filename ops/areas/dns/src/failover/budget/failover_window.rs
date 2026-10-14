use super::*;

use acme_kit::text;

/// Where a failover window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FailoverWindowStatus {
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

impl std::fmt::Display for FailoverWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverWindow {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub unit: String,
    pub external_ref: String,
    pub namespace: String,
    pub grace_days: i64,
    pub status: FailoverWindowStatus,
}

impl FailoverWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverWindowStatus) -> bool {
        match self.status {
            FailoverWindowStatus::Draft => {
                match next {
                    FailoverWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverWindowStatus::Pending => {
                match next {
                    FailoverWindowStatus::Active | FailoverWindowStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverWindowStatus::Active => {
                match next {
                    FailoverWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverWindowStatus::Suspended => {
                match next {
                    FailoverWindowStatus::Closed => {
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

    pub fn move_to(&mut self, next: FailoverWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "failover window", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverWindow {
        FailoverWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            unit: self.unit.clone(),
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a failover window in status draft.
pub fn new_failover_window(code: &str, name: &str) -> FailoverWindow {
    FailoverWindow {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        unit: "".to_string(),
        external_ref: "".to_string(),
        namespace: "".to_string(),
        grace_days: 0,
        status: FailoverWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_failover_window("code-628", "name-997");
        assert_eq!(item.status, FailoverWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_failover_window("code-628", "name-997");
        if item.move_to(FailoverWindowStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverWindowStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_failover_window("code-628", "name-997");
        item.status = FailoverWindowStatus::Closed;
        assert!(
            item.move_to(FailoverWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
