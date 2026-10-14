use super::*;

use acme_kit::text;

/// Where a canary record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CanaryRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CanaryRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary record as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRecord {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub issuer: String,
    pub tenant_id: String,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub margin: f64,
    pub maximum: f64,
    pub starts_at: i64,
    pub status: CanaryRecordStatus,
}

impl CanaryRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryRecordStatus) -> bool {
        match self.status {
            CanaryRecordStatus::Created => {
                match next {
                    CanaryRecordStatus::Validated | CanaryRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryRecordStatus::Validated => {
                match next {
                    CanaryRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryRecordStatus::Published => {
                match next {
                    CanaryRecordStatus::Retired => {
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

    pub fn move_to(&mut self, next: CanaryRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary record {} cannot move from {} to {}",
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

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryRecord {
        CanaryRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            margin: self.margin,
            maximum: self.maximum,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary record in status created.
pub fn new_canary_record(code: &str, name: &str) -> CanaryRecord {
    CanaryRecord {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        issuer: "".to_string(),
        tenant_id: "".to_string(),
        archived: false,
        ttl_seconds: 0,
        retries: 0,
        margin: 0.0,
        maximum: 0.0,
        starts_at: 0,
        status: CanaryRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_canary_record("code-447", "name-48");
        assert_eq!(item.status, CanaryRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_canary_record("code-447", "name-48");
        if item.move_to(CanaryRecordStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryRecordStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_canary_record("code-447", "name-48");
        item.status = CanaryRecordStatus::Retired;
        assert!(
            item.move_to(CanaryRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
