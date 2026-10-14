use super::*;

use acme_kit::text;

/// Where a page snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PageSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageSnapshot {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub minimum: f64,
    pub created_at: i64,
    pub namespace: String,
    pub attempts: i64,
    pub threshold: f64,
    pub summary: String,
    pub status: PageSnapshotStatus,
}

impl PageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageSnapshotStatus) -> bool {
        match self.status {
            PageSnapshotStatus::Created => {
                match next {
                    PageSnapshotStatus::Validated | PageSnapshotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageSnapshotStatus::Validated => {
                return next == PageSnapshotStatus::Published;
            }
            PageSnapshotStatus::Published => {
                match next {
                    PageSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: PageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page snapshot {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "page snapshot", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.created_at as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageSnapshot {
        PageSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            namespace: self.namespace.clone(),
            attempts: self.attempts,
            threshold: self.threshold,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a page snapshot in status created.
pub fn new_page_snapshot(code: &str, label: &str) -> PageSnapshot {
    PageSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        minimum: 0.0,
        created_at: 0,
        namespace: "".to_string(),
        attempts: 0,
        threshold: 0.0,
        summary: "".to_string(),
        status: PageSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_page_snapshot("code-610", "label-946");
        assert_eq!(item.status, PageSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_page_snapshot("code-610", "label-946");
        if item.move_to(PageSnapshotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageSnapshotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_page_snapshot("code-610", "label-946");
        item.status = PageSnapshotStatus::Retired;
        assert!(
            item.move_to(PageSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
