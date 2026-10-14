use super::*;

use acme_kit::text;

/// Where a adaptive plan template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePlanTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptivePlanTemplateStatus {
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

impl std::fmt::Display for AdaptivePlanTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive plan template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanTemplate {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub budget: f64,
    pub owner_id: String,
    pub label: String,
    pub contact_email: String,
    pub phone: String,
    pub name: String,
    pub country: String,
    pub status: AdaptivePlanTemplateStatus,
}

impl AdaptivePlanTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePlanTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePlanTemplateStatus) -> bool {
        match self.status {
            AdaptivePlanTemplateStatus::Draft => {
                return next == AdaptivePlanTemplateStatus::Active || next == AdaptivePlanTemplateStatus::Closed;
            }
            AdaptivePlanTemplateStatus::Pending => {
                match next {
                    AdaptivePlanTemplateStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePlanTemplateStatus::Active => {
                return next == AdaptivePlanTemplateStatus::Closed;
            }
            AdaptivePlanTemplateStatus::Suspended => {
                match next {
                    AdaptivePlanTemplateStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptivePlanTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive plan template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> AdaptivePlanTemplate {
        AdaptivePlanTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            budget: self.budget,
            owner_id: self.owner_id.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            phone: self.phone.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive plan template in status draft.
pub fn new_adaptive_plan_template(code: &str, title: &str) -> AdaptivePlanTemplate {
    AdaptivePlanTemplate {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        budget: 0.0,
        owner_id: "".to_string(),
        label: "".to_string(),
        contact_email: "".to_string(),
        phone: "".to_string(),
        name: "".to_string(),
        country: "".to_string(),
        status: AdaptivePlanTemplateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_plan_template("code-516", "title-256");
        assert_eq!(item.status, AdaptivePlanTemplateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_adaptive_plan_template("code-516", "title-256");
        if item.move_to(AdaptivePlanTemplateStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePlanTemplateStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_plan_template("code-516", "title-256");
        item.status = AdaptivePlanTemplateStatus::Closed;
        assert!(
            item.move_to(AdaptivePlanTemplateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
