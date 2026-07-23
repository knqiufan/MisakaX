use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Window, WindowEvent,
};

use crate::{
    config::{AppConfig, CloseBehavior},
    AppState,
};

pub const CLOSE_REQUESTED_EVENT: &str = "tray:close-requested";
pub const NEW_TASK_EVENT: &str = "tray:new-task";
pub const OPEN_SETTINGS_EVENT: &str = "tray:open-settings";
pub const OPERATION_ERROR_EVENT: &str = "tray:operation-error";

const TRAY_ID: &str = "misakax-tray";
const MENU_OPEN: &str = "tray-open";
const MENU_NEW_TASK: &str = "tray-new-task";
const MENU_OPEN_PROJECT: &str = "tray-open-project";
const MENU_SETTINGS: &str = "tray-settings";
const MENU_QUIT: &str = "tray-quit";

/// Retains native tray resources and the project context reflected in the menu.
#[derive(Default)]
pub struct TrayState {
    inner: Mutex<TrayStateInner>,
}

#[derive(Default)]
struct TrayStateInner {
    controls: Option<TrayControls>,
    context: Option<ProjectContext>,
}

struct TrayControls {
    // The icon is removed when its final handle is dropped, so retain it for
    // the complete application lifetime.
    _tray: TrayIcon,
    open: MenuItem<tauri::Wry>,
    project: MenuItem<tauri::Wry>,
    new_task: MenuItem<tauri::Wry>,
    open_project: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectContext {
    directory: PathBuf,
    project_name: Option<String>,
    is_default_workspace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrayMenuState {
    open: String,
    project: String,
    new_task: String,
    open_project: String,
    settings: String,
    quit: String,
    has_project: bool,
}

#[derive(Debug, Clone, Copy)]
struct TrayLabels {
    open: &'static str,
    current_project: &'static str,
    no_active_project: &'static str,
    new_task: &'static str,
    new_task_in_project: &'static str,
    open_project: &'static str,
    settings: &'static str,
    quit: &'static str,
    default_workspace: &'static str,
}

impl TrayLabels {
    fn for_language(language: &str) -> Self {
        if language.to_ascii_lowercase().starts_with("zh") {
            Self {
                open: "打开 MisakaX",
                current_project: "当前项目：",
                no_active_project: "当前没有活动项目",
                new_task: "新建任务",
                new_task_in_project: "在当前项目中新建任务",
                open_project: "在文件资源管理器中打开当前项目",
                settings: "设置",
                quit: "退出 MisakaX",
                default_workspace: "默认工作目录",
            }
        } else {
            Self {
                open: "Open MisakaX",
                current_project: "Current project: ",
                no_active_project: "No active project",
                new_task: "New task",
                new_task_in_project: "New task in current project",
                open_project: "Open current project in File Explorer",
                settings: "Settings",
                quit: "Quit MisakaX",
                default_workspace: "Default workspace",
            }
        }
    }
}

impl ProjectContext {
    fn from_input(
        working_directory: Option<String>,
        project_name: Option<String>,
        workspace_kind: Option<String>,
    ) -> Option<Self> {
        let working_directory = working_directory?.trim().to_string();
        if working_directory.is_empty() {
            return None;
        }

        let path = PathBuf::from(working_directory);
        if !path.is_dir() {
            return None;
        }

        let directory = std::fs::canonicalize(&path).unwrap_or(path);
        let project_name = project_name.filter(|name| !name.trim().is_empty());
        Some(Self {
            directory,
            project_name,
            is_default_workspace: workspace_kind.as_deref() == Some("default"),
        })
    }

    fn display_name(&self, labels: TrayLabels) -> String {
        if self.is_default_workspace {
            return labels.default_workspace.to_string();
        }
        if let Some(project_name) = &self.project_name {
            return project_name.clone();
        }
        self.directory
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.directory.display().to_string())
    }
}

fn menu_state(context: Option<&ProjectContext>, language: &str) -> TrayMenuState {
    let labels = TrayLabels::for_language(language);
    let project = context.map(|context| context.display_name(labels));
    TrayMenuState {
        open: labels.open.to_string(),
        project: project
            .as_ref()
            .map(|name| format!("{}{}", labels.current_project, name.replace('&', "&&")))
            .unwrap_or_else(|| labels.no_active_project.to_string()),
        new_task: if project.is_some() {
            labels.new_task_in_project.to_string()
        } else {
            labels.new_task.to_string()
        },
        open_project: labels.open_project.to_string(),
        settings: labels.settings.to_string(),
        quit: labels.quit.to_string(),
        has_project: project.is_some(),
    }
}

/// Create the persistent system tray icon and its native menu.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let language = app
        .state::<AppState>()
        .config
        .lock()
        .map(|config| config.language.clone())
        .unwrap_or_else(|error| {
            tracing::warn!(error = %error, "Unable to read language for tray menu");
            "en".to_string()
        });
    let labels = TrayLabels::for_language(&language);

