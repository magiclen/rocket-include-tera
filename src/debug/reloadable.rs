use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::SystemTime;

use tera::{Error as TeraError, Tera};

#[derive(Debug)]
/// Reloadable Tera.
pub struct ReloadableTera {
    tera: Tera,
    files: HashMap<&'static str, (PathBuf, Option<SystemTime>)>,
}

impl ReloadableTera {
    /// Create an instance of `ReloadableTera`.
    #[inline]
    pub fn new() -> ReloadableTera {
        let tera = Tera::default();

        ReloadableTera {
            tera,
            files: HashMap::new(),
        }
    }

    /// Register a template from a path and it can be reloaded automatically.
    #[inline]
    pub fn register_template_file<P: Into<PathBuf>>(
        &mut self,
        name: &'static str,
        file_path: P,
    ) -> Result<(), TeraError> {
        let file_path = file_path.into();

        let metadata = file_path.metadata()?;

        let mtime = metadata.modified().ok();

        self.tera.add_template_file(&file_path, Some(name))?;

        self.files.insert(name, (file_path, mtime));

        Ok(())
    }

    /// Unregister a template from a file by a name.
    #[inline]
    pub fn unregister_template_file<S: AsRef<str>>(&mut self, name: S) -> Option<PathBuf> {
        let name = name.as_ref();

        self.files.remove(name).map(|(file_path, _)| {
            // TODO Remove template
            file_path
        })
    }

    /// Reload templates if needed.
    #[inline]
    pub fn reload_if_needed(&mut self) -> Result<(), TeraError> {
        for (name, (file_path, mtime)) in &mut self.files {
            let metadata = file_path.metadata()?;

            let (reload, new_mtime) = match mtime {
                Some(mtime) => {
                    match metadata.modified() {
                        Ok(new_mtime) => (new_mtime > *mtime, Some(new_mtime)),
                        Err(_) => (true, None),
                    }
                }
                None => {
                    match metadata.modified() {
                        Ok(new_mtime) => (true, Some(new_mtime)),
                        Err(_) => (true, None),
                    }
                }
            };

            if reload {
                self.tera.add_template_file(&file_path, Some(name))?;

                *mtime = new_mtime;
            }
        }

        Ok(())
    }
}

impl Default for ReloadableTera {
    #[inline]
    fn default() -> Self {
        ReloadableTera::new()
    }
}

impl Deref for ReloadableTera {
    type Target = Tera;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.tera
    }
}

impl DerefMut for ReloadableTera {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tera
    }
}
