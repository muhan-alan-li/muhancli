use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Education {
    pub school: String,
    pub degree: String,
    pub location: String,
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub tagline: String,
    pub location: String,
    pub phone: String,
    pub email: String,
    pub links: Vec<Link>,
    pub education: Vec<Education>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Experience {
    pub id: String,
    pub company: String,
    pub role: String,
    pub location: String,
    pub start: String,
    pub end: String,
    pub bullets: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub slug: String,
    pub name: String,
    pub period: String,
    pub summary: String,
    pub stack: Vec<String>,
    pub highlights: Vec<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillGroup {
    pub group: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Portfolio {
    pub profile: Profile,
    pub experience: Vec<Experience>,
    pub projects: Vec<Project>,
    pub skills: Vec<SkillGroup>,
}

const PROFILE_JSON: &str = include_str!("data/profile.json");
const EXPERIENCE_JSON: &str = include_str!("data/experience.json");
const PROJECTS_JSON: &str = include_str!("data/projects.json");
const SKILLS_JSON: &str = include_str!("data/skills.json");

impl Portfolio {
    pub fn load() -> Self {
        let profile: Profile = serde_json::from_str(PROFILE_JSON).expect("profile.json must parse");
        let experience: Vec<Experience> =
            serde_json::from_str(EXPERIENCE_JSON).expect("experience.json must parse");
        let projects: Vec<Project> =
            serde_json::from_str(PROJECTS_JSON).expect("projects.json must parse");
        let skills: Vec<SkillGroup> =
            serde_json::from_str(SKILLS_JSON).expect("skills.json must parse");
        Self {
            profile,
            experience,
            projects,
            skills,
        }
    }

    pub fn find_experience(&self, id: &str) -> Option<&Experience> {
        self.experience
            .iter()
            .find(|e| e.id.eq_ignore_ascii_case(id))
    }

    pub fn find_project(&self, slug: &str) -> Option<&Project> {
        self.projects
            .iter()
            .find(|p| p.slug.eq_ignore_ascii_case(slug))
    }

    pub fn projects_by_tag(&self, tag: &str) -> Vec<&Project> {
        let needle = tag.to_lowercase();
        self.projects
            .iter()
            .filter(|p| {
                p.stack.iter().any(|s| s.to_lowercase().contains(&needle))
                    || p.summary.to_lowercase().contains(&needle)
                    || p.name.to_lowercase().contains(&needle)
            })
            .collect()
    }

    pub fn skill_group(&self, group: &str) -> Option<&SkillGroup> {
        self.skills
            .iter()
            .find(|g| g.group.eq_ignore_ascii_case(group))
    }
}
