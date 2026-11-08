use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::SystemTime,
};

use tera::{Error as TeraError, Tera};

#[derive(Debug)]
/// Reloadable Tera.
pub struct ReloadableTera {
    tera:  Tera,
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
        self.register_template_files([(name, file_path)])
    }

    #[doc(hidden)]
    pub fn register_template_files<P: Into<PathBuf>>(
        &mut self,
        files: impl IntoIterator<Item = (&'static str, P)>,
    ) -> Result<(), TeraError> {
        let mut registered_files = Vec::new();

        for (name, path) in files {
            let path = path.into();
            let mtime = path.metadata()?.modified().ok();

            registered_files.push((name, (path, mtime)));
        }

        self.tera.add_template_files(
            registered_files.iter().map(|(name, (path, _))| (path, Some(*name))),
        )?;

        self.files.extend(registered_files);

        Ok(())
    }

    /// Unregister a template from a file by a name.
    ///
    /// Tera provides no API for removing a single template, so the template itself stays inside Tera and only stops being reloaded.
    #[inline]
    pub fn unregister_template_file<S: AsRef<str>>(&mut self, name: S) -> Option<PathBuf> {
        let name = name.as_ref();

        self.files.remove(name).map(|(file_path, _)| file_path)
    }

    /// Reload templates if needed.
    ///
    /// A template file that cannot be read does not stop the others from being reloaded, and the first error is returned.
    /// Changed templates are reloaded together, so if one of them is invalid, none of them is reloaded.
    #[inline]
    pub fn reload_if_needed(&mut self) -> Result<(), TeraError> {
        let mut changed_files = Vec::new();
        let mut first_error = None;

        for (&name, (file_path, mtime)) in &mut self.files {
            let new_mtime = match file_path.metadata() {
                Ok(metadata) => metadata.modified().ok(),
                Err(err) => {
                    first_error.get_or_insert(TeraError::from(err));

                    continue;
                },
            };

            if new_mtime.is_none() || new_mtime != *mtime {
                changed_files.push((name, &*file_path, mtime, new_mtime));
            }
        }

        if !changed_files.is_empty() {
            self.tera.add_template_files(
                changed_files.iter().map(|&(name, path, ..)| (path, Some(name))),
            )?;

            for (_, _, mtime, new_mtime) in changed_files {
                *mtime = new_mtime;
            }
        }

        first_error.map_or(Ok(()), Err)
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
