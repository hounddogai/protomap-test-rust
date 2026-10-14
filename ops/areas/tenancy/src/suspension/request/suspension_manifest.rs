use super::*;

use acme_kit::text;

/// Where a suspension manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionManifest {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub port: i64,
    pub limit: i64,
    pub ttl_seconds: i64,
    pub discount: f64,
    pub batch_size: i64,
    pub reviewer: String,
    pub description: String,
    pub retries: i64,
    pub status: SuspensionManifestStatus,
}

impl SuspensionManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionManifestStatus) -> bool {
        match self.status {
            SuspensionManifestStatus::Requested => {
                return next == SuspensionManifestStatus::Expired;
            }
            SuspensionManifestStatus::Granted => {
                match next {
                    SuspensionManifestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionManifestStatus::Revoked => {
                match next {
                    SuspensionManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: SuspensionManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension manifest", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.retries as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionManifest {
        SuspensionManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            port: self.port,
            limit: self.limit,
            ttl_seconds: self.ttl_seconds,
            discount: self.discount,
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            description: self.description.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a suspension manifest in status requested.
pub fn new_suspension_manifest(code: &str, title: &str) -> SuspensionManifest {
    SuspensionManifest {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        port: 0,
        limit: 0,
        ttl_seconds: 0,
        discount: 0.0,
        batch_size: 0,
        reviewer: "".to_string(),
        description: "".to_string(),
        retries: 0,
        status: SuspensionManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_suspension_manifest("code-230", "title-373");
        assert_eq!(item.status, SuspensionManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_suspension_manifest("code-230", "title-373");
        if item.move_to(SuspensionManifestStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionManifestStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_suspension_manifest("code-230", "title-373");
        item.status = SuspensionManifestStatus::Expired;
        assert!(
            item.move_to(SuspensionManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
