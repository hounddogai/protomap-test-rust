use super::*;

use acme_kit::text;

/// Where a postmortem estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PostmortemEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PostmortemEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem estimate as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEstimate {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub locale: String,
    pub weight: f64,
    pub cost: f64,
    pub external_ref: String,
    pub expires_at: i64,
    pub description: String,
    pub phone: String,
    pub status: PostmortemEstimateStatus,
}

impl PostmortemEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemEstimateStatus) -> bool {
        match self.status {
            PostmortemEstimateStatus::Created => {
                return next == PostmortemEstimateStatus::Validated || next == PostmortemEstimateStatus::Retired;
            }
            PostmortemEstimateStatus::Validated => {
                match next {
                    PostmortemEstimateStatus::Published | PostmortemEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemEstimateStatus::Published => {
                match next {
                    PostmortemEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: PostmortemEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.expires_at as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemEstimate {
        PostmortemEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            locale: self.locale.clone(),
            weight: self.weight,
            cost: self.cost,
            external_ref: self.external_ref.clone(),
            expires_at: self.expires_at,
            description: self.description.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a postmortem estimate in status created.
pub fn new_postmortem_estimate(code: &str, title: &str) -> PostmortemEstimate {
    PostmortemEstimate {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        locale: "".to_string(),
        weight: 0.0,
        cost: 0.0,
        external_ref: "".to_string(),
        expires_at: 0,
        description: "".to_string(),
        phone: "".to_string(),
        status: PostmortemEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_postmortem_estimate("code-892", "title-775");
        assert_eq!(item.status, PostmortemEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_postmortem_estimate("code-892", "title-775");
        if item.move_to(PostmortemEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_postmortem_estimate("code-892", "title-775");
        item.status = PostmortemEstimateStatus::Retired;
        assert!(
            item.move_to(PostmortemEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
