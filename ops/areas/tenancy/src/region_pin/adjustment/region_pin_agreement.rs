use super::*;

use acme_kit::text;

/// Where a region pin agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreement {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub batch_size: i64,
    pub quantity: i64,
    pub visible: bool,
    pub principal: String,
    pub limit: i64,
    pub capacity: i64,
    pub region: String,
    pub status: RegionPinAgreementStatus,
}

impl RegionPinAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinAgreementStatus) -> bool {
        match self.status {
            RegionPinAgreementStatus::Created => {
                match next {
                    RegionPinAgreementStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinAgreementStatus::Validated => {
                return next == RegionPinAgreementStatus::Published || next == RegionPinAgreementStatus::Retired;
            }
            RegionPinAgreementStatus::Published => {
                return next == RegionPinAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.limit as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinAgreement {
        RegionPinAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            quantity: self.quantity,
            visible: self.visible,
            principal: self.principal.clone(),
            limit: self.limit,
            capacity: self.capacity,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a region pin agreement in status created.
pub fn new_region_pin_agreement(code: &str, title: &str) -> RegionPinAgreement {
    RegionPinAgreement {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        batch_size: 0,
        quantity: 0,
        visible: false,
        principal: "".to_string(),
        limit: 0,
        capacity: 0,
        region: "".to_string(),
        status: RegionPinAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_region_pin_agreement("code-117", "title-278");
        assert_eq!(item.status, RegionPinAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_region_pin_agreement("code-117", "title-278");
        if item.move_to(RegionPinAgreementStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinAgreementStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_region_pin_agreement("code-117", "title-278");
        item.status = RegionPinAgreementStatus::Retired;
        assert!(
            item.move_to(RegionPinAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
