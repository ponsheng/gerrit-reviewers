// TODO Store queried reviewers under local directories

use crate::gerrit_if;
use crate::gerrit_if::GerritUser;

pub struct GerritUserGroup {
    users: Vec<GerritUser>,
    name: String,
}

pub fn get_groups() -> Vec<GerritUserGroup> {
    let g1 = GerritUserGroup {
        users: vec![GerritUser::from_str("Fanya")],
        name: "Group 1".to_string(),
    };

    vec![g1]
}

