use super::*;

use acme_kit::text;

/// Where a dns record manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DnsRecordManifestStatus {
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

impl std::fmt::Display for DnsRecordManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifest {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub unit: String,
    pub burst: i64,
    pub cipher: String,
    pub rate: f64,
    pub status: DnsRecordManifestStatus,
}

impl DnsRecordManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordManifestStatus) -> bool {
        match self.status {
            DnsRecordManifestStatus::Draft => {
                match next {
                    DnsRecordManifestStatus::Pending | DnsRecordManifestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordManifestStatus::Pending => {
                match next {
                    DnsRecordManifestStatus::Active | DnsRecordManifestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordManifestStatus::Active => {
                match next {
                    DnsRecordManifestStatus::Suspended | DnsRecordManifestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordManifestStatus::Suspended => {
                match next {
                    DnsRecordManifestStatus::Closed => {
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

    pub fn move_to(&mut self, next: DnsRecordManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.burst as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordManifest {
        DnsRecordManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            unit: self.unit.clone(),
            burst: self.burst,
            cipher: self.cipher.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a dns record manifest in status draft.
pub fn new_dns_record_manifest(code: &str, name: &str) -> DnsRecordManifest {
    DnsRecordManifest {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        namespace: "".to_string(),
        ttl_seconds: 0,
        unit: "".to_string(),
        burst: 0,
        cipher: "".to_string(),
        rate: 0.0,
        status: DnsRecordManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dns_record_manifest("code-337", "name-64");
        assert_eq!(item.status, DnsRecordManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_dns_record_manifest("code-337", "name-64");
        if item.move_to(DnsRecordManifestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordManifestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dns_record_manifest("code-337", "name-64");
        item.status = DnsRecordManifestStatus::Closed;
        assert!(
            item.move_to(DnsRecordManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
