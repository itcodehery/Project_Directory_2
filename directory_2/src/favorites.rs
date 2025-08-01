use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Favorite {
    alias_name: String,
    path: PathBuf,
}

impl Favorite {
    pub fn get_alias_name(&self) -> &str {
        &self.alias_name
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn from(path: PathBuf) -> Favorite {
        let alias_name = path
            .file_name()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        Favorite { alias_name, path }
    }
}

pub struct FavoritesManager {
    favorites: Vec<Favorite>,
    file_path: PathBuf,
}

impl FavoritesManager {
    pub fn new() -> Result<Self, String> {
        let file_path = PathBuf::from(
            "E:/D/Coding/Repositories/Project_Directory_2/directory_2/.directory_2/favorites.json",
        );

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Could not create folder: {}", e))?;
        }
        let mut manager = FavoritesManager {
            favorites: Vec::new(),
            file_path,
        };

        manager.load()?;
        Ok(manager)
    }

    fn load(&mut self) -> Result<(), String> {
        if !self.file_path.exists() {
            // File doesn't exist yet, start with an empty list
            self.favorites = Vec::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read favorites file: {}", e))?;

        self.favorites = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse favorites JSON: {}", e))?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.favorites)
            .map_err(|e| format!("Failed to serialize favorites: {}", e))?;

        // Ensure the parent directory exists before writing
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }

        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write favorites file: {}", e))?;

        Ok(())
    }

    pub fn add(&mut self, favorite: Favorite) -> Result<(), String> {
        // Check if alias already exists
        if self
            .favorites
            .iter()
            .any(|f| f.alias_name == favorite.alias_name)
        {
            return Err(format!(
                "Favorite with alias '{}' already exists",
                favorite.alias_name
            ));
        }

        self.favorites.push(favorite);
        self.save()?;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<(), String> {
        let initial_len = self.favorites.len();

        if index > initial_len || index > 10 {
            return Err(format!("Index out of range: {}", index));
        }
        self.favorites.remove(index);

        if self.favorites.len() == initial_len {
            return Err(format!(
                "Favorite with index '{}' not found",
                index.to_string()
            ));
        }

        self.save()?;
        Ok(())
    }

    pub fn get_all(&self) -> &Vec<Favorite> {
        &self.favorites
    }

    // pub fn get_by_alias(&self, alias_name: &str) -> Option<&Favorite> {
    //     self.favorites.iter().find(|f| f.alias_name == alias_name)
    // }

    pub fn get_by_index(&self, index: usize) -> Option<&Favorite> {
        self.favorites.get(index)
    }

    pub fn len(&self) -> usize {
        self.favorites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.favorites.is_empty()
    }
}

// impl Favorite {
// pub fn from_json(json: &str) -> Favorite {
//     let json: serde_json::Value = serde_json::from_str(json).expect("FAV: Couldn't parse JSON");
//     Favorite {
//         alias_name: json["alias_name"].as_str().unwrap().to_string(),
//         path: PathBuf::from(json["path"].as_str().expect("FAV: Invalid path")),
//     }
// }

// pub fn to_json(&self) -> String {
//     serde_json::to_string(self).unwrap()
// }
// }
