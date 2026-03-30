use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalTag {
    pub id: String,
    pub label: String,
}

pub fn get_tag_registry() -> Vec<JournalTag> {
    vec![
        JournalTag { id: "Career".into(), label: "Career" .into() },
        JournalTag { id: "Coding".into(), label: "Coding" .into() },
        JournalTag { id: "Wellness".into(), label: "Wellness" .into() },
        JournalTag { id: "Personal".into(), label: "Personal" .into() },
        JournalTag { id: "Growth".into(), label: "Growth" .into() },
        JournalTag { id: "Social".into(), label: "Social" .into() },
    ]
}