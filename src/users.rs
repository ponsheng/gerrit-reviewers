use std::fmt;

//use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use regex::Regex;
use linked_hash_set::LinkedHashSet;
 
#[derive(Eq, Clone)]
pub struct GerritUser {
    pub email: String,
    pub full_name: String,
    pub username: String,
}

pub type UsersTy = LinkedHashSet<GerritUser>;

impl GerritUser {
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        if self.full_name.len() > 0 {
            s.push_str(&self.full_name);
        }
        if self.username.len() > 0 {
            if s.len() == 0 {
                s.push_str(&self.username);
            } else {
                s.push_str(&format!("({})", self.username));
            }
        }
        if self.email.len() > 0 {
            s.push_str(&format!(" [{}]", self.email));
        }
        s
    }

    pub fn from_str(set_name: &str) -> GerritUser {
        GerritUser {
            username: set_name.to_string(),
            email: String::new(),
            full_name: String::new(),
        }
    }
    pub fn from_string(set_name: String) -> GerritUser {
        GerritUser {
            username: set_name,
            email: String::new(),
            full_name: String::new(),
        }
    }
}
impl fmt::Display for GerritUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl Hash for GerritUser {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.username.hash(state);
    }
}
impl PartialEq for GerritUser {
    fn eq(&self, other: &Self) -> bool {
        self.username.eq(&other.username)
    }
}

pub fn is_valid_username(name: &str) -> bool {
    // Check for non ascii
    let non_ascii_matcher = Regex::new(r"[[:^ascii:]]").unwrap();
    if non_ascii_matcher.find(name).is_some() {
        return false;
    }
    // Check for space?

    return true;
}

pub fn get_git_user(json_val: &serde_json::Value) -> GerritUser {
    let user = GerritUser {
        username: json_val["username"].as_str().unwrap_or_default().to_string(),
        email: json_val["email"].as_str().unwrap_or_default().to_string(),
        full_name: json_val["name"].as_str().unwrap_or_default().to_string(),
    };
    return user;
}
