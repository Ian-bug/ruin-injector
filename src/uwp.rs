#[derive(Debug)]
pub enum UwpError {
    NotImplemented(String),
}

impl std::fmt::Display for UwpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UwpError::NotImplemented(msg) => write!(f, "UWP permissions: {}", msg),
        }
    }
}

impl std::error::Error for UwpError {}

pub fn set_uwp_permissions(_dll_path: &str) -> Result<(), UwpError> {
    Err(UwpError::NotImplemented(
        "UWP permission setting requires manual configuration. Please set 'All Applications Packages' permission on the DLL file manually.".to_string()
    ))
}
