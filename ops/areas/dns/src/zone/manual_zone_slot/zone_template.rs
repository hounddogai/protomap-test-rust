use super::*;

use acme_kit::text;

/// Where a zone template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTemplate {
    pub code: String,
    pub label: String,
    pub category: String,
    pub visible: bool,
    pub tenant_id: String,
    pub locale: String,
    pub archived: bool,
    pub amount: f64,
    pub currency: String,
    pub ratio: f64,
    pub updated_at: i64,
    pub status: ZoneTemplateStatus,
}

impl ZoneTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneTemplateStatus) -> bool {
        match self.status {
            ZoneTemplateStatus::Proposed => {
                match next {
                    ZoneTemplateStatus::Approved | ZoneTemplateStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneTemplateStatus::Approved => {
                return next == ZoneTemplateStatus::Withdrawn;
            }
            ZoneTemplateStatus::Rejected => {
                match next {
                    ZoneTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ZoneTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.amount * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneTemplate {
        ZoneTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            locale: self.locale.clone(),
            archived: self.archived,
            amount: self.amount,
            currency: self.currency.clone(),
            ratio: self.ratio,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a zone template in status proposed.
pub fn new_zone_template(code: &str, label: &str) -> ZoneTemplate {
    ZoneTemplate {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        visible: false,
        tenant_id: "".to_string(),
        locale: "".to_string(),
        archived: false,
        amount: 0.0,
        currency: "".to_string(),
        ratio: 0.0,
        updated_at: 0,
        status: ZoneTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_template("code-795", "label-85");
        assert_eq!(item.status, ZoneTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_zone_template("code-795", "label-85");
        if item.move_to(ZoneTemplateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneTemplateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_template("code-795", "label-85");
        item.status = ZoneTemplateStatus::Withdrawn;
        assert!(
            item.move_to(ZoneTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
