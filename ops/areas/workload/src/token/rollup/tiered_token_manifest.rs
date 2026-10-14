use super::*;

use acme_kit::text;

/// Where a tiered token manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTokenManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredTokenManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredTokenManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered token manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifest {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub tags: Vec<String>,
    pub ttl_seconds: i64,
    pub amount: f64,
    pub approved: bool,
    pub floor_value: f64,
    pub issuer: String,
    pub cipher: String,
    pub unit: String,
    pub status: TieredTokenManifestStatus,
}

impl TieredTokenManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTokenManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTokenManifestStatus) -> bool {
        match self.status {
            TieredTokenManifestStatus::Created => {
                match next {
                    TieredTokenManifestStatus::Validated | TieredTokenManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTokenManifestStatus::Validated => {
                return next == TieredTokenManifestStatus::Published || next == TieredTokenManifestStatus::Retired;
            }
            TieredTokenManifestStatus::Published => {
                match next {
                    TieredTokenManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredTokenManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered token manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTokenManifest {
        TieredTokenManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            tags: self.tags.clone(),
            ttl_seconds: self.ttl_seconds,
            amount: self.amount,
            approved: self.approved,
            floor_value: self.floor_value,
            issuer: self.issuer.clone(),
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered token manifest in status created.
pub fn new_tiered_token_manifest(code: &str, title: &str) -> TieredTokenManifest {
    TieredTokenManifest {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        tags: Vec::new(),
        ttl_seconds: 0,
        amount: 0.0,
        approved: false,
        floor_value: 0.0,
        issuer: "".to_string(),
        cipher: "".to_string(),
        unit: "".to_string(),
        status: TieredTokenManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_token_manifest("code-275", "title-532");
        assert_eq!(item.status, TieredTokenManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_token_manifest("code-275", "title-532");
        if item.move_to(TieredTokenManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTokenManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_token_manifest("code-275", "title-532");
        item.status = TieredTokenManifestStatus::Retired;
        assert!(
            item.move_to(TieredTokenManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
