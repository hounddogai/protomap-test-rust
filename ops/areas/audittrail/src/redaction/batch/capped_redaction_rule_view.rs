use super::*;

use acme_kit::text;

/// How screens and exports show a capped redaction rule.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRuleView {
    pub code_text: String,
    pub title: String,
    pub maximum: String,
    pub updated_at: String,
    pub fingerprint: String,
    pub phone: String,
    pub checksum_text: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_capped_redaction_rule_view(item: &CappedRedactionRule) -> CappedRedactionRuleView {
    CappedRedactionRuleView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        maximum: format!("{:.2}", item.maximum),
        updated_at: item.updated_at.to_string(),
        fingerprint: item.fingerprint.clone(),
        phone: text::truncate(&item.phone, 80),
        checksum_text: item.checksum.clone(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_redaction_rule_views(items: &[CappedRedactionRule]) -> Vec<CappedRedactionRuleView> {
    let mut views: Vec<CappedRedactionRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_redaction_rule_view(item));
    }
    views
}

pub fn sum_capped_redaction_rule_view(view: &CappedRedactionRuleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_maximum + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_redaction_rule("code-20", "title-671");
        let view = to_capped_redaction_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_redaction_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
