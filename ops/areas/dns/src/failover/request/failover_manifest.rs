use super::*;

use acme_kit::text;

/// Where a failover manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FailoverManifestStatus {
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

impl std::fmt::Display for FailoverManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverManifest {
    pub code: String,
    pub title: String,
    pub host: String,
    pub contact_email: String,
    pub principal: String,
    pub port: i64,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub sequence: i64,
    pub minimum: f64,
    pub status: FailoverManifestStatus,
}

impl FailoverManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverManifestStatus) -> bool {
        match self.status {
            FailoverManifestStatus::Draft => {
                match next {
                    FailoverManifestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverManifestStatus::Pending => {
                match next {
                    FailoverManifestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverManifestStatus::Active => {
                match next {
                    FailoverManifestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverManifestStatus::Suspended => {
                return next == FailoverManifestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverManifest {
        FailoverManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            contact_email: self.contact_email.clone(),
            principal: self.principal.clone(),
            port: self.port,
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            sequence: self.sequence,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a failover manifest in status draft.
pub fn new_failover_manifest(code: &str, title: &str) -> FailoverManifest {
    FailoverManifest {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        contact_email: "".to_string(),
        principal: "".to_string(),
        port: 0,
        burst: 0,
        ttl_seconds: 0,
        sequence: 0,
        minimum: 0.0,
        status: FailoverManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_failover_manifest("code-385", "title-793");
        assert_eq!(item.status, FailoverManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_failover_manifest("code-385", "title-793");
        if item.move_to(FailoverManifestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverManifestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_failover_manifest("code-385", "title-793");
        item.status = FailoverManifestStatus::Closed;
        assert!(
            item.move_to(FailoverManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
