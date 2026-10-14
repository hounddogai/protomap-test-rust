use super::*;

use acme_kit::text;

/// Where a pending data export tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDataExportTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingDataExportTierStatus {
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

impl std::fmt::Display for PendingDataExportTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending data export tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDataExportTier {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub timezone: String,
    pub quantity: i64,
    pub discount: f64,
    pub unit: String,
    pub description: String,
    pub tenant_id: String,
    pub cipher: String,
    pub status: PendingDataExportTierStatus,
}

impl PendingDataExportTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDataExportTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDataExportTierStatus) -> bool {
        match self.status {
            PendingDataExportTierStatus::Draft => {
                return next == PendingDataExportTierStatus::Active || next == PendingDataExportTierStatus::Closed;
            }
            PendingDataExportTierStatus::Pending => {
                match next {
                    PendingDataExportTierStatus::Active | PendingDataExportTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDataExportTierStatus::Active => {
                match next {
                    PendingDataExportTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDataExportTierStatus::Suspended => {
                return next == PendingDataExportTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingDataExportTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending data export tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDataExportTier {
        PendingDataExportTier {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            discount: self.discount,
            unit: self.unit.clone(),
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending data export tier in status draft.
pub fn new_pending_data_export_tier(code: &str, title: &str) -> PendingDataExportTier {
    PendingDataExportTier {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        timezone: "".to_string(),
        quantity: 0,
        discount: 0.0,
        unit: "".to_string(),
        description: "".to_string(),
        tenant_id: "".to_string(),
        cipher: "".to_string(),
        status: PendingDataExportTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_data_export_tier("code-107", "title-841");
        assert_eq!(item.status, PendingDataExportTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_data_export_tier("code-107", "title-841");
        if item.move_to(PendingDataExportTierStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDataExportTierStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_data_export_tier("code-107", "title-841");
        item.status = PendingDataExportTierStatus::Closed;
        assert!(
            item.move_to(PendingDataExportTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
