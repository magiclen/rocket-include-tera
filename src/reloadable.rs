use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::Tera;
use crate::tera::Error as TeraError;

#[derive(Debug)]
/// Reloadable Tera.
pub struct ReloadableTera {
    tera: Tera,
    files: HashMap<&'static str, (PathBuf, Option<SystemTime>)>,
}

impl ReloadableTera {
    #[inline]
    /// Create an instance of `ReloadableTera`.
    pub fn new() -> ReloadableTera {
        ReloadableTera {
            tera: Tera::default(),
            files: HashMap::new(),
        }
    }

    #[inline]
    /// Register a template from a path and it can be reloaded automatically.
    pub fn register_template_file<P: Into<PathBuf>>(&mut self, name: &'static str, file_path: P) -> Result<(), TeraError> {
        let file_path = file_path.into();

        self.tera.add_template_file(&file_path, Some(name))?;

        let metadata = file_path.metadata().unwrap();

        let mtime = metadata.modified().ok();

        self.files.insert(name, (file_path, mtime));

        Ok(())
    }

    #[inline]
    /// Unregister a template from a file by a name.
    pub fn unregister_template_file<S: AsRef<str>>(&mut self, name: S) -> Option<PathBuf> {
        let name = name.as_ref();

        match self.files.remove(name) {
            Some((file_path, _)) => {
                // TODO Remove template

                Some(file_path)
            }
            None => {
                None
            }
        }
    }

    #[inline]
    /// Reload templates if needed.
    pub fn reload_if_needed(&mut self) -> Result<(), TeraError> {
        for (name, (file_path, mtime)) in &mut self.files {
            let metadata = file_path.metadata().map_err(|err| TeraError::msg(err.to_string()))?;

            let (reload, new_mtime) = match mtime {
                Some(mtime) => {
                    match metadata.modified() {
                        Ok(new_mtime) => {
                            (new_mtime > *mtime, Some(new_mtime))
                        }
                        Err(_) => {
                            (true, None)
                        }
                    }
                }
                None => {
                    match metadata.modified() {
                        Ok(new_mtime) => {
                            (true, Some(new_mtime))
                        }
                        Err(_) => {
                            (true, None)
                        }
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