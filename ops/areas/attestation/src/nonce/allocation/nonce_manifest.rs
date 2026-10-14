use super::*;

use acme_kit::text;

/// Where a nonce manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl NonceManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for NonceManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceManifest {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub audience: String,
    pub weight: f64,
    pub description: String,
    pub namespace: String,
    pub key_id: String,
    pub region: String,
    pub budget: f64,
    pub status: NonceManifestStatus,
}

impl NonceManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceManifestStatus) -> bool {
        match self.status {
            NonceManifestStatus::Created => {
                match next {
                    NonceManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceManifestStatus::Validated => {
                return next == NonceManifestStatus::Published || next == NonceManifestStatus::Retired;
            }
            NonceManifestStatus::Published => {
                match next {
                    NonceManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: NonceManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce manifest", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.budget * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceManifest {
        NonceManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            audience: self.audience.clone(),
            weight: self.weight,
            description: self.description.clone(),
            namespace: self.namespace.clone(),
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a nonce manifest in status created.
pub fn new_nonce_manifest(code: &str, title: &str) -> NonceManifest {
    NonceManifest {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        audience: "".to_string(),
        weight: 0.0,
        description: "".to_string(),
        namespace: "".to_string(),
        key_id: "".to_string(),
        region: "".to_string(),
        budget: 0.0,
        status: NonceManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_nonce_manifest("code-660", "title-537");
        assert_eq!(item.status, NonceManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_nonce_manifest("code-660", "title-537");
        if item.move_to(NonceManifestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceManifestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_nonce_manifest("code-660", "title-537");
        item.status = NonceManifestStatus::Retired;
        assert!(
            item.move_to(NonceManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
