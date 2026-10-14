use super::*;

use acme_kit::text;

/// Where a page audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PageAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAudit {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub contact_email: String,
    pub category: String,
    pub phone: String,
    pub approved: bool,
    pub amount: f64,
    pub status: PageAuditStatus,
}

impl PageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageAuditStatus) -> bool {
        match self.status {
            PageAuditStatus::Planned => {
                match next {
                    PageAuditStatus::InProgress | PageAuditStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAuditStatus::Scheduled => {
                return next == PageAuditStatus::Completed;
            }
            PageAuditStatus::InProgress => {
                match next {
                    PageAuditStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAuditStatus::Completed => {
                return next == PageAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "page audit", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> PageAudit {
        PageAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            contact_email: self.contact_email.clone(),
            category: self.category.clone(),
            phone: self.phone.clone(),
            approved: self.approved,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a page audit in status planned.
pub fn new_page_audit(code: &str, label: &str) -> PageAudit {
    PageAudit {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        contact_email: "".to_string(),
        category: "".to_string(),
        phone: "".to_string(),
        approved: false,
        amount: 0.0,
        status: PageAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_page_audit("code-13", "label-297");
        assert_eq!(item.status, PageAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_page_audit("code-13", "label-297");
        if item.move_to(PageAuditStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageAuditStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_page_audit("code-13", "label-297");
        item.status = PageAuditStatus::Cancelled;
        assert!(item.move_to(PageAuditStatus::Planned).is_err(), "expected move to fail");
    }
}
