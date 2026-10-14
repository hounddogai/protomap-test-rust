use super::*;

use acme_kit::text;

/// Where a tenant segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TenantSegmentStatus {
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

impl std::fmt::Display for TenantSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSegment {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub region: String,
    pub budget: f64,
    pub visible: bool,
    pub amount: f64,
    pub summary: String,
    pub currency: String,
    pub timezone: String,
    pub maximum: f64,
    pub status: TenantSegmentStatus,
}

impl TenantSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantSegmentStatus) -> bool {
        match self.status {
            TenantSegmentStatus::Draft => {
                match next {
                    TenantSegmentStatus::Pending | TenantSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSegmentStatus::Pending => {
                return next == TenantSegmentStatus::Active || next == TenantSegmentStatus::Closed;
            }
            TenantSegmentStatus::Active => {
                match next {
                    TenantSegmentStatus::Suspended | TenantSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSegmentStatus::Suspended => {
                match next {
                    TenantSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: TenantSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tenant segment", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.port as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantSegment {
        TenantSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            region: self.region.clone(),
            budget: self.budget,
            visible: self.visible,
            amount: self.amount,
            summary: self.summary.clone(),
            currency: self.currency.clone(),
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tenant segment in status draft.
pub fn new_tenant_segment(code: &str, label: &str) -> TenantSegment {
    TenantSegment {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        region: "".to_string(),
        budget: 0.0,
        visible: false,
        amount: 0.0,
        summary: "".to_string(),
        currency: "".to_string(),
        timezone: "".to_string(),
        maximum: 0.0,
        status: TenantSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tenant_segment("code-892", "label-322");
        assert_eq!(item.status, TenantSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tenant_segment("code-892", "label-322");
        if item.move_to(TenantSegmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantSegmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tenant_segment("code-892", "label-322");
        item.status = TenantSegmentStatus::Closed;
        assert!(
            item.move_to(TenantSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
