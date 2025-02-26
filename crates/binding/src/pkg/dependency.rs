use napi::{
    bindgen_prelude::{FromNapiValue, JavaScriptClassExt, Object},
    sys, JsUnknown, Result,
};
use ws_pkg::package::Dependency as RepoDependency;

/// Dependency class.
/// Represents a package dependency.
///
/// @class Dependency - The Dependency class.
/// @property {string} name - The name of the dependency.
/// @property {string} version - The version of the dependency.
///
/// @example
///
/// ```typescript
/// const dep = new Dependency("foo", "1.0.0");
/// console.log(dep.name); // foo
/// console.log(dep.version); // 1.0.0
/// ```
#[napi(js_name = "Dependency")]
#[derive(Clone)]
pub struct Dependency {
    pub(crate) instance: RepoDependency,
}

#[napi]
impl Dependency {
    #[napi(constructor)]
    pub fn new(name: String, version: String) -> Self {
        //Self { instance: RepoDependency { name, version: version.parse().unwrap() } }
        Self { instance: RepoDependency { name, version: version.parse().unwrap() } }
    }

    /// Gets the name of the dependency.
    ///
    /// @returns {string} The name of the dependency.
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.instance.name.clone()
    }

    /// Gets the version of the dependency.
    ///
    /// @returns {string} The version of the dependency.
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.instance.version.to_string()
    }
}

impl FromNapiValue for Dependency {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        // Implement conversion from napi value to DependencyClass
        /*let obj = Object::from_napi_value(env, napi_val)?;
        let name: String = obj.get_named_property("name")?;
        let version: String = obj.get_named_property("version")?;
        Ok(Dependency {
            instance: Arc::new(RepoDependency { name, version: version.parse().unwrap() }),
        })*/
        unsafe {
            let unknown = JsUnknown::from_napi_value(env, napi_val)?;

            if !Dependency::instance_of(env.into(), &unknown)? {
                return Err(napi::Error::from_status(napi::Status::GenericFailure));
            }

            let object: Object = unknown.cast();
            let name: String = object.get_named_property_unchecked("name")?;
            let version: String = object.get_named_property_unchecked("version")?;
            Ok(Self { instance: RepoDependency { name, version: version.parse().unwrap() } })
        }
    }
}
