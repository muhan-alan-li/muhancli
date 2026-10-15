use muhancli::Portfolio;

#[test]
fn portfolio_loads() {
    let p = Portfolio::load();
    assert!(!p.profile.name.is_empty());
    assert!(!p.experience.is_empty());
    assert!(!p.projects.is_empty());
    assert!(!p.skills.is_empty());
}

#[test]
fn ids_and_slugs_unique() {
    let p = Portfolio::load();
    let mut ids = std::collections::HashSet::new();
    for e in &p.experience {
        assert!(!e.id.is_empty());
        assert!(ids.insert(e.id.to_lowercase()), "duplicate id {}", e.id);
        assert!(!e.bullets.is_empty(), "empty bullets {}", e.id);
    }
    let mut slugs = std::collections::HashSet::new();
    for pr in &p.projects {
        assert!(!pr.slug.is_empty());
        assert!(
            slugs.insert(pr.slug.to_lowercase()),
            "duplicate slug {}",
            pr.slug
        );
        assert!(!pr.highlights.is_empty(), "empty highlights {}", pr.slug);
    }
}

#[test]
fn links_are_urls() {
    let p = Portfolio::load();
    for l in &p.profile.links {
        assert!(
            l.url.starts_with("https://"),
            "link {} must be https url",
            l.label
        );
    }
}

#[test]
fn lookup_case_insensitive() {
    let p = Portfolio::load();
    assert!(p.find_experience("APERA-AI").is_some());
    assert!(p.find_project("DUNE-IMPERIUM").is_some());
    assert!(p.skill_group("languages").is_some());
}