    let open = MenuItem::with_id(app, MENU_OPEN, labels.open, true, None::<&str>)?;
    let project = MenuItem::with_id(
        app,
        "tray-project",
        labels.no_active_project,
        false,
        None::<&str>,
    )?;
    let new_task = MenuItem::with_id(app, MENU_NEW_TASK, labels.new_task, true, None::<&str>)?;
    let open_project = MenuItem::with_id(
        app,
        MENU_OPEN_PROJECT,
        labels.open_project,
        false,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, MENU_SETTINGS, labels.settings, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, labels.quit, true, None::<&str>)?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let separator_three = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &separator_one,
            &project,
            &new_task,
            &open_project,
            &separator_two,
            &settings,
            &separator_three,
            &quit,
        ],
    )?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".to_string()))?;
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("MisakaX")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_icon_event)
        .build(app)?;

    let state = app.state::<TrayState>();
    let mut inner = state.inner.lock().map_err(|error| {
        tracing::error!(error = %error, "Unable to retain tray icon state");
        tauri::Error::AssetNotFound("tray state lock".to_string())
    })?;
    inner.controls = Some(TrayControls {
        _tray: tray,
        open,
        project,
        new_task,
        open_project,
        settings,
        quit,
    });
    apply_menu_state(&inner, &language);

    Ok(())
}

/// Synchronize the native menu with the active session's workspace.
pub fn update_context(
    app: &AppHandle,
    working_directory: Option<String>,
    project_name: Option<String>,
    workspace_kind: Option<String>,
    language: &str,
) -> Result<(), String> {
    let context = ProjectContext::from_input(working_directory, project_name, workspace_kind);
    let state = app.state::<TrayState>();
    let mut inner = state.inner.lock().map_err(|error| error.to_string())?;
    inner.context = context;
    apply_menu_state(&inner, language);
    Ok(())
}

/// Refresh native strings after a persisted language change.
pub fn refresh_labels(app: &AppHandle, config: &AppConfig) {
    let state = app.state::<TrayState>();
    let inner = match state.inner.lock() {
        Ok(inner) => inner,
        Err(error) => {
            tracing::warn!(error = %error, "Unable to refresh tray labels");
            return;
        }
    };
    apply_menu_state(&inner, &config.language);
}

/// Process primary window close requests without stopping the managed Sidecar
/// unless the user explicitly chooses to quit.
pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != "main" {
        return;
    }

    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };

    let app = window.app_handle();
    let close_behavior = app
        .state::<AppState>()
        .config
        .lock()
        .map(|config| config.close_behavior)
        .unwrap_or_else(|error| {
            tracing::warn!(error = %error, "Unable to read close behavior; asking user");
            CloseBehavior::Ask
        });

    match close_behavior {
        CloseBehavior::Ask => {
            api.prevent_close();
            if let Err(error) = app.emit(CLOSE_REQUESTED_EVENT, ()) {
                tracing::warn!(error = %error, "Unable to request close behavior from frontend");
            }
        }
        CloseBehavior::MinimizeToTray => {
            api.prevent_close();
            if let Err(error) = hide_main_window(app) {
                tracing::warn!(error = %error, "Unable to hide main window to system tray");
            }
        }
        CloseBehavior::Quit => {
            api.prevent_close();
            app.exit(0);
        }
    }
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Primary window is unavailable".to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Primary window is unavailable".to_string())?;
    window.hide().map_err(|error| error.to_string())
}

fn apply_menu_state(inner: &TrayStateInner, language: &str) {
    let Some(controls) = &inner.controls else {
        return;
    };
    let state = menu_state(inner.context.as_ref(), language);

    set_menu_text(&controls.open, &state.open);
    set_menu_text(&controls.project, &state.project);
    set_menu_text(&controls.new_task, &state.new_task);
    set_menu_text(&controls.open_project, &state.open_project);
    set_menu_text(&controls.settings, &state.settings);
    set_menu_text(&controls.quit, &state.quit);
    set_menu_enabled(&controls.project, false);
    set_menu_enabled(&controls.new_task, true);
    set_menu_enabled(&controls.open_project, state.has_project);
}

