use super::*;

use acme_kit::text;

/// Where a dns record item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordItemStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DnsRecordItemStatus {
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

impl std::fmt::Display for DnsRecordItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordItem {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub retries: i64,
    pub tags: Vec<String>,
    pub rotation_days: i64,
    pub grace_days: i64,
    pub visible: bool,
    pub status: DnsRecordItemStatus,
}

impl DnsRecordItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordItemStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordItemStatus) -> bool {
        match self.status {
            DnsRecordItemStatus::Open => {
                return next == DnsRecordItemStatus::Archived;
            }
            DnsRecordItemStatus::InReview => {
                return next == DnsRecordItemStatus::Archived;
            }
            DnsRecordItemStatus::Resolved => {
                return next == DnsRecordItemStatus::Reopened;
            }
            DnsRecordItemStatus::Reopened => {
                return next == DnsRecordItemStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rotation_days as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordItem {
        DnsRecordItem {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            retries: self.retries,
            tags: self.tags.clone(),
            rotation_days: self.rotation_days,
            grace_days: self.grace_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record item in status open.
pub fn new_dns_record_item(code: &str, label: &str) -> DnsRecordItem {
    DnsRecordItem {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        retries: 0,
        tags: Vec::new(),
        rotation_days: 0,
        grace_days: 0,
        visible: false,
        status: DnsRecordItemStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dns_record_item("code-974", "label-391");
        assert_eq!(item.status, DnsRecordItemStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_dns_record_item("code-974", "label-391");
        if item.move_to(DnsRecordItemStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordItemStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dns_record_item("code-974", "label-391");
        item.status = DnsRecordItemStatus::Archived;
        assert!(
            item.move_to(DnsRecordItemStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
