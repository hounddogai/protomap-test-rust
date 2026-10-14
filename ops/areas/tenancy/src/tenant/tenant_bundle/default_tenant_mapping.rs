use super::*;

use acme_kit::text;

/// Where a default tenant mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTenantMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultTenantMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultTenantMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default tenant mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMapping {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub ttl_seconds: i64,
    pub approved: bool,
    pub cluster: String,
    pub description: String,
    pub margin: f64,
    pub retries: i64,
    pub status: DefaultTenantMappingStatus,
}

impl DefaultTenantMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTenantMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTenantMappingStatus) -> bool {
        match self.status {
            DefaultTenantMappingStatus::Created => {
                match next {
                    DefaultTenantMappingStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTenantMappingStatus::Validated => {
                match next {
                    DefaultTenantMappingStatus::Published | DefaultTenantMappingStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTenantMappingStatus::Published => {
                return next == DefaultTenantMappingStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultTenantMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default tenant mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        (self.margin * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTenantMapping {
        DefaultTenantMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            ttl_seconds: self.ttl_seconds,
            approved: self.approved,
            cluster: self.cluster.clone(),
            description: self.description.clone(),
            margin: self.margin,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default tenant mapping in status created.
pub fn new_default_tenant_mapping(code: &str, title: &str) -> DefaultTenantMapping {
    DefaultTenantMapping {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        ttl_seconds: 0,
        approved: false,
        cluster: "".to_string(),
        description: "".to_string(),
        margin: 0.0,
        retries: 0,
        status: DefaultTenantMappingStatus::Created,
    }
}
