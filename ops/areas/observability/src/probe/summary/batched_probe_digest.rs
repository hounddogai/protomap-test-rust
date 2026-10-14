use super::*;

use acme_kit::text;

/// Where a batched probe digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedProbeDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedProbeDigestStatus {
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

impl std::fmt::Display for BatchedProbeDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched probe digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeDigest {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub region: String,
    pub starts_at: i64,
    pub threshold: f64,
    pub checksum: String,
    pub status: BatchedProbeDigestStatus,
}

impl BatchedProbeDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedProbeDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedProbeDigestStatus) -> bool {
        match self.status {
            BatchedProbeDigestStatus::Draft => {
                return next == BatchedProbeDigestStatus::Suspended;
            }
            BatchedProbeDigestStatus::Pending => {
                match next {
                    BatchedProbeDigestStatus::Active | BatchedProbeDigestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedProbeDigestStatus::Active => {
                return next == BatchedProbeDigestStatus::Suspended || next == BatchedProbeDigestStatus::Closed;
            }
            BatchedProbeDigestStatus::Suspended => {
                match next {
                    BatchedProbeDigestStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedProbeDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched probe digest {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.threshold * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedProbeDigest {
        BatchedProbeDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            region: self.region.clone(),
            starts_at: self.starts_at,
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched probe digest in status draft.
pub fn new_batched_probe_digest(code: &str, name: &str) -> BatchedProbeDigest {
    BatchedProbeDigest {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        window_days: 0,
        floor_value: 0.0,
        region: "".to_string(),
        starts_at: 0,
        threshold: 0.0,
        checksum: "".to_string(),
        status: BatchedProbeDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_probe_digest("code-390", "name-520");
        assert_eq!(item.status, BatchedProbeDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_batched_probe_digest("code-390", "name-520");
        if item.move_to(BatchedProbeDigestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedProbeDigestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_probe_digest("code-390", "name-520");
        item.status = BatchedProbeDigestStatus::Closed;
        assert!(
            item.move_to(BatchedProbeDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
