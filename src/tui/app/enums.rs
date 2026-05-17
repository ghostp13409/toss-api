use crate::cli::args::Method;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
    Command,
    Rename,
    Search,
    ConfirmQuit,
    ConfirmDelete,
    CreateItem,
    Help,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusedPanel {
    Collections,
    Apis,
    Environments,
    Properties,
    Details,
    Response,
    Stats,
    RequestBar,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LeftBottomTab {
    Apis,
    Environments,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RequestBarPart {
    Method,
    Url,
    Send,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PropertyTab {
    Params,
    Headers,
    Auth,
    Body,
    Scripts,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PropertyEditorField {
    Key,
    Value,
    Description,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TuiAction {
    SendRequest,
    EditBody,
    CopyBody,
    PasteBody,
    CopyResponseBody,
    CopyResponseAll,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PendingItemType {
    Collection,
    Folder,
    Request,
    KVParam,
    EnvVar,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VisibleItemType {
    Collection { expanded: bool },
    Folder { expanded: bool },
    Request { method: Method, id: String },
}
