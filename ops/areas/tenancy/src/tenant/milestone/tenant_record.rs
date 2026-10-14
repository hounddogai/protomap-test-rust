use super::*;

use acme_kit::text;

/// Where a tenant record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecord {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub approved: bool,
    pub active: bool,
    pub namespace: String,
    pub duration_minutes: i64,
    pub budget: f64,
    pub status: TenantRecordStatus,
}

impl TenantRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantRecordStatus) -> bool {
        match self.status {
            TenantRecordStatus::Created => {
                match next {
                    TenantRecordStatus::Validated | TenantRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantRecordStatus::Validated => {
                return next == TenantRecordStatus::Retired;
            }
            TenantRecordStatus::Published => {
                match next {
                    TenantRecordStatus::Retired => {
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

    pub fn move_to(&mut self, next: TenantRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.budget + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantRecord {
        TenantRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            approved: self.approved,
            active: self.active,
            namespace: self.namespace.clone(),
            duration_minutes: self.duration_minutes,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tenant record in status created.
pub fn new_tenant_record(code: &str, name: &str) -> TenantRecord {
    TenantRecord {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        approved: false,
        active: false,
        namespace: "".to_string(),
        duration_minutes: 0,
        budget: 0.0,
        status: TenantRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tenant_record("code-790", "name-475");
        assert_eq!(item.status, TenantRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tenant_record("code-790", "name-475");
        if item.move_to(TenantRecordStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantRecordStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tenant_record("code-790", "name-475");
        item.status = TenantRecordStatus::Retired;
        assert!(
            item.move_to(TenantRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
