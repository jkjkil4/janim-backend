mod attr_names;
mod compute;
mod exception;
mod relation;
mod utils;

use pyo3::prelude::*;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

#[pymodule]
mod janim_backend {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::compute::compute;
    #[pymodule_export]
    use super::exception::exception;
    #[pymodule_export]
    use super::relation::relation;

    #[pyfunction]
    pub fn set_locale(locale: &str) {
        rust_i18n::set_locale(locale);
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let patcher = crate::SubModulePatcher::new(m, "janim_backend")?;
        patcher.patch("compute")?;
        patcher.patch("exception")?;
        patcher.patch("relation")?;
        Ok(())
    }
}

struct SubModulePatcher<'a> {
    module: &'a Bound<'a, PyModule>,
    sys_modules: Bound<'a, PyAny>,
    module_path: &'static str,
}

impl<'a> SubModulePatcher<'a> {
    pub fn new(m: &'a Bound<'a, PyModule>, module_path: &'static str) -> PyResult<Self> {
        let sys_modules = m.py().import("sys")?.getattr("modules")?;
        Ok(Self {
            module: m,
            sys_modules,
            module_path,
        })
    }
}

impl SubModulePatcher<'_> {
    pub fn patch(&self, submodule_name: &'static str) -> PyResult<()> {
        let submodule_path = format!("{}.{}", self.module_path, submodule_name);
        let submodule = self.module.getattr(submodule_name)?;
        submodule.setattr("__name__", submodule_path.clone())?;
        self.sys_modules.set_item(submodule_path, submodule)
    }
}
