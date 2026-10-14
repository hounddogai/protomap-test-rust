use super::*;

use acme_kit::text;

/// Where a batched zone manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedZoneManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedZoneManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BatchedZoneManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched zone manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneManifest {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub duration_minutes: i64,
    pub starts_at: i64,
    pub channel: String,
    pub cipher: String,
    pub status: BatchedZoneManifestStatus,
}

impl BatchedZoneManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedZoneManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedZoneManifestStatus) -> bool {
        match self.status {
            BatchedZoneManifestStatus::Open => {
                match next {
                    BatchedZoneManifestStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedZoneManifestStatus::InReview => {
                return next == BatchedZoneManifestStatus::Resolved || next == BatchedZoneManifestStatus::Reopened;
            }
            BatchedZoneManifestStatus::Resolved => {
                match next {
                    BatchedZoneManifestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedZoneManifestStatus::Reopened => {
                match next {
                    BatchedZoneManifestStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedZoneManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched zone manifest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedZoneManifest {
        BatchedZoneManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            duration_minutes: self.duration_minutes,
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched zone manifest in status open.
pub fn new_batched_zone_manifest(code: &str, label: &str) -> BatchedZoneManifest {
    BatchedZoneManifest {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        duration_minutes: 0,
        starts_at: 0,
        channel: "".to_string(),
        cipher: "".to_string(),
        status: BatchedZoneManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_zone_manifest("code-339", "label-946");
        assert_eq!(item.status, BatchedZoneManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_zone_manifest("code-339", "label-946");
        if item.move_to(BatchedZoneManifestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedZoneManifestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_zone_manifest("code-339", "label-946");
        item.status = BatchedZoneManifestStatus::Archived;
        assert!(
            item.move_to(BatchedZoneManifestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
