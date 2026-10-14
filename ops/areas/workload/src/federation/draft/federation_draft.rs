use super::*;

use acme_kit::text;

/// Where a federation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FederationDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FederationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation draft as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationDraft {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub limit: i64,
    pub summary: String,
    pub namespace: String,
    pub status: FederationDraftStatus,
}

impl FederationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationDraftStatus) -> bool {
        match self.status {
            FederationDraftStatus::Created => {
                match next {
                    FederationDraftStatus::Validated | FederationDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationDraftStatus::Validated => {
                match next {
                    FederationDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationDraftStatus::Published => {
                return next == FederationDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationDraft {
        FederationDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            limit: self.limit,
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a federation draft in status created.
pub fn new_federation_draft(code: &str, name: &str) -> FederationDraft {
    FederationDraft {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        limit: 0,
        summary: "".to_string(),
        namespace: "".to_string(),
        status: FederationDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_federation_draft("code-585", "name-811");
        assert_eq!(item.status, FederationDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_federation_draft("code-585", "name-811");
        if item.move_to(FederationDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_federation_draft("code-585", "name-811");
        item.status = FederationDraftStatus::Retired;
        assert!(
            item.move_to(FederationDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
