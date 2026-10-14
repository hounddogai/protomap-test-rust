use super::*;

use acme_kit::text;

/// Where a dns record tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DnsRecordTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DnsRecordTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTier {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub description: String,
    pub category: String,
    pub source: String,
    pub namespace: String,
    pub threshold: f64,
    pub status: DnsRecordTierStatus,
}

impl DnsRecordTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordTierStatus) -> bool {
        match self.status {
            DnsRecordTierStatus::Requested => {
                match next {
                    DnsRecordTierStatus::Revoked | DnsRecordTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordTierStatus::Granted => {
                match next {
                    DnsRecordTierStatus::Revoked | DnsRecordTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordTierStatus::Revoked => {
                return next == DnsRecordTierStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dns record tier", self.code)
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
        self.threshold
    }

    pub fn copy(&self) -> DnsRecordTier {
        DnsRecordTier {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            source: self.source.clone(),
            namespace: self.namespace.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a dns record tier in status requested.
pub fn new_dns_record_tier(code: &str, label: &str) -> DnsRecordTier {
    DnsRecordTier {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        description: "".to_string(),
        category: "".to_string(),
        source: "".to_string(),
        namespace: "".to_string(),
        threshold: 0.0,
        status: DnsRecordTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dns_record_tier("code-941", "label-759");
        assert_eq!(item.status, DnsRecordTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_dns_record_tier("code-941", "label-759");
        if item.move_to(DnsRecordTierStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordTierStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dns_record_tier("code-941", "label-759");
        item.status = DnsRecordTierStatus::Expired;
        assert!(
            item.move_to(DnsRecordTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
