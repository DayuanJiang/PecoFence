use super::*;
use windows_core::implement_decl;

/// The scrollbar appearance WebView2 uses for pages, set with
/// [`EnvironmentOptions::scrollbar_style`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollBarStyle {
    /// The browser default scrollbars.
    #[default]
    Default,
    /// Thin overlay scrollbars in the Fluent style.
    FluentOverlay,
}

impl ScrollBarStyle {
    fn to_raw(self) -> COREWEBVIEW2_SCROLLBAR_STYLE {
        match self {
            Self::Default => 0,
            Self::FluentOverlay => 1,
        }
    }
}

/// Configures the WebView2 [`Environment`], including the user data folder, the
/// browser executable folder, additional browser command-line arguments, and the
/// UI language. Build one with the fluent setters and pass it to
/// [`Environment::with_options`].
///
/// ```
/// use windows_webview::EnvironmentOptions;
///
/// let options = EnvironmentOptions::new()
///     .user_data_folder(r"C:\MyApp\WebView2")
///     .additional_browser_arguments("--disable-features=msSmartScreenProtection")
///     .language("en-US");
/// ```
#[derive(Clone, Debug, Default)]
pub struct EnvironmentOptions {
    browser_executable_folder: Option<String>,
    user_data_folder: Option<String>,
    additional_browser_arguments: Option<String>,
    language: Option<String>,
    target_compatible_browser_version: Option<String>,
    allow_single_sign_on_using_os_primary_account: bool,
    are_browser_extensions_enabled: bool,
    scrollbar_style: ScrollBarStyle,
    custom_schemes: Vec<(String, Vec<String>)>,
}

impl EnvironmentOptions {
    /// Creates options with all WebView2 defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the folder containing the WebView2 browser (Edge) binaries to use
    /// instead of the installed runtime.
    pub fn browser_executable_folder(mut self, value: impl Into<String>) -> Self {
        self.browser_executable_folder = Some(value.into());
        self
    }

    /// Sets the folder where WebView2 stores its user data (cache, cookies, ...).
    pub fn user_data_folder(mut self, value: impl Into<String>) -> Self {
        self.user_data_folder = Some(value.into());
        self
    }

    /// Sets additional command-line arguments passed to the browser process.
    pub fn additional_browser_arguments(mut self, value: impl Into<String>) -> Self {
        self.additional_browser_arguments = Some(value.into());
        self
    }

    /// Sets the default display language (for example `"en-US"`).
    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }

    /// Sets the minimum compatible browser version the environment requires.
    ///
    /// When unset, the environment uses the WebView2 GA baseline
    /// (`86.0.616.0`), which every supported runtime satisfies.
    pub fn target_compatible_browser_version(mut self, value: impl Into<String>) -> Self {
        self.target_compatible_browser_version = Some(value.into());
        self
    }

    /// Enables single sign-on using the operating system's primary account.
    pub fn allow_single_sign_on_using_os_primary_account(mut self, value: bool) -> Self {
        self.allow_single_sign_on_using_os_primary_account = value;
        self
    }

    /// Enables loading and running browser extensions in the environment.
    pub fn are_browser_extensions_enabled(mut self, value: bool) -> Self {
        self.are_browser_extensions_enabled = value;
        self
    }

    /// Sets the scrollbar style pages render with.
    pub fn scrollbar_style(mut self, value: ScrollBarStyle) -> Self {
        self.scrollbar_style = value;
        self
    }

    /// Adds the custom scheme `name` (`name://host/path`): its pages are secure contexts with the
    /// origin `name://host`, its requests reach only [`WebView::on_web_resource_requested`] and
    /// never the network. Requests with an `Origin` header (POST, CORS) are allowed from
    /// `allowed_origins` (patterns with `*`, as the request filter takes).
    pub fn custom_scheme(mut self, name: impl Into<String>, allowed_origins: &[&str]) -> Self {
        let origins = allowed_origins.iter().map(|o| (*o).to_string()).collect();
        self.custom_schemes.push((name.into(), origins));
        self
    }

    pub(crate) fn create_environment<F: FnOnce(Result<Environment>) + 'static>(
        &self,
        handler: F,
    ) -> Result<()> {
        let handler = handler::EnvironmentCompleted::create(handler);
        let options: ICoreWebView2EnvironmentOptions = OptionsObject::new(self).into();

        let browser = self.browser_executable_folder.as_deref().map(HSTRING::from);
        let user_data = self.user_data_folder.as_deref().map(HSTRING::from);
        let browser = browser
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR::from_raw(value.as_ptr()));
        let user_data = user_data
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR::from_raw(value.as_ptr()));

        unsafe {
            CreateCoreWebView2EnvironmentWithOptions(
                browser,
                user_data,
                Interface::as_raw(&options),
                Interface::as_raw(&handler),
            )
            .ok()
        }
    }
}

/// Read-only COM options object backed by [`EnvironmentOptions`].
pub(crate) struct OptionsObject {
    additional_browser_arguments: String,
    language: String,
    target_compatible_browser_version: String,
    allow_single_sign_on_using_os_primary_account: BOOL,
    are_browser_extensions_enabled: BOOL,
    scrollbar_style: COREWEBVIEW2_SCROLLBAR_STYLE,
    custom_schemes: Vec<ICoreWebView2CustomSchemeRegistration>,
}

