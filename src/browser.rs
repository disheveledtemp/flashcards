use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub enum Entry {
    Dir(PathBuf),
    File(PathBuf),
}

pub struct Browser {
    cwd: PathBuf,
    entries: Vec<Entry>,
    pub selected: usize,
    message: Option<String>,
}

impl Browser {
    pub fn new(cwd: PathBuf) -> Result<Self> {
        let mut browser = Self {
            cwd,
            entries: Vec::new(),
            selected: 0,
            message: None,
        };

        browser.reload()?;

        Ok(browser)
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    // Open the selected entry; returns the deck path when a file is chosen.
    pub fn open(&mut self) -> Result<Option<PathBuf>> {
        match self.entries.get(self.selected) {
            Some(Entry::Dir(path)) => {
                self.cwd = path.clone();
                self.reload()?;
                Ok(None)
            }
            Some(Entry::File(path)) => Ok(Some(path.clone())),
            None => Ok(None),
        }
    }

    pub fn go_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.reload()?;
        }

        Ok(())
    }

    fn reload(&mut self) -> Result<()> {
        self.message = None;

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        let read = fs::read_dir(&self.cwd)
            .with_context(|| format!("Could not read {}", self.cwd.display()))?;

        for entry in read {
            let path = entry?.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();

            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }

        dirs.sort();
        files.sort();

        self.entries = dirs
            .into_iter()
            .map(Entry::Dir)
            .chain(files.into_iter().map(Entry::File))
            .collect();
        self.selected = 0;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "flashcards-browser-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn names(browser: &Browser) -> Vec<String> {
        browser
            .entries()
            .iter()
            .map(|entry| match entry {
                Entry::Dir(path) | Entry::File(path) => {
                    path.file_name().unwrap().to_str().unwrap().to_string()
                }
            })
            .collect()
    }

    #[test]
    fn lists_dirs_before_markdown_files() {
        let dir = temp_dir("listing");
        fs::create_dir(dir.join("b-dir")).unwrap();
        fs::create_dir(dir.join("a-dir")).unwrap();
        fs::write(dir.join("z.md"), "# Q\nA").unwrap();
        fs::write(dir.join("a.md"), "# Q\nA").unwrap();
        fs::write(dir.join("notes.txt"), "ignored").unwrap();
        fs::write(dir.join(".hidden.md"), "# Q\nA").unwrap();

        let browser = Browser::new(dir).unwrap();

        assert_eq!(names(&browser), ["a-dir", "b-dir", "a.md", "z.md"]);
    }

    #[test]
    fn message_clears_after_successful_navigation() {
        let dir = temp_dir("message");
        fs::create_dir(dir.join("sub")).unwrap();

        let mut browser = Browser::new(dir).unwrap();
        browser.set_message("boom".to_string());
        assert_eq!(browser.message(), Some("boom"));

        browser.open().unwrap();
        assert_eq!(browser.message(), None);
    }

    #[test]
    fn opens_directories_and_selects_files() {
        let dir = temp_dir("navigate");
        fs::create_dir(dir.join("decks")).unwrap();
        fs::write(dir.join("decks").join("bio.md"), "# Q\nA").unwrap();

        let mut browser = Browser::new(dir.clone()).unwrap();

        assert!(browser.open().unwrap().is_none(), "enters the directory");
        assert_eq!(browser.cwd(), dir.join("decks"));

        let chosen = browser.open().unwrap().expect("selects the markdown file");
        assert_eq!(chosen, dir.join("decks").join("bio.md"));

        browser.go_parent().unwrap();
        assert_eq!(browser.cwd(), dir);
    }
}
