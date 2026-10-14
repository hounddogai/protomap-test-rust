use super::*;

use acme_kit::text;

/// Where a tiered page proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPageProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredPageProposalStatus {
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

impl std::fmt::Display for TieredPageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered page proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageProposal {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub region: String,
    pub floor_value: f64,
    pub ceiling: f64,
    pub minimum: f64,
    pub cipher: String,
    pub visible: bool,
    pub checksum: String,
    pub status: TieredPageProposalStatus,
}

impl TieredPageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPageProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPageProposalStatus) -> bool {
        match self.status {
            TieredPageProposalStatus::Draft => {
                return next == TieredPageProposalStatus::Closed;
            }
            TieredPageProposalStatus::Pending => {
                match next {
                    TieredPageProposalStatus::Active | TieredPageProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPageProposalStatus::Active => {
                return next == TieredPageProposalStatus::Suspended || next == TieredPageProposalStatus::Closed;
            }
            TieredPageProposalStatus::Suspended => {
                return next == TieredPageProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredPageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered page proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered page proposal", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.minimum * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPageProposal {
        TieredPageProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            region: self.region.clone(),
            floor_value: self.floor_value,
            ceiling: self.ceiling,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            visible: self.visible,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered page proposal in status draft.
pub fn new_tiered_page_proposal(code: &str, label: &str) -> TieredPageProposal {
    TieredPageProposal {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        region: "".to_string(),
        floor_value: 0.0,
        ceiling: 0.0,
        minimum: 0.0,
        cipher: "".to_string(),
        visible: false,
        checksum: "".to_string(),
        status: TieredPageProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_page_proposal("code-995", "label-590");
        assert_eq!(item.status, TieredPageProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_tiered_page_proposal("code-995", "label-590");
        if item.move_to(TieredPageProposalStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPageProposalStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_page_proposal("code-995", "label-590");
        item.status = TieredPageProposalStatus::Closed;
        assert!(
            item.move_to(TieredPageProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
