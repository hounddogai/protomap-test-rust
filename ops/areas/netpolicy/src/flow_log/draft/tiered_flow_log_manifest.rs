use super::*;

use acme_kit::text;

/// Where a tiered flow log manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFlowLogManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredFlowLogManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredFlowLogManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered flow log manifest as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifest {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub batch_size: i64,
    pub created_at: i64,
    pub external_ref: String,
    pub fingerprint: String,
    pub owner_id: String,
    pub contact_email: String,
    pub summary: String,
    pub status: TieredFlowLogManifestStatus,
}

impl TieredFlowLogManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFlowLogManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFlowLogManifestStatus) -> bool {
        match self.status {
            TieredFlowLogManifestStatus::Created => {
                return next == TieredFlowLogManifestStatus::Validated;
            }
            TieredFlowLogManifestStatus::Validated => {
                return next == TieredFlowLogManifestStatus::Published;
            }
            TieredFlowLogManifestStatus::Published => {
                match next {
                    TieredFlowLogManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredFlowLogManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered flow log manifest {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.created_at as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFlowLogManifest {
        TieredFlowLogManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            batch_size: self.batch_size,
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            owner_id: self.owner_id.clone(),
            contact_email: self.contact_email.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered flow log manifest in status created.
pub fn new_tiered_flow_log_manifest(code: &str, name: &str) -> TieredFlowLogManifest {
    TieredFlowLogManifest {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        batch_size: 0,
        created_at: 0,
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        owner_id: "".to_string(),
        contact_email: "".to_string(),
        summary: "".to_string(),
        status: TieredFlowLogManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_flow_log_manifest("code-344", "name-838");
        assert_eq!(item.status, TieredFlowLogManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_flow_log_manifest("code-344", "name-838");
        if item.move_to(TieredFlowLogManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFlowLogManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_flow_log_manifest("code-344", "name-838");
        item.status = TieredFlowLogManifestStatus::Retired;
        assert!(
            item.move_to(TieredFlowLogManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
