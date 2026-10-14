use super::*;

use acme_kit::text;

/// Where a dns record mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DnsRecordMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DnsRecordMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMapping {
    pub code: String,
    pub label: String,
    pub description: String,
    pub timezone: String,
    pub expires_at: i64,
    pub status: DnsRecordMappingStatus,
}

impl DnsRecordMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordMappingStatus) -> bool {
        match self.status {
            DnsRecordMappingStatus::Created => {
                match next {
                    DnsRecordMappingStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordMappingStatus::Validated => {
                return next == DnsRecordMappingStatus::Published || next == DnsRecordMappingStatus::Retired;
            }
            DnsRecordMappingStatus::Published => {
                match next {
                    DnsRecordMappingStatus::Retired => {
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

    pub fn move_to(&mut self, next: DnsRecordMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record mapping {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> DnsRecordMapping {
        DnsRecordMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            timezone: self.timezone.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record mapping in status created.
pub fn new_dns_record_mapping(code: &str, label: &str) -> DnsRecordMapping {
    DnsRecordMapping {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        timezone: "".to_string(),
        expires_at: 0,
        status: DnsRecordMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_dns_record_mapping("code-966", "label-568");
        assert_eq!(item.status, DnsRecordMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_dns_record_mapping("code-966", "label-568");
        if item.move_to(DnsRecordMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_dns_record_mapping("code-966", "label-568");
        item.status = DnsRecordMappingStatus::Retired;
        assert!(
            item.move_to(DnsRecordMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
