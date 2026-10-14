use super::*;

/// How screens and exports show a legacy isolation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationClaimView {
    pub code: String,
    pub label: String,
    pub category_text: String,
    pub region: String,
    pub revision: String,
    pub created_at_text: String,
    pub summary: String,
    pub limit_text: String,
    pub phone: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_legacy_isolation_claim_view(item: &LegacyIsolationClaim) -> LegacyIsolationClaimView {
    LegacyIsolationClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        category_text: item.category.clone(),
        region: item.region.clone(),
        revision: item.revision.to_string(),
        created_at_text: item.created_at.to_string(),
        summary: item.summary.clone(),
        limit_text: item.limit.to_string(),
        phone: item.phone.clone(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_isolation_claim_views(items: &[LegacyIsolationClaim]) -> Vec<LegacyIsolationClaimView> {
    let mut views: Vec<LegacyIsolationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_isolation_claim_view(item));
    }
    views
}

pub fn sum_legacy_isolation_claim_view(view: &LegacyIsolationClaimView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_created_at as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_isolation_claim("code-352", "label-552");
        let view = to_legacy_isolation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_isolation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
