use super::*;

use acme_kit::text;

/// Where a tenant mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMapping {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub external_ref: String,
    pub ends_at: i64,
    pub segment: String,
    pub expires_at: i64,
    pub tenant_id: String,
    pub status: TenantMappingStatus,
}

impl TenantMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantMappingStatus) -> bool {
        match self.status {
            TenantMappingStatus::Created => {
                return next == TenantMappingStatus::Validated || next == TenantMappingStatus::Retired;
            }
            TenantMappingStatus::Validated => {
                return next == TenantMappingStatus::Published || next == TenantMappingStatus::Retired;
            }
            TenantMappingStatus::Published => {
                return next == TenantMappingStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.expires_at as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantMapping {
        TenantMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            external_ref: self.external_ref.clone(),
            ends_at: self.ends_at,
            segment: self.segment.clone(),
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tenant mapping in status created.
pub fn new_tenant_mapping(code: &str, label: &str) -> TenantMapping {
    TenantMapping {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        external_ref: "".to_string(),
        ends_at: 0,
        segment: "".to_string(),
        expires_at: 0,
        tenant_id: "".to_string(),
        status: TenantMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tenant_mapping("code-773", "label-440");
        assert_eq!(item.status, TenantMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tenant_mapping("code-773", "label-440");
        if item.move_to(TenantMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tenant_mapping("code-773", "label-440");
        item.status = TenantMappingStatus::Retired;
        assert!(
            item.move_to(TenantMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
