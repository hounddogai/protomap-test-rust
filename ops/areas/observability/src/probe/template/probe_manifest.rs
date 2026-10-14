use super::*;

use acme_kit::text;

/// Where a probe manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeManifest {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub owner_id: String,
    pub revision: i64,
    pub starts_at: i64,
    pub port: i64,
    pub category: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub host: String,
    pub status: ProbeManifestStatus,
}

impl ProbeManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeManifestStatus) -> bool {
        match self.status {
            ProbeManifestStatus::Created => {
                return next == ProbeManifestStatus::Published;
            }
            ProbeManifestStatus::Validated => {
                match next {
                    ProbeManifestStatus::Published | ProbeManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeManifestStatus::Published => {
                match next {
                    ProbeManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: ProbeManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe manifest", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeManifest {
        ProbeManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            owner_id: self.owner_id.clone(),
            revision: self.revision,
            starts_at: self.starts_at,
            port: self.port,
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a probe manifest in status created.
pub fn new_probe_manifest(code: &str, label: &str) -> ProbeManifest {
    ProbeManifest {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        owner_id: "".to_string(),
        revision: 0,
        starts_at: 0,
        port: 0,
        category: "".to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        host: "".to_string(),
        status: ProbeManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_probe_manifest("code-792", "label-652");
        assert_eq!(item.status, ProbeManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_probe_manifest("code-792", "label-652");
        if item.move_to(ProbeManifestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeManifestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_probe_manifest("code-792", "label-652");
        item.status = ProbeManifestStatus::Retired;
        assert!(
            item.move_to(ProbeManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