fn set_menu_text(item: &MenuItem<tauri::Wry>, text: &str) {
    if let Err(error) = item.set_text(text) {
        tracing::warn!(error = %error, "Unable to update tray menu label");
    }
}

fn set_menu_enabled(item: &MenuItem<tauri::Wry>, enabled: bool) {
    if let Err(error) = item.set_enabled(enabled) {
        tracing::warn!(error = %error, "Unable to update tray menu availability");
    }
}

fn handle_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    let should_show = matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        }
    );
    if should_show {
        if let Err(error) = show_main_window(tray.app_handle()) {
            tracing::warn!(error = %error, "Unable to restore main window from tray");
        }
    }
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        MENU_OPEN => show_or_log(app),
        MENU_NEW_TASK => {
            show_or_log(app);
            let working_directory =
                active_context_directory(app).map(|path| path.display().to_string());
            if let Err(error) = app.emit(NEW_TASK_EVENT, NewTaskPayload { working_directory }) {
                tracing::warn!(error = %error, "Unable to request a new task from tray menu");
            }
        }
        MENU_OPEN_PROJECT => {
            if let Err(error) = open_active_project(app) {
                tracing::warn!(error = %error, "Unable to open current project from tray menu");
                show_or_log(app);
                if let Err(emit_error) = app.emit(
                    OPERATION_ERROR_EVENT,
                    TrayOperationError {
                        operation: "open_project",
                        error,
                    },
                ) {
                    tracing::warn!(error = %emit_error, "Unable to report tray operation error");
                }
            }
        }
        MENU_SETTINGS => {
            show_or_log(app);
            if let Err(error) = app.emit(OPEN_SETTINGS_EVENT, ()) {
                tracing::warn!(error = %error, "Unable to request settings from tray menu");
            }
        }
        MENU_QUIT => app.exit(0),
        _ => {}
    }
}

fn show_or_log(app: &AppHandle) {
    if let Err(error) = show_main_window(app) {
        tracing::warn!(error = %error, "Unable to show main window");
    }
}

fn active_context_directory(app: &AppHandle) -> Option<PathBuf> {
    let state = app.state::<TrayState>();
    let inner = state.inner.lock().ok()?;
    inner
        .context
        .as_ref()
        .map(|context| context.directory.clone())
}

fn open_active_project(app: &AppHandle) -> Result<(), String> {
    let path = active_context_directory(app)
        .ok_or_else(|| "No active project directory is available".to_string())?;
    if !path.is_dir() {
        clear_context(app);
        return Err("The current project directory is no longer available".to_string());
    }
    open_project_directory(&path)
}

fn clear_context(app: &AppHandle) {
    let language = app
        .state::<AppState>()
        .config
        .lock()
        .map(|config| config.language.clone())
        .unwrap_or_else(|_| "en".to_string());
    let state = app.state::<TrayState>();
    if let Ok(mut inner) = state.inner.lock() {
        inner.context = None;
        apply_menu_state(&inner, &language);
    };
}

fn open_project_directory(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let command = "explorer";
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let command = "xdg-open";

    Command::new(command)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to open {}: {error}", path.display()))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NewTaskPayload {
    working_directory: Option<String>,
}

#[derive(Clone, Serialize)]
struct TrayOperationError {
    operation: &'static str,
    error: String,
}

#[cfg(test)]
mod tests {
    use super::{menu_state, ProjectContext};
    use std::path::PathBuf;

    #[test]
    fn menu_without_project_keeps_new_task_available_and_folder_disabled() {
        let state = menu_state(None, "en");
        assert_eq!(state.project, "No active project");
        assert_eq!(state.new_task, "New task");
        assert!(!state.has_project);
    }

    #[test]
    fn menu_with_project_uses_project_context_and_localized_labels() {
        let context = ProjectContext {
            directory: PathBuf::from("C:/work/alpha"),
            project_name: Some("Alpha".to_string()),
            is_default_workspace: false,
        };
        let state = menu_state(Some(&context), "zh-CN");
        assert_eq!(state.project, "当前项目：Alpha");
        assert_eq!(state.new_task, "在当前项目中新建任务");
        assert!(state.has_project);
    }

    #[test]
    fn default_workspace_uses_its_localized_name_instead_of_a_path_label() {
        let context = ProjectContext {
            directory: PathBuf::from("C:/Users/example/.misakax/workspace"),
            project_name: Some("workspace".to_string()),
            is_default_workspace: true,
        };
        let state = menu_state(Some(&context), "en");
        assert_eq!(state.project, "Current project: Default workspace");
    }
}