implement_decl! {
    impl OptionsObject as pub(crate) OptionsObject_Impl:
        [ICoreWebView2EnvironmentOptions, ICoreWebView2EnvironmentOptions4, ICoreWebView2EnvironmentOptions6, ICoreWebView2EnvironmentOptions8]
}

/// Read-only COM registration of one custom scheme ([`EnvironmentOptions::custom_scheme`]): a
/// secure standard scheme with a host.
struct SchemeObject {
    name: String,
    allowed_origins: Vec<String>,
}

implement_decl! {
    impl SchemeObject as SchemeObject_Impl: [ICoreWebView2CustomSchemeRegistration]
}

/// A `CoTaskMemAlloc` array of `items` for WebView2, which frees it.
fn com_array<T>(items: impl ExactSizeIterator<Item = Result<*mut T>>) -> Result<(u32, *mut *mut T)> {
    let n = items.len();
    let list = unsafe { CoTaskMemAlloc(n.max(1) * size_of::<*mut T>()) } as *mut *mut T;
    if list.is_null() {
        return Err(Error::from_hresult(E_OUTOFMEMORY));
    }
    for (i, item) in items.enumerate() {
        unsafe { list.add(i).write(item?) };
    }
    Ok((n as u32, list))
}

impl ICoreWebView2CustomSchemeRegistration_Impl for SchemeObject_Impl {
    fn SchemeName(&self) -> Result<LPWSTR> {
        unsafe { string::allocate(&self.name) }
    }

    fn TreatAsSecure(&self) -> Result<BOOL> {
        Ok(true.into())
    }

    fn GetAllowedOrigins(&self) -> Result<(u32, *mut LPWSTR)> {
        com_array(self.allowed_origins.iter().map(|o| unsafe { string::allocate(o) }))
    }

    fn HasAuthorityComponent(&self) -> Result<BOOL> {
        Ok(true.into())
    }
}

impl ICoreWebView2EnvironmentOptions4_Impl for OptionsObject_Impl {
    fn GetCustomSchemeRegistrations(&self) -> Result<(u32, *mut *mut core::ffi::c_void)> {
        com_array(self.custom_schemes.iter().map(|s| Ok(s.clone().into_raw())))
    }
}

/// Default `TargetCompatibleBrowserVersion`; WebView2 rejects an empty value.
const DEFAULT_TARGET_COMPATIBLE_BROWSER_VERSION: &str = "86.0.616.0";

impl OptionsObject {
    fn new(options: &EnvironmentOptions) -> Self {
        Self {
            additional_browser_arguments: options
                .additional_browser_arguments
                .clone()
                .unwrap_or_default(),
            language: options.language.clone().unwrap_or_default(),
            target_compatible_browser_version: options
                .target_compatible_browser_version
                .clone()
                .unwrap_or_else(|| DEFAULT_TARGET_COMPATIBLE_BROWSER_VERSION.to_string()),
            allow_single_sign_on_using_os_primary_account: options
                .allow_single_sign_on_using_os_primary_account
                .into(),
            are_browser_extensions_enabled: options.are_browser_extensions_enabled.into(),
            scrollbar_style: options.scrollbar_style.to_raw(),
            custom_schemes: (options.custom_schemes.iter())
                .map(|(name, allowed_origins)| {
                    SchemeObject {
                        name: name.clone(),
                        allowed_origins: allowed_origins.clone(),
                    }
                    .into()
                })
                .collect(),
        }
    }
}

impl ICoreWebView2EnvironmentOptions_Impl for OptionsObject_Impl {
    fn AdditionalBrowserArguments(&self) -> Result<LPWSTR> {
        unsafe { string::allocate(&self.additional_browser_arguments) }
    }

    fn SetAdditionalBrowserArguments(&self, _value: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn Language(&self) -> Result<LPWSTR> {
        unsafe { string::allocate(&self.language) }
    }

    fn SetLanguage(&self, _value: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn TargetCompatibleBrowserVersion(&self) -> Result<LPWSTR> {
        unsafe { string::allocate(&self.target_compatible_browser_version) }
    }

    fn SetTargetCompatibleBrowserVersion(&self, _value: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn AllowSingleSignOnUsingOSPrimaryAccount(&self) -> Result<BOOL> {
        Ok(self.allow_single_sign_on_using_os_primary_account)
    }

    fn SetAllowSingleSignOnUsingOSPrimaryAccount(&self, _allow: BOOL) -> Result<()> {
        Ok(())
    }
}

impl ICoreWebView2EnvironmentOptions6_Impl for OptionsObject_Impl {
    fn AreBrowserExtensionsEnabled(&self) -> Result<BOOL> {
        Ok(self.are_browser_extensions_enabled)
    }

    fn SetAreBrowserExtensionsEnabled(&self, _value: BOOL) -> Result<()> {
        Ok(())
    }
}

impl ICoreWebView2EnvironmentOptions8_Impl for OptionsObject_Impl {
    fn ScrollBarStyle(&self) -> Result<COREWEBVIEW2_SCROLLBAR_STYLE> {
        Ok(self.scrollbar_style)
    }

    fn SetScrollBarStyle(&self, _value: COREWEBVIEW2_SCROLLBAR_STYLE) -> Result<()> {
        Ok(())
    }
}
