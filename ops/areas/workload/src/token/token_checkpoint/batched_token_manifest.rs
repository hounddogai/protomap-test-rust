use super::*;

use acme_kit::text;

/// Where a batched token manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTokenManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedTokenManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedTokenManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched token manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenManifest {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub rate: f64,
    pub fingerprint: String,
    pub expires_at: i64,
    pub locale: String,
    pub starts_at: i64,
    pub cluster: String,
    pub limit: i64,
    pub port: i64,
    pub status: BatchedTokenManifestStatus,
}

impl BatchedTokenManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTokenManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTokenManifestStatus) -> bool {
        match self.status {
            BatchedTokenManifestStatus::Created => {
                return next == BatchedTokenManifestStatus::Validated || next == BatchedTokenManifestStatus::Retired;
            }
            BatchedTokenManifestStatus::Validated => {
                match next {
                    BatchedTokenManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTokenManifestStatus::Published => {
                return next == BatchedTokenManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTokenManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched token manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched token manifest", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.port as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTokenManifest {
        BatchedTokenManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            expires_at: self.expires_at,
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            cluster: self.cluster.clone(),
            limit: self.limit,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched token manifest in status created.
pub fn new_batched_token_manifest(code: &str, name: &str) -> BatchedTokenManifest {
    BatchedTokenManifest {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        rate: 0.0,
        fingerprint: "".to_string(),
        expires_at: 0,
        locale: "".to_string(),
        starts_at: 0,
        cluster: "".to_string(),
        limit: 0,
        port: 0,
        status: BatchedTokenManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_token_manifest("code-277", "name-223");
        assert_eq!(item.status, BatchedTokenManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_token_manifest("code-277", "name-223");
        if item.move_to(BatchedTokenManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTokenManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_token_manifest("code-277", "name-223");
        item.status = BatchedTokenManifestStatus::Retired;
        assert!(
            item.move_to(BatchedTokenManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
