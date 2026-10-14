use super::*;

use acme_kit::text;

/// Where a strict tenant adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTenantAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictTenantAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictTenantAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict tenant adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAdjustment {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub port: i64,
    pub created_at: i64,
    pub unit: String,
    pub phone: String,
    pub cipher: String,
    pub status: StrictTenantAdjustmentStatus,
}

impl StrictTenantAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTenantAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTenantAdjustmentStatus) -> bool {
        match self.status {
            StrictTenantAdjustmentStatus::Created => {
                match next {
                    StrictTenantAdjustmentStatus::Validated | StrictTenantAdjustmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTenantAdjustmentStatus::Validated => {
                return next == StrictTenantAdjustmentStatus::Published
                    || next == StrictTenantAdjustmentStatus::Retired;
            }
            StrictTenantAdjustmentStatus::Published => {
                return next == StrictTenantAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictTenantAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict tenant adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.sequence as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTenantAdjustment {
        StrictTenantAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            port: self.port,
            created_at: self.created_at,
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict tenant adjustment in status created.
pub fn new_strict_tenant_adjustment(code: &str, name: &str) -> StrictTenantAdjustment {
    StrictTenantAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        port: 0,
        created_at: 0,
        unit: "".to_string(),
        phone: "".to_string(),
        cipher: "".to_string(),
        status: StrictTenantAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_tenant_adjustment("code-265", "name-300");
        assert_eq!(item.status, StrictTenantAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_tenant_adjustment("code-265", "name-300");
        if item.move_to(StrictTenantAdjustmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTenantAdjustmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_tenant_adjustment("code-265", "name-300");
        item.status = StrictTenantAdjustmentStatus::Retired;
        assert!(
            item.move_to(StrictTenantAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
