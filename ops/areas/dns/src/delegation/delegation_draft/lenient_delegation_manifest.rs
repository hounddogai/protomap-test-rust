use super::*;

use acme_kit::text;

/// Where a lenient delegation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDelegationManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientDelegationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientDelegationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient delegation manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifest {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub batch_size: i64,
    pub approved: bool,
    pub status: LenientDelegationManifestStatus,
}

impl LenientDelegationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDelegationManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDelegationManifestStatus) -> bool {
        match self.status {
            LenientDelegationManifestStatus::Created => {
                return next == LenientDelegationManifestStatus::Retired;
            }
            LenientDelegationManifestStatus::Validated => {
                match next {
                    LenientDelegationManifestStatus::Published | LenientDelegationManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDelegationManifestStatus::Published => {
                return next == LenientDelegationManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDelegationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient delegation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient delegation manifest", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDelegationManifest {
        LenientDelegationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            batch_size: self.batch_size,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient delegation manifest in status created.
pub fn new_lenient_delegation_manifest(code: &str, title: &str) -> LenientDelegationManifest {
    LenientDelegationManifest {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        batch_size: 0,
        approved: false,
        status: LenientDelegationManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_delegation_manifest("code-153", "title-466");
        assert_eq!(item.status, LenientDelegationManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_lenient_delegation_manifest("code-153", "title-466");
        if item.move_to(LenientDelegationManifestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDelegationManifestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_delegation_manifest("code-153", "title-466");
        item.status = LenientDelegationManifestStatus::Retired;
        assert!(
            item.move_to(LenientDelegationManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
