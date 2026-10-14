use super::*;

use acme_kit::text;

/// Where a page estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PageEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PageEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageEstimate {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub quantity: i64,
    pub owner_id: String,
    pub audience: String,
    pub ratio: f64,
    pub external_ref: String,
    pub phone: String,
    pub timezone: String,
    pub score: f64,
    pub status: PageEstimateStatus,
}

impl PageEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageEstimateStatus) -> bool {
        match self.status {
            PageEstimateStatus::Created => {
                return next == PageEstimateStatus::Validated || next == PageEstimateStatus::Retired;
            }
            PageEstimateStatus::Validated => {
                return next == PageEstimateStatus::Published || next == PageEstimateStatus::Retired;
            }
            PageEstimateStatus::Published => {
                match next {
                    PageEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: PageEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quantity as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageEstimate {
        PageEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            quantity: self.quantity,
            owner_id: self.owner_id.clone(),
            audience: self.audience.clone(),
            ratio: self.ratio,
            external_ref: self.external_ref.clone(),
            phone: self.phone.clone(),
            timezone: self.timezone.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a page estimate in status created.
pub fn new_page_estimate(code: &str, title: &str) -> PageEstimate {
    PageEstimate {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        quantity: 0,
        owner_id: "".to_string(),
        audience: "".to_string(),
        ratio: 0.0,
        external_ref: "".to_string(),
        phone: "".to_string(),
        timezone: "".to_string(),
        score: 0.0,
        status: PageEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_page_estimate("code-205", "title-83");
        assert_eq!(item.status, PageEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_page_estimate("code-205", "title-83");
        if item.move_to(PageEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_page_estimate("code-205", "title-83");
        item.status = PageEstimateStatus::Retired;
        assert!(
            item.move_to(PageEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
