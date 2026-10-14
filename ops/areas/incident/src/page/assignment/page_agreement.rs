use super::*;

use acme_kit::text;

/// Where a page agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PageAgreementStatus {
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

impl std::fmt::Display for PageAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAgreement {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub port: i64,
    pub burst: i64,
    pub active: bool,
    pub title: String,
    pub starts_at: i64,
    pub approved: bool,
    pub ends_at: i64,
    pub status: PageAgreementStatus,
}

impl PageAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageAgreementStatus) -> bool {
        match self.status {
            PageAgreementStatus::Planned => {
                return next == PageAgreementStatus::InProgress || next == PageAgreementStatus::Completed;
            }
            PageAgreementStatus::Scheduled => {
                match next {
                    PageAgreementStatus::InProgress | PageAgreementStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAgreementStatus::InProgress => {
                match next {
                    PageAgreementStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAgreementStatus::Completed => {
                match next {
                    PageAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PageAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ends_at as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageAgreement {
        PageAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            port: self.port,
            burst: self.burst,
            active: self.active,
            title: self.title.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a page agreement in status planned.
pub fn new_page_agreement(code: &str, label: &str) -> PageAgreement {
    PageAgreement {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        port: 0,
        burst: 0,
        active: false,
        title: "".to_string(),
        starts_at: 0,
        approved: false,
        ends_at: 0,
        status: PageAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_page_agreement("code-53", "label-388");
        assert_eq!(item.status, PageAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_page_agreement("code-53", "label-388");
        if item.move_to(PageAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_page_agreement("code-53", "label-388");
        item.status = PageAgreementStatus::Cancelled;
        assert!(
            item.move_to(PageAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
