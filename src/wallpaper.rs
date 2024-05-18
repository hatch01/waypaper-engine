use std::error::Error;
use std::fs::{DirEntry, File};
use crate::project::{WEProject, WallpaperType};

#[derive(Debug, Clone)]
pub enum Wallpaper {
    Video { project: WEProject },
    Scene { project: WEProject },
    Web { project: WEProject },
    Preset { project: WEProject },
}

impl Wallpaper {
    pub fn from(entry: &DirEntry) -> Result<Wallpaper, Box<dyn Error>> {
        let project_file = File::open(entry.path().join("project.json"))?;
        let project: WEProject = serde_json::from_reader(project_file)?;

        Ok(match project.wallpaper_type {
            WallpaperType::Video => Wallpaper::Video { project },
            WallpaperType::Scene => Wallpaper::Scene { project },
            WallpaperType::Web => Wallpaper::Web { project },
            WallpaperType::Preset => Wallpaper::Preset { project }
        })
    }
}