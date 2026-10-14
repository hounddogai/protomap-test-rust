use super::*;

use acme_kit::text;

/// Where a weighted page plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPagePlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedPagePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for WeightedPagePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted page plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlan {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub principal: String,
    pub namespace: String,
    pub external_ref: String,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub status: WeightedPagePlanStatus,
}

impl WeightedPagePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPagePlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPagePlanStatus) -> bool {
        match self.status {
            WeightedPagePlanStatus::Open => {
                match next {
                    WeightedPagePlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPagePlanStatus::InReview => {
                return next == WeightedPagePlanStatus::Reopened;
            }
            WeightedPagePlanStatus::Resolved => {
                match next {
                    WeightedPagePlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPagePlanStatus::Reopened => {
                return next == WeightedPagePlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedPagePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted page plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> WeightedPagePlan {
        WeightedPagePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            external_ref: self.external_ref.clone(),
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted page plan in status open.
pub fn new_weighted_page_plan(code: &str, title: &str) -> WeightedPagePlan {
    WeightedPagePlan {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        principal: "".to_string(),
        namespace: "".to_string(),
        external_ref: "".to_string(),
        ttl_seconds: 0,
        cipher: "".to_string(),
        status: WeightedPagePlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_page_plan("code-814", "title-654");
        assert_eq!(item.status, WeightedPagePlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_weighted_page_plan("code-814", "title-654");
        if item.move_to(WeightedPagePlanStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPagePlanStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_page_plan("code-814", "title-654");
        item.status = WeightedPagePlanStatus::Archived;
        assert!(
            item.move_to(WeightedPagePlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
