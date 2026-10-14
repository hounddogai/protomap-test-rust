use super::*;

use acme_kit::text;

/// Where a page template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PageTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page template as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplate {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub archived: bool,
    pub score: f64,
    pub quota_units: i64,
    pub issuer: String,
    pub attempts: i64,
    pub status: PageTemplateStatus,
}

impl PageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageTemplateStatus) -> bool {
        match self.status {
            PageTemplateStatus::Proposed => {
                return next == PageTemplateStatus::Withdrawn;
            }
            PageTemplateStatus::Approved => {
                return next == PageTemplateStatus::Rejected || next == PageTemplateStatus::Withdrawn;
            }
            PageTemplateStatus::Rejected => {
                match next {
                    PageTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "page template", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageTemplate {
        PageTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            archived: self.archived,
            score: self.score,
            quota_units: self.quota_units,
            issuer: self.issuer.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a page template in status proposed.
pub fn new_page_template(code: &str, label: &str) -> PageTemplate {
    PageTemplate {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        archived: false,
        score: 0.0,
        quota_units: 0,
        issuer: "".to_string(),
        attempts: 0,
        status: PageTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_page_template("code-125", "label-737");
        assert_eq!(item.status, PageTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_page_template("code-125", "label-737");
        if item.move_to(PageTemplateStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageTemplateStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_page_template("code-125", "label-737");
        item.status = PageTemplateStatus::Withdrawn;
        assert!(
            item.move_to(PageTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
