use super::*;

use acme_kit::text;

/// Where a dns record override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DnsRecordOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DnsRecordOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverride {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub owner_id: String,
    pub revision: i64,
    pub label: String,
    pub tags: Vec<String>,
    pub status: DnsRecordOverrideStatus,
}

impl DnsRecordOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordOverrideStatus) -> bool {
        match self.status {
            DnsRecordOverrideStatus::Proposed => {
                match next {
                    DnsRecordOverrideStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordOverrideStatus::Approved => {
                match next {
                    DnsRecordOverrideStatus::Rejected | DnsRecordOverrideStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordOverrideStatus::Rejected => {
                match next {
                    DnsRecordOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DnsRecordOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dns record override", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> DnsRecordOverride {
        DnsRecordOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            owner_id: self.owner_id.clone(),
            revision: self.revision,
            label: self.label.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record override in status proposed.
pub fn new_dns_record_override(code: &str, title: &str) -> DnsRecordOverride {
    DnsRecordOverride {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        owner_id: "".to_string(),
        revision: 0,
        label: "".to_string(),
        tags: Vec::new(),
        status: DnsRecordOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_dns_record_override("code-916", "title-29");
        assert_eq!(item.status, DnsRecordOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_dns_record_override("code-916", "title-29");
        if item.move_to(DnsRecordOverrideStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordOverrideStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_dns_record_override("code-916", "title-29");
        item.status = DnsRecordOverrideStatus::Withdrawn;
        assert!(
            item.move_to(DnsRecordOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
