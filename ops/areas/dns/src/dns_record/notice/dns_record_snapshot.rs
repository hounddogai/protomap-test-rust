use super::*;

use acme_kit::text;

/// Where a dns record snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DnsRecordSnapshotStatus {
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

impl std::fmt::Display for DnsRecordSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record snapshot as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSnapshot {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub tenant_id: String,
    pub weight: f64,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub status: DnsRecordSnapshotStatus,
}

impl DnsRecordSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordSnapshotStatus) -> bool {
        match self.status {
            DnsRecordSnapshotStatus::Open => {
                return next == DnsRecordSnapshotStatus::Reopened;
            }
            DnsRecordSnapshotStatus::InReview => {
                return next == DnsRecordSnapshotStatus::Reopened;
            }
            DnsRecordSnapshotStatus::Resolved => {
                match next {
                    DnsRecordSnapshotStatus::Reopened | DnsRecordSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordSnapshotStatus::Reopened => {
                match next {
                    DnsRecordSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: DnsRecordSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dns record snapshot", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordSnapshot {
        DnsRecordSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            tenant_id: self.tenant_id.clone(),
            weight: self.weight,
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a dns record snapshot in status open.
pub fn new_dns_record_snapshot(code: &str, label: &str) -> DnsRecordSnapshot {
    DnsRecordSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        tenant_id: "".to_string(),
        weight: 0.0,
        cluster: "".to_string(),
        ttl_seconds: 0,
        status: DnsRecordSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dns_record_snapshot("code-838", "label-338");
        assert_eq!(item.status, DnsRecordSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_dns_record_snapshot("code-838", "label-338");
        if item.move_to(DnsRecordSnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordSnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dns_record_snapshot("code-838", "label-338");
        item.status = DnsRecordSnapshotStatus::Archived;
        assert!(
            item.move_to(DnsRecordSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
