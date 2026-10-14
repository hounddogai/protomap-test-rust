use super::*;

use acme_kit::text;

/// Where a pending dashboard batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDashboardBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingDashboardBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingDashboardBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending dashboard batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBatch {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub checksum: String,
    pub tags: Vec<String>,
    pub port: i64,
    pub status: PendingDashboardBatchStatus,
}

impl PendingDashboardBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDashboardBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDashboardBatchStatus) -> bool {
        match self.status {
            PendingDashboardBatchStatus::Created => {
                return next == PendingDashboardBatchStatus::Validated;
            }
            PendingDashboardBatchStatus::Validated => {
                return next == PendingDashboardBatchStatus::Retired;
            }
            PendingDashboardBatchStatus::Published => {
                return next == PendingDashboardBatchStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingDashboardBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending dashboard batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDashboardBatch {
        PendingDashboardBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            checksum: self.checksum.clone(),
            tags: self.tags.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending dashboard batch in status created.
pub fn new_pending_dashboard_batch(code: &str, title: &str) -> PendingDashboardBatch {
    PendingDashboardBatch {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        checksum: "".to_string(),
        tags: Vec::new(),
        port: 0,
        status: PendingDashboardBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_dashboard_batch("code-116", "title-42");
        assert_eq!(item.status, PendingDashboardBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_dashboard_batch("code-116", "title-42");
        if item.move_to(PendingDashboardBatchStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDashboardBatchStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_dashboard_batch("code-116", "title-42");
        item.status = PendingDashboardBatchStatus::Retired;
        assert!(
            item.move_to(PendingDashboardBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
