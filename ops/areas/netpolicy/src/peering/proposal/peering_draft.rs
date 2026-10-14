use super::*;

use acme_kit::text;

/// Where a peering draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PeeringDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PeeringDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraft {
    pub code: String,
    pub name: String,
    pub region: String,
    pub currency: String,
    pub label: String,
    pub locale: String,
    pub segment: String,
    pub ttl_seconds: i64,
    pub host: String,
    pub status: PeeringDraftStatus,
}

impl PeeringDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringDraftStatus) -> bool {
        match self.status {
            PeeringDraftStatus::Created => {
                return next == PeeringDraftStatus::Validated || next == PeeringDraftStatus::Published;
            }
            PeeringDraftStatus::Validated => {
                match next {
                    PeeringDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringDraftStatus::Published => {
                return next == PeeringDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "peering draft", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> PeeringDraft {
        PeeringDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            currency: self.currency.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            segment: self.segment.clone(),
            ttl_seconds: self.ttl_seconds,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a peering draft in status created.
pub fn new_peering_draft(code: &str, name: &str) -> PeeringDraft {
    PeeringDraft {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        currency: "".to_string(),
        label: "".to_string(),
        locale: "".to_string(),
        segment: "".to_string(),
        ttl_seconds: 0,
        host: "".to_string(),
        status: PeeringDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_peering_draft("code-907", "name-137");
        assert_eq!(item.status, PeeringDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_peering_draft("code-907", "name-137");
        if item.move_to(PeeringDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_peering_draft("code-907", "name-137");
        item.status = PeeringDraftStatus::Retired;
        assert!(
            item.move_to(PeeringDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
