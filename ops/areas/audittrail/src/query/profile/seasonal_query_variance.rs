use super::*;

use acme_kit::text;

/// Where a seasonal query variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalQueryVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalQueryVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalQueryVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal query variance as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVariance {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub description: String,
    pub cluster: String,
    pub cost: f64,
    pub name: String,
    pub visible: bool,
    pub score: f64,
    pub limit: i64,
    pub created_at: i64,
    pub status: SeasonalQueryVarianceStatus,
}

impl SeasonalQueryVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalQueryVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalQueryVarianceStatus) -> bool {
        match self.status {
            SeasonalQueryVarianceStatus::Requested => {
                match next {
                    SeasonalQueryVarianceStatus::Granted | SeasonalQueryVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQueryVarianceStatus::Granted => {
                match next {
                    SeasonalQueryVarianceStatus::Revoked | SeasonalQueryVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQueryVarianceStatus::Revoked => {
                return next == SeasonalQueryVarianceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalQueryVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal query variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.version as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalQueryVariance {
        SeasonalQueryVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            description: self.description.clone(),
            cluster: self.cluster.clone(),
            cost: self.cost,
            name: self.name.clone(),
            visible: self.visible,
            score: self.score,
            limit: self.limit,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal query variance in status requested.
pub fn new_seasonal_query_variance(code: &str, title: &str) -> SeasonalQueryVariance {
    SeasonalQueryVariance {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        description: "".to_string(),
        cluster: "".to_string(),
        cost: 0.0,
        name: "".to_string(),
        visible: false,
        score: 0.0,
        limit: 0,
        created_at: 0,
        status: SeasonalQueryVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_query_variance("code-462", "title-211");
        assert_eq!(item.status, SeasonalQueryVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_query_variance("code-462", "title-211");
        if item.move_to(SeasonalQueryVarianceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalQueryVarianceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_query_variance("code-462", "title-211");
        item.status = SeasonalQueryVarianceStatus::Expired;
        assert!(
            item.move_to(SeasonalQueryVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
