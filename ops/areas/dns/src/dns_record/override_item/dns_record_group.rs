use super::*;

use acme_kit::text;

/// Where a dns record group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DnsRecordGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DnsRecordGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroup {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub weight: f64,
    pub starts_at: i64,
    pub port: i64,
    pub pinned: bool,
    pub discount: f64,
    pub retries: i64,
    pub status: DnsRecordGroupStatus,
}

impl DnsRecordGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordGroupStatus) -> bool {
        match self.status {
            DnsRecordGroupStatus::Created => {
                match next {
                    DnsRecordGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordGroupStatus::Validated => {
                return next == DnsRecordGroupStatus::Published || next == DnsRecordGroupStatus::Retired;
            }
            DnsRecordGroupStatus::Published => {
                match next {
                    DnsRecordGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: DnsRecordGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record group {} cannot move from {} to {}",
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
        format!("{} {}", "dns record group", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.port as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordGroup {
        DnsRecordGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            weight: self.weight,
            starts_at: self.starts_at,
            port: self.port,
            pinned: self.pinned,
            discount: self.discount,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record group in status created.
pub fn new_dns_record_group(code: &str, title: &str) -> DnsRecordGroup {
    DnsRecordGroup {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        weight: 0.0,
        starts_at: 0,
        port: 0,
        pinned: false,
        discount: 0.0,
        retries: 0,
        status: DnsRecordGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_dns_record_group("code-643", "title-964");
        assert_eq!(item.status, DnsRecordGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_dns_record_group("code-643", "title-964");
        if item.move_to(DnsRecordGroupStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordGroupStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_dns_record_group("code-643", "title-964");
        item.status = DnsRecordGroupStatus::Retired;
        assert!(
            item.move_to(DnsRecordGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
