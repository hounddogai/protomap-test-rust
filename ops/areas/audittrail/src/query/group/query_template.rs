use super::*;

use acme_kit::text;

/// Where a query template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplate {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub principal: String,
    pub cost: f64,
    pub discount: f64,
    pub host: String,
    pub visible: bool,
    pub status: QueryTemplateStatus,
}

impl QueryTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryTemplateStatus) -> bool {
        match self.status {
            QueryTemplateStatus::Proposed => {
                return next == QueryTemplateStatus::Approved || next == QueryTemplateStatus::Rejected;
            }
            QueryTemplateStatus::Approved => {
                match next {
                    QueryTemplateStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryTemplateStatus::Rejected => {
                return next == QueryTemplateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query template", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        (self.cost * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryTemplate {
        QueryTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            principal: self.principal.clone(),
            cost: self.cost,
            discount: self.discount,
            host: self.host.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query template in status proposed.
pub fn new_query_template(code: &str, title: &str) -> QueryTemplate {
    QueryTemplate {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        principal: "".to_string(),
        cost: 0.0,
        discount: 0.0,
        host: "".to_string(),
        visible: false,
        status: QueryTemplateStatus::Proposed,
    }
}
