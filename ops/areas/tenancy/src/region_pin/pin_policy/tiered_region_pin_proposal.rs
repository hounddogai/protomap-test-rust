use super::*;

use acme_kit::text;

/// Where a tiered region pin proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRegionPinProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredRegionPinProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredRegionPinProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered region pin proposal as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinProposal {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub contact_email: String,
    pub category: String,
    pub country: String,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub region: String,
    pub status: TieredRegionPinProposalStatus,
}

impl TieredRegionPinProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRegionPinProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRegionPinProposalStatus) -> bool {
        match self.status {
            TieredRegionPinProposalStatus::Created => {
                match next {
                    TieredRegionPinProposalStatus::Published | TieredRegionPinProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRegionPinProposalStatus::Validated => {
                match next {
                    TieredRegionPinProposalStatus::Published | TieredRegionPinProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRegionPinProposalStatus::Published => {
                return next == TieredRegionPinProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRegionPinProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered region pin proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered region pin proposal", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRegionPinProposal {
        TieredRegionPinProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            contact_email: self.contact_email.clone(),
            category: self.category.clone(),
            country: self.country.clone(),
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered region pin proposal in status created.
pub fn new_tiered_region_pin_proposal(code: &str, name: &str) -> TieredRegionPinProposal {
    TieredRegionPinProposal {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        contact_email: "".to_string(),
        category: "".to_string(),
        country: "".to_string(),
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        region: "".to_string(),
        status: TieredRegionPinProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_region_pin_proposal("code-864", "name-867");
        assert_eq!(item.status, TieredRegionPinProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tiered_region_pin_proposal("code-864", "name-867");
        if item.move_to(TieredRegionPinProposalStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRegionPinProposalStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_region_pin_proposal("code-864", "name-867");
        item.status = TieredRegionPinProposalStatus::Retired;
        assert!(
            item.move_to(TieredRegionPinProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
