use crate::{
    controller::*,
    render::{Renderer, WinColorFmt, WinDepthFmt},
    settings::{ControlSettings, Settings},
    ui, Error,
};
use common_base::span;
use crossbeam::channel;
use gilrs::{EventType, Gilrs};
use hashbrown::HashMap;
use itertools::Itertools;
use old_school_gfx_glutin_ext::{ContextBuilderExt, WindowInitExt, WindowUpdateExt};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{error, info, warn};
use vek::*;
use winit::monitor::VideoMode;

/// Represents a key that the game recognises after input mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum GameInput {
    Primary,
    Secondary,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
    Slot9,
    Slot10,
    ToggleCursor,
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,
    Jump,
    Sit,
    Dance,
    Glide,
    Climb,
    ClimbDown,
    SwimUp,
    SwimDown,
    Fly,
    Sneak,
    ToggleLantern,
    Mount,
    Chat,
    Command,
    Escape,
    Map,
    Bag,
    Trade,
    Social,
    Crafting,
    Spellbook,
    Settings,
    ToggleInterface,
    Help,
    ToggleDebug,
    Fullscreen,
    Screenshot,
    ToggleIngameUi,
    Roll,
    Respawn,
    Interact,
    ToggleWield,
    SwapLoadout,
    FreeLook,
    AutoWalk,
    CameraClamp,
    CycleCamera,
    Select,
    AcceptGroupInvite,
    DeclineGroupInvite,
}

impl GameInput {
    pub fn get_localization_key(&self) -> &str {
        match *self {
            GameInput::Primary => "gameinput.primary",
            GameInput::Secondary => "gameinput.secondary",
            GameInput::ToggleCursor => "gameinput.togglecursor",
            GameInput::MoveForward => "gameinput.moveforward",
            GameInput::MoveLeft => "gameinput.moveleft",
            GameInput::MoveRight => "gameinput.moveright",
            GameInput::MoveBack => "gameinput.moveback",
            GameInput::Jump => "gameinput.jump",
            GameInput::Sit => "gameinput.sit",
            GameInput::Dance => "gameinput.dance",
            GameInput::Glide => "gameinput.glide",
            GameInput::Climb => "gameinput.climb",
            GameInput::ClimbDown => "gameinput.climbdown",
            GameInput::SwimUp => "gameinput.swimup",
            GameInput::SwimDown => "gameinput.swimdown",
            GameInput::Fly => "gameinput.fly",
            GameInput::Sneak => "gameinput.sneak",
            GameInput::ToggleLantern => "gameinput.togglelantern",
            GameInput::Mount => "gameinput.mount",
            GameInput::Chat => "gameinput.chat",
            GameInput::Command => "gameinput.command",
            GameInput::CycleCamera => "gameinput.cyclecamera",
            GameInput::Escape => "gameinput.escape",
            GameInput::Map => "gameinput.map",
            GameInput::Bag => "gameinput.bag",
            GameInput::Trade => "gameinput.trade",
            GameInput::Social => "gameinput.social",
            GameInput::Crafting => "gameinput.crafting",
            GameInput::Spellbook => "gameinput.spellbook",
            GameInput::Settings => "gameinput.settings",
            GameInput::ToggleInterface => "gameinput.toggleinterface",
            GameInput::Help => "gameinput.help",
            GameInput::ToggleDebug => "gameinput.toggledebug",
            GameInput::Fullscreen => "gameinput.fullscreen",
            GameInput::Screenshot => "gameinput.screenshot",
            GameInput::ToggleIngameUi => "gameinput.toggleingameui",
            GameInput::Roll => "gameinput.roll",
            GameInput::Respawn => "gameinput.respawn",
            GameInput::Interact => "gameinput.interact",
            GameInput::ToggleWield => "gameinput.togglewield",
            GameInput::FreeLook => "gameinput.freelook",
            GameInput::AutoWalk => "gameinput.autowalk",
            GameInput::CameraClamp => "gameinput.cameraclamp",
            GameInput::Slot1 => "gameinput.slot1",
            GameInput::Slot2 => "gameinput.slot2",
            GameInput::Slot3 => "gameinput.slot3",
            GameInput::Slot4 => "gameinput.slot4",
            GameInput::Slot5 => "gameinput.slot5",
            GameInput::Slot6 => "gameinput.slot6",
            GameInput::Slot7 => "gameinput.slot7",
            GameInput::Slot8 => "gameinput.slot8",
            GameInput::Slot9 => "gameinput.slot9",
            GameInput::Slot10 => "gameinput.slot10",
            GameInput::SwapLoadout => "gameinput.swaploadout",
            GameInput::Select => "gameinput.select",
            GameInput::AcceptGroupInvite => "gameinput.acceptgroupinvite",
            GameInput::DeclineGroupInvite => "gameinput.declinegroupinvite",
        }
    }

    pub fn iterator() -> impl Iterator<Item = GameInput> {
        [
            GameInput::Primary,
            GameInput::Secondary,
            GameInput::ToggleCursor,
            GameInput::MoveForward,
            GameInput::MoveLeft,
            GameInput::MoveRight,
            GameInput::MoveBack,
            GameInput::Jump,
            GameInput::Sit,
            GameInput::Dance,
            GameInput::Glide,
            GameInput::Climb,
            GameInput::ClimbDown,
            GameInput::SwimUp,
            GameInput::SwimDown,
            GameInput::Fly,
            GameInput::Sneak,
            GameInput::ToggleLantern,
            GameInput::Mount,
            GameInput::Chat,
            GameInput::Command,
            GameInput::Escape,
            GameInput::Map,
            GameInput::Bag,
            GameInput::Trade,
            GameInput::Social,
            GameInput::Crafting,
            GameInput::Spellbook,
            GameInput::Settings,
            GameInput::ToggleInterface,
            GameInput::Help,
            GameInput::ToggleDebug,
            GameInput::Fullscreen,
            GameInput::Screenshot,
            GameInput::ToggleIngameUi,
            GameInput::Roll,
            GameInput::Respawn,
            GameInput::Interact,
            GameInput::ToggleWield,
            GameInput::FreeLook,
            GameInput::AutoWalk,
            GameInput::CameraClamp,
            GameInput::Slot1,
            GameInput::Slot2,
            GameInput::Slot3,
            GameInput::Slot4,
            GameInput::Slot5,
            GameInput::Slot6,
            GameInput::Slot7,
            GameInput::Slot8,
            GameInput::Slot9,
            GameInput::Slot10,
            GameInput::SwapLoadout,
            GameInput::CycleCamera,
            GameInput::Select,
            GameInput::AcceptGroupInvite,
            GameInput::DeclineGroupInvite,
        ]
        .iter()
        .copied()
    }

    /// Return true if `a` and `b` are able to be bound to the same key at the
    /// same time without conflict. For example, the player can't jump and climb
    /// at the same time, so these can be bound to the same key.
    pub fn can_share_bindings(a: GameInput, b: GameInput) -> bool {
        a.get_representative_binding() == b.get_representative_binding()
    }

    /// If two GameInputs are able to be bound at the same time, then they will
    /// return the same value from this function (the representative value for
    /// that set). This models the Find operation of a disjoint-set data
    /// structure.
    fn get_representative_binding(&self) -> GameInput {
        match self {
            GameInput::Jump => GameInput::Jump,
            GameInput::Climb => GameInput::Jump,
            GameInput::SwimUp => GameInput::Jump,
            GameInput::Respawn => GameInput::Jump,

            GameInput::FreeLook => GameInput::FreeLook,
            GameInput::AutoWalk => GameInput::FreeLook,

            _ => *self,
        }
    }
}

/// Represents a key that the game menus recognise after input mapping
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MenuInput {
    Up,
    Down,
    Left,
    Right,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    Home,
    End,
    Apply,
    Back,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum AnalogMenuInput {
    MoveX(f32),
    MoveY(f32),
    ScrollX(f32),
    ScrollY(f32),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum AnalogGameInput {
    MovementX(f32),
    MovementY(f32),
    CameraX(f32),
    CameraY(f32),
}

/// Represents an incoming event from the window.
#[derive(Clone, Debug)]
pub enum Event {
    /// The window has been requested to close.
    Close,
    /// The window has been resized.
    Resize(Vec2<u32>),
    /// The window scale factor has been changed
    ScaleFactorChanged(f64),
    /// The window has been moved.
    Moved(Vec2<u32>),
    /// A key has been typed that corresponds to a specific character.
    Char(char),
    /// The cursor has been panned across the screen while grabbed.
    CursorPan(Vec2<f32>),
    /// The cursor has been moved across the screen while ungrabbed.
    CursorMove(Vec2<f32>),
    /// A mouse button has been pressed or released
    MouseButton(MouseButton, PressState),
    /// The camera has been requested to zoom.
    Zoom(f32),
    /// A key that the game recognises has been pressed or released.
    InputUpdate(GameInput, bool),
    /// Event that the ui uses.
    Ui(ui::Event),
    /// Event that the iced ui uses.
    IcedUi(ui::ice::Event),
    /// The view distance has changed.
    ViewDistanceChanged(u32),
    /// Game settings have changed.
    SettingsChanged,
    /// The window is (un)focused
    Focused(bool),
    /// A key that the game recognises for menu navigation has been pressed or
    /// released
    MenuInput(MenuInput, bool),
    /// Update of the analog inputs recognized by the menus
    AnalogMenuInput(AnalogMenuInput),
    /// Update of the analog inputs recognized by the game
    AnalogGameInput(AnalogGameInput),
    /// We tried to save a screenshot
    ScreenshotMessage(String),
}

pub type MouseButton = winit::event::MouseButton;
pub type PressState = winit::event::ElementState;
pub type EventLoop = winit::event_loop::EventLoop<()>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum KeyMouse {
    Key(winit::event::VirtualKeyCode),
    Mouse(winit::event::MouseButton),
    ScanKey(winit::event::ScanCode),
}

impl fmt::Display for KeyMouse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::KeyMouse::*;
        use winit::event::{MouseButton, VirtualKeyCode::*};
        write!(f, "{}", match self {
            Key(Key1) => "1",
            Key(Key2) => "2",
            Key(Key3) => "3",
            Key(Key4) => "4",
            Key(Key5) => "5",
            Key(Key6) => "6",
            Key(Key7) => "7",
            Key(Key8) => "8",
            Key(Key9) => "9",
            Key(Key0) => "0",
            Key(A) => "A",
            Key(B) => "B",
            Key(C) => "C",
            Key(D) => "D",
            Key(E) => "E",
            Key(F) => "F",
            Key(G) => "G",
            Key(H) => "H",
            Key(I) => "I",
            Key(J) => "J",
            Key(K) => "K",
            Key(L) => "L",
            Key(M) => "M",
            Key(N) => "N",
            Key(O) => "O",
            Key(P) => "P",
            Key(Q) => "Q",
            Key(R) => "R",
            Key(S) => "S",
            Key(T) => "T",
            Key(U) => "U",
            Key(V) => "V",
            Key(W) => "W",
            Key(X) => "X",
            Key(Y) => "Y",
            Key(Z) => "Z",
            Key(Escape) => "ESC",
            Key(F1) => "F1",
            Key(F2) => "F2",
            Key(F3) => "F3",
            Key(F4) => "F4",
            Key(F5) => "F5",
            Key(F6) => "F6",
            Key(F7) => "F7",
            Key(F8) => "F8",
            Key(F9) => "F9",
            Key(F10) => "F10",
            Key(F11) => "F11",
            Key(F12) => "F12",
            Key(F13) => "F13",
            Key(F14) => "F14",
            Key(F15) => "F15",
            Key(F16) => "F16",
            Key(F17) => "F17",
            Key(F18) => "F18",
            Key(F19) => "F19",
            Key(F20) => "F20",
            Key(F21) => "F21",
            Key(F22) => "F22",
            Key(F23) => "F23",
            Key(F24) => "F24",
            Key(Snapshot) => "Print Screen",
            Key(Scroll) => "Scroll Lock",
            Key(Pause) => "Pause/Break",
            Key(Insert) => "Insert",
            Key(Home) => "Home",
            Key(Delete) => "Delete",
            Key(End) => "End",
            Key(PageDown) => "PageDown",
            Key(PageUp) => "PageUp",
            Key(Left) => "Left Arrow",
            Key(Up) => "Up Arrow",
            Key(Right) => "Right Arrow",
            Key(Down) => "Down Arrow",
            Key(Back) => "Backspace",
            Key(Return) => "Enter",
            Key(Space) => "Space",
            Key(Compose) => "Compose",
            Key(Caret) => "^",
            Key(Numlock) => "Numlock",
            Key(Numpad0) => "Numpad 0",
            Key(Numpad1) => "Numpad 1",
            Key(Numpad2) => "Numpad 2",
            Key(Numpad3) => "Numpad 3",
            Key(Numpad4) => "Numpad 4",
            Key(Numpad5) => "Numpad 5",
            Key(Numpad6) => "Numpad 6",
            Key(Numpad7) => "Numpad 7",
            Key(Numpad8) => "Numpad 8",
            Key(Numpad9) => "Numpad 9",
            Key(AbntC1) => "Abnt C1",
            Key(AbntC2) => "Abnt C2",
            Key(NumpadAdd) => "Numpad +",
            Key(Apostrophe) => "'",
            Key(Apps) => "Context Menu",
            Key(At) => "@",
            Key(Ax) => "Ax",
            Key(Backslash) => "\\",
            Key(Calculator) => "Calculator",
            Key(Capital) => "Caps Lock",
            Key(Colon) => ":",
            Key(Comma) => ",",
            Key(Convert) => "Convert",
            Key(NumpadDecimal) => "Numpad .",
            Key(NumpadDivide) => "Numpad /",
            Key(Equals) => "=",
            Key(Grave) => "`",
            Key(Kana) => "Kana",
            Key(Kanji) => "Kanji",
            Key(LAlt) => "LAlt",
            Key(LBracket) => "[",
            Key(LControl) => "LControl",
            Key(LShift) => "LShift",
            Key(LWin) => "LWin",
            Key(Mail) => "Mail",
            Key(MediaSelect) => "MediaSelect",
            Key(MediaStop) => "MediaStop",
            Key(Minus) => "-",
            Key(NumpadMultiply) => "Numpad *",
            Key(Mute) => "Mute",
            Key(MyComputer) => "My Computer",
            Key(NavigateForward) => "Navigate Forward",
            Key(NavigateBackward) => "Navigate Backward",
            Key(NextTrack) => "Next Track",
            Key(NoConvert) => "Non Convert",
            Key(NumpadComma) => "Num ,",
            Key(NumpadEnter) => "Num Enter",
            Key(NumpadEquals) => "Num =",
            Key(OEM102) => "<",
            Key(Period) => ".",
            Key(PlayPause) => "Play / Pause",
            Key(Power) => "Power",
            Key(PrevTrack) => "Prev Track",
            Key(RAlt) => "RAlt",
            Key(RBracket) => "]",
            Key(RControl) => "RControl",
            Key(RShift) => "RShift",
            Key(RWin) => "RWin",
            Key(Semicolon) => ";",
            Key(Slash) => "/",
            Key(Sleep) => "Sleep",
            Key(Stop) => "Media Stop",
            Key(NumpadSubtract) => "Num -",
            Key(Sysrq) => "Sysrq",
            Key(Tab) => "Tab",
            Key(Underline) => "_",
            Key(Unlabeled) => "No Name",
            Key(VolumeDown) => "Volume Down",
            Key(VolumeUp) => "Volume Up",
            Key(Wake) => "Wake",
            Key(WebBack) => "Browser Back",
            Key(WebFavorites) => "Browser Favorites",
            Key(WebForward) => "Browser Forward",
            Key(WebHome) => "Browser Home",
            Key(WebRefresh) => "Browser Refresh",
            Key(WebSearch) => "Browser Search",
            Key(WebStop) => "Browser Stop",
            Key(Yen) => "Yen",
            Key(Copy) => "Copy",
            Key(Paste) => "Paste",
            Key(Cut) => "Cut",
            Key(Asterisk) => "*",
            Key(Plus) => "+",
            Mouse(MouseButton::Left) => "M1",
            Mouse(MouseButton::Right) => "M2",
            Mouse(MouseButton::Middle) => "M3",
            Mouse(MouseButton::Other(button)) =>
            // Additional mouse buttons after middle click start at 1
                return write!(f, "M{}", button + 3),
            ScanKey(_) => "Unknown",
        })
    }
}

pub struct Window {
    renderer: Renderer,
    window: glutin::ContextWrapper<glutin::PossiblyCurrent, winit::window::Window>,
    cursor_grabbed: bool,
    pub pan_sensitivity: u32,
    pub zoom_sensitivity: u32,
    pub zoom_inversion: bool,
    pub mouse_y_inversion: bool,
    fullscreen: FullScreenSettings,
    modifiers: winit::event::ModifiersState,
    scale_factor: f64,
    needs_refresh_resize: bool,
    keypress_map: HashMap<GameInput, winit::event::ElementState>,
    pub remapping_keybindings: Option<GameInput>,
    events: Vec<Event>,
    focused: bool,
    gilrs: Option<Gilrs>,
    pub controller_settings: ControllerSettings,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    mouse_emulation_vec: Vec2<f32>,
    // Currently used to send and receive screenshot result messages
    message_sender: channel::Sender<String>,
    message_receiver: channel::Receiver<String>,
    // Used for screenshots & fullscreen toggle to deduplicate/postpone to after event handler
    take_screenshot: bool,
    toggle_fullscreen: bool,
}

impl Window {
    pub fn new(settings: &Settings) -> Result<(Window, EventLoop), Error> {
        let event_loop = EventLoop::new();

        let size = settings.graphics.window_size;

        let win_builder = winit::window::WindowBuilder::new()
            .with_title("Veloren")
            .with_inner_size(winit::dpi::LogicalSize::new(size[0] as f64, size[1] as f64))
            .with_maximized(true);

        // Avoid cpal / winit OleInitialize conflict
        // See: https://github.com/rust-windowing/winit/pull/1524
        #[cfg(target_os = "windows")]
        let win_builder = winit::platform::windows::WindowBuilderExtWindows::with_drag_and_drop(
            win_builder,
            false,
        );

        let (window, device, factory, win_color_view, win_depth_view) =
            glutin::ContextBuilder::new()
                .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
                .with_vsync(false)
                .with_gfx_color_depth::<WinColorFmt, WinDepthFmt>()
                .build_windowed(win_builder, &event_loop)
                .map_err(|err| Error::BackendError(Box::new(err)))?
                .init_gfx::<WinColorFmt, WinDepthFmt>();

        let vendor = device.get_info().platform_name.vendor;
        let renderer = device.get_info().platform_name.renderer;
        let opengl_version = device.get_info().version;
        let glsl_version = device.get_info().shading_language;
        info!(
            ?vendor,
            ?renderer,
            ?opengl_version,
            ?glsl_version,
            "selected graphics device"
        );

        let keypress_map = HashMap::new();

        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(gilrs::Error::NotImplemented(_dummy)) => {
                warn!("Controller input is unsupported on this platform.");
                None
            },
            Err(gilrs::Error::InvalidAxisToBtn) => {
                error!(
                    "Invalid AxisToBtn controller mapping. Falling back to no controller support."
                );
                None
            },
            Err(gilrs::Error::Other(e)) => {
                error!(
                    ?e,
                    "Platform-specific error when creating a Gilrs instance. Falling back to no \
                     controller support."
                );
                None
            },
        };

        let controller_settings = ControllerSettings::from(&settings.controller);

        let (message_sender, message_receiver): (
            channel::Sender<String>,
            channel::Receiver<String>,
        ) = channel::unbounded::<String>();

        let scale_factor = window.window().scale_factor();

        let mut this = Self {
            renderer: Renderer::new(
                device,
                factory,
                win_color_view,
                win_depth_view,
                settings.graphics.render_mode.clone(),
            )?,
            window,
            cursor_grabbed: false,
            pan_sensitivity: settings.gameplay.pan_sensitivity,
            zoom_sensitivity: settings.gameplay.zoom_sensitivity,
            zoom_inversion: settings.gameplay.zoom_inversion,
            mouse_y_inversion: settings.gameplay.mouse_y_inversion,
            fullscreen: FullScreenSettings::default(),
            modifiers: Default::default(),
            scale_factor,
            needs_refresh_resize: false,
            keypress_map,
            remapping_keybindings: None,
            events: Vec::new(),
            focused: true,
            gilrs,
            controller_settings,
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            mouse_emulation_vec: Vec2::zero(),
            // Currently used to send and receive screenshot result messages
            message_sender,
            message_receiver,
            take_screenshot: false,
            toggle_fullscreen: false,
        };

        this.set_fullscreen_mode(settings.graphics.fullscreen);

        Ok((this, event_loop))
    }

    pub fn renderer(&self) -> &Renderer { &self.renderer }

    pub fn renderer_mut(&mut self) -> &mut Renderer { &mut self.renderer }

    pub fn resolve_deduplicated_events(&mut self, settings: &mut Settings) {
        // Handle screenshots and toggling fullscreen
        if self.take_screenshot {
            self.take_screenshot = false;
            self.take_screenshot(&settings);
        }
        if self.toggle_fullscreen {
            self.toggle_fullscreen = false;
            self.toggle_fullscreen(settings);
        }
    }

    pub fn fetch_events(&mut self) -> Vec<Event> {
        // Refresh ui size (used when changing playstates)
        if self.needs_refresh_resize {
            let logical_size = self.logical_size();
            self.events
                .push(Event::Ui(ui::Event::new_resize(logical_size)));
            self.events.push(Event::IcedUi(iced::Event::Window(
                iced::window::Event::Resized {
                    width: logical_size.x as u32,
                    height: logical_size.y as u32,
                },
            )));
            self.events
                .push(Event::ScaleFactorChanged(self.scale_factor));
            self.needs_refresh_resize = false;
        }

        // Receive any messages sent through the message channel
        for message in self.message_receiver.try_iter() {
            self.events.push(Event::ScreenshotMessage(message))
        }

        if let Some(gilrs) = &mut self.gilrs {
            while let Some(event) = gilrs.next_event() {
                fn handle_buttons(
                    settings: &ControllerSettings,
                    events: &mut Vec<Event>,
                    button: &Button,
                    is_pressed: bool,
                ) {
                    if let Some(evs) = settings.game_button_map.get(button) {
                        for ev in evs {
                            events.push(Event::InputUpdate(*ev, is_pressed));
                        }
                    }
                    if let Some(evs) = settings.menu_button_map.get(button) {
                        for ev in evs {
                            events.push(Event::MenuInput(*ev, is_pressed));
                        }
                    }
                }

                match event.event {
                    EventType::ButtonPressed(button, code)
                    | EventType::ButtonRepeated(button, code) => {
                        handle_buttons(
                            &self.controller_settings,
                            &mut self.events,
                            &Button::from((button, code)),
                            true,
                        );
                    },
                    EventType::ButtonReleased(button, code) => {
                        handle_buttons(
                            &self.controller_settings,
                            &mut self.events,
                            &Button::from((button, code)),
                            false,
                        );
                    },
                    EventType::ButtonChanged(button, _value, code) => {
                        if let Some(actions) = self
                            .controller_settings
                            .game_analog_button_map
                            .get(&AnalogButton::from((button, code)))
                        {
                            for action in actions {
                                match *action {}
                            }
                        }
                        if let Some(actions) = self
                            .controller_settings
                            .menu_analog_button_map
                            .get(&AnalogButton::from((button, code)))
                        {
                            for action in actions {
                                match *action {}
                            }
                        }
                    },

                    EventType::AxisChanged(axis, value, code) => {
                        let value = if self
                            .controller_settings
                            .inverted_axes
                            .contains(&Axis::from((axis, code)))
                        {
                            -value
                        } else {
                            value
                        };

                        let value = self
                            .controller_settings
                            .apply_axis_deadzone(&Axis::from((axis, code)), value);

                        if self.cursor_grabbed {
                            if let Some(actions) = self
                                .controller_settings
                                .game_axis_map
                                .get(&Axis::from((axis, code)))
                            {
                                for action in actions {
                                    match *action {
                                        AxisGameAction::MovementX => {
                                            self.events.push(Event::AnalogGameInput(
                                                AnalogGameInput::MovementX(value),
                                            ));
                                        },
                                        AxisGameAction::MovementY => {
                                            self.events.push(Event::AnalogGameInput(
                                                AnalogGameInput::MovementY(value),
                                            ));
                                        },
                                        AxisGameAction::CameraX => {
                                            self.events.push(Event::AnalogGameInput(
                                                AnalogGameInput::CameraX(
                                                    value
                                                        * self.controller_settings.pan_sensitivity
                                                            as f32
                                                        / 100.0,
                                                ),
                                            ));
                                        },
                                        AxisGameAction::CameraY => {
                                            let pan_invert_y =
                                                match self.controller_settings.pan_invert_y {
                                                    true => -1.0,
                                                    false => 1.0,
                                                };

                                            self.events.push(Event::AnalogGameInput(
                                                AnalogGameInput::CameraY(
                                                    -value
                                                        * self.controller_settings.pan_sensitivity
                                                            as f32
                                                        * pan_invert_y
                                                        / 100.0,
                                                ),
                                            ));
                                        },
                                    }
                                }
                            }
                        } else if let Some(actions) = self
                            .controller_settings
                            .menu_axis_map
                            .get(&Axis::from((axis, code)))
                        {
                            // TODO: possibly add sensitivity settings when this is used
                            for action in actions {
                                match *action {
                                    AxisMenuAction::MoveX => {
                                        self.events.push(Event::AnalogMenuInput(
                                            AnalogMenuInput::MoveX(value),
                                        ));
                                    },
                                    AxisMenuAction::MoveY => {
                                        self.events.push(Event::AnalogMenuInput(
                                            AnalogMenuInput::MoveY(value),
                                        ));
                                    },
                                    AxisMenuAction::ScrollX => {
                                        self.events.push(Event::AnalogMenuInput(
                                            AnalogMenuInput::ScrollX(value),
                                        ));
                                    },
                                    AxisMenuAction::ScrollY => {
                                        self.events.push(Event::AnalogMenuInput(
                                            AnalogMenuInput::ScrollY(value),
                                        ));
                                    },
                                }
                            }
                        }
                    },
                    EventType::Connected => {},
                    EventType::Disconnected => {},
                    EventType::Dropped => {},
                }
            }
        }

        let mut events = std::mem::take(&mut self.events);
        // Mouse emulation for the menus, to be removed when a proper menu navigation
        // system is available
        if !self.cursor_grabbed {
            events = events
                .into_iter()
                .filter_map(|event| match event {
                    Event::AnalogMenuInput(input) => match input {
                        AnalogMenuInput::MoveX(d) => {
                            self.mouse_emulation_vec.x = d;
                            None
                        },
                        AnalogMenuInput::MoveY(d) => {
                            // This just has to be inverted for some reason
                            self.mouse_emulation_vec.y = d * -1.0;
                            None
                        },
                        input => Some(Event::AnalogMenuInput(input)),
                    },
                    Event::MenuInput(MenuInput::Apply, state) => Some(match state {
                        true => Event::Ui(ui::Event(conrod_core::event::Input::Press(
                            conrod_core::input::Button::Mouse(
                                conrod_core::input::state::mouse::Button::Left,
                            ),
                        ))),
                        false => Event::Ui(ui::Event(conrod_core::event::Input::Release(
                            conrod_core::input::Button::Mouse(
                                conrod_core::input::state::mouse::Button::Left,
                            ),
                        ))),
                    }),
                    _ => Some(event),
                })
                .collect();

            let sensitivity = self.controller_settings.mouse_emulation_sensitivity;
            // TODO: make this independent of framerate
            // TODO: consider multiplying by scale factor
            self.offset_cursor(self.mouse_emulation_vec * sensitivity as f32);
        }

        events
    }

    pub fn handle_device_event(&mut self, event: winit::event::DeviceEvent) {
        use winit::event::DeviceEvent;

        let mouse_y_inversion = match self.mouse_y_inversion {
            true => -1.0,
            false => 1.0,
        };

        match event {
            DeviceEvent::MouseMotion {
                delta: (dx, dy), ..
            } if self.focused => {
                let delta = Vec2::new(
                    dx as f32 * (self.pan_sensitivity as f32 / 100.0),
                    dy as f32 * (self.pan_sensitivity as f32 * mouse_y_inversion / 100.0),
                );

                if self.cursor_grabbed {
                    self.events.push(Event::CursorPan(delta));
                } else {
                    self.events.push(Event::CursorMove(delta));
                }
            },
            _ => {},
        }
    }

    pub fn handle_window_event(
        &mut self,
        event: winit::event::WindowEvent,
        settings: &mut Settings,
    ) {
        use winit::event::WindowEvent;

        let controls = &mut settings.controls;

        match event {
            WindowEvent::CloseRequested => self.events.push(Event::Close),
            WindowEvent::Resized(physical) => {
                let (mut color_view, mut depth_view) = self.renderer.win_views_mut();
                self.window.resize(physical);
                self.window.update_gfx(&mut color_view, &mut depth_view);
                self.renderer.on_resize().unwrap();
                // TODO: update users of this event with the fact that it is now the physical
                // size
                let winit::dpi::PhysicalSize { width, height } = physical;
                self.events
                    .push(Event::Resize(Vec2::new(width as u32, height as u32)));
            },
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.events.push(Event::ScaleFactorChanged(scale_factor));
            },
            WindowEvent::Moved(winit::dpi::PhysicalPosition { x, y }) => {
                self.events
                    .push(Event::Moved(Vec2::new(x as u32, y as u32)));
            },
            WindowEvent::ReceivedCharacter(c) => self.events.push(Event::Char(c)),
            WindowEvent::MouseInput { button, state, .. } => {
                if let (true, Some(game_inputs)) =
                    // Mouse input not mapped to input if it is not grabbed
                    (
                    self.cursor_grabbed,
                    Window::map_input(
                        KeyMouse::Mouse(button),
                        controls,
                        &mut self.remapping_keybindings,
                    ),
                ) {
                    for game_input in game_inputs {
                        self.events.push(Event::InputUpdate(
                            *game_input,
                            state == winit::event::ElementState::Pressed,
                        ));
                    }
                }
                self.events.push(Event::MouseButton(button, state));
            },
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers,
            WindowEvent::KeyboardInput {
                input,
                is_synthetic,
                ..
            } => {
                // Ignore synthetic tab presses so that we don't get tabs when alt-tabbing back
                // into the window
                if matches!(
                    input.virtual_keycode,
                    Some(winit::event::VirtualKeyCode::Tab)
                ) && is_synthetic
                {
                    return;
                }
                // Ignore Alt-F4 so we don't try to do anything heavy like take a screenshot
                // when the window is about to close
                if matches!(input, winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(winit::event::VirtualKeyCode::F4),
                    ..
                }) && self.modifiers.alt()
                {
                    return;
                }

                let input_key = match input.virtual_keycode {
                    Some(key) => KeyMouse::Key(key),
                    None => KeyMouse::ScanKey(input.scancode),
                };

                if let Some(game_inputs) =
                    Window::map_input(input_key, controls, &mut self.remapping_keybindings)
                {
                    for game_input in game_inputs {
                        match game_input {
                            GameInput::Fullscreen => {
                                if input.state == winit::event::ElementState::Pressed
                                    && !Self::is_pressed(
                                        &mut self.keypress_map,
                                        GameInput::Fullscreen,
                                    )
                                {
                                    self.toggle_fullscreen = !self.toggle_fullscreen;
                                }
                                Self::set_pressed(
                                    &mut self.keypress_map,
                                    GameInput::Fullscreen,
                                    input.state,
                                );
                            },
                            GameInput::Screenshot => {
                                self.take_screenshot = input.state
                                    == winit::event::ElementState::Pressed
                                    && !Self::is_pressed(
                                        &mut self.keypress_map,
                                        GameInput::Screenshot,
                                    );
                                Self::set_pressed(
                                    &mut self.keypress_map,
                                    GameInput::Screenshot,
                                    input.state,
                                );
                            },
                            _ => self.events.push(Event::InputUpdate(
                                *game_input,
                                input.state == winit::event::ElementState::Pressed,
                            )),
                        }
                    }
                }
            },
            WindowEvent::Focused(state) => {
                self.focused = state;
                self.events.push(Event::Focused(state));
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = position;
            },
            WindowEvent::MouseWheel { delta, .. } if self.cursor_grabbed && self.focused => {
                const DIFFERENCE_FROM_DEVICE_EVENT_ON_X11: f32 = -15.0;
                self.events.push(Event::Zoom({
                    let y = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => y,
                        // TODO: Check to see if there is a better way to find the "line
                        // height" than just hardcoding 16.0 pixels.  Alternately we could
                        // get rid of this and have the user set zoom sensitivity, since
                        // it's unlikely people would expect a configuration file to work
                        // across operating systems.
                        winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.y / 16.0) as f32,
                    };
                    y * (self.zoom_sensitivity as f32 / 100.0)
                        * if self.zoom_inversion { -1.0 } else { 1.0 }
                        * DIFFERENCE_FROM_DEVICE_EVENT_ON_X11
                }))
            },
            _ => {},
        }
    }

    /// Moves cursor by an offset
    pub fn offset_cursor(&self, d: Vec2<f32>) {
        if d != Vec2::zero() {
            if let Err(err) =
                self.window
                    .window()
                    .set_cursor_position(winit::dpi::LogicalPosition::new(
                        d.x as f64 + self.cursor_position.x,
                        d.y as f64 + self.cursor_position.y,
                    ))
            {
                error!("Error setting cursor position: {:?}", err);
            }
        }
    }

    pub fn swap_buffers(&self) -> Result<(), Error> {
        span!(_guard, "swap_buffers", "Window::swap_buffers");
        self.window
            .swap_buffers()
            .map_err(|err| Error::BackendError(Box::new(err)))
    }

    pub fn is_cursor_grabbed(&self) -> bool { self.cursor_grabbed }

    pub fn grab_cursor(&mut self, grab: bool) {
        self.cursor_grabbed = grab;
        self.window.window().set_cursor_visible(!grab);
        let _ = self.window.window().set_cursor_grab(grab);
    }

    pub fn toggle_fullscreen(&mut self, settings: &mut Settings) {
        let fullscreen = FullScreenSettings {
            enabled: !self.is_fullscreen(),
            ..settings.graphics.fullscreen
        };

        self.set_fullscreen_mode(fullscreen);
        settings.graphics.fullscreen = fullscreen;
        settings.save_to_file_warn();
    }

    pub fn is_fullscreen(&self) -> bool { self.fullscreen.enabled }

    pub fn select_video_mode_rec(
        &self,
        resolution: [u16; 2],
        bit_depth: Option<u16>,
        refresh_rate: Option<u16>,
        correct_res: Option<Vec<VideoMode>>,
        correct_depth: Option<Option<VideoMode>>,
        correct_rate: Option<Option<VideoMode>>,
    ) -> Option<VideoMode> {
        // if a previous iteration of this method filtered the available video modes for
        // the correct resolution already, load that value, otherwise filter it
        // in this iteration
        let correct_res = correct_res.unwrap_or_else(|| {
            let window = self.window.window();
            window
                .current_monitor()
                .unwrap()
                .video_modes()
                .filter(|mode| mode.size().width == resolution[0] as u32)
                .filter(|mode| mode.size().height == resolution[1] as u32)
                .collect()
        });

        match bit_depth {
            // A bit depth is given
            Some(depth) => {
                // analogous to correct_res
                let correct_depth = correct_depth.unwrap_or_else(|| {
                    correct_res
                        .iter()
                        .find(|mode| mode.bit_depth() == depth)
                        .cloned()
                });

                match refresh_rate {
                    // A bit depth and a refresh rate is given
                    Some(rate) => {
                        // analogous to correct_res
                        let correct_rate = correct_rate.unwrap_or_else(|| {
                            correct_res
                                .iter()
                                .find(|mode| mode.refresh_rate() == rate)
                                .cloned()
                        });

                        // if no video mode with the given bit depth and refresh rate exists, fall
                        // back to a video mode that fits the resolution and either bit depth or
                        // refresh rate depending on which parameter was causing the correct video
                        // mode not to be found
                        correct_res
                            .iter()
                            .filter(|mode| mode.bit_depth() == depth)
                            .find(|mode| mode.refresh_rate() == rate)
                            .cloned()
                            .or_else(|| {
                                if correct_depth.is_none() && correct_rate.is_none() {
                                    warn!(
                                        "Bit depth and refresh rate specified in settings are \
                                         incompatible with the monitor. Choosing highest bit \
                                         depth and refresh rate possible instead."
                                    );
                                }

                                self.select_video_mode_rec(
                                    resolution,
                                    correct_depth.is_some().then_some(depth),
                                    correct_rate.is_some().then_some(rate),
                                    Some(correct_res),
                                    Some(correct_depth),
                                    Some(correct_rate),
                                )
                            })
                    },
                    // A bit depth and no refresh rate is given
                    // if no video mode with the given bit depth exists, fall
                    // back to a video mode that fits only the resolution
                    None => match correct_depth {
                        Some(mode) => Some(mode),
                        None => {
                            warn!(
                                "Bit depth specified in settings is incompatible with the \
                                 monitor. Choosing highest bit depth possible instead."
                            );

                            self.select_video_mode_rec(
                                resolution,
                                None,
                                None,
                                Some(correct_res),
                                Some(correct_depth),
                                None,
                            )
                        },
                    },
                }
            },
            // No bit depth is given
            None => match refresh_rate {
                // No bit depth and a refresh rate is given
                Some(rate) => {
                    // analogous to correct_res
                    let correct_rate = correct_rate.unwrap_or_else(|| {
                        correct_res
                            .iter()
                            .find(|mode| mode.refresh_rate() == rate)
                            .cloned()
                    });

                    // if no video mode with the given bit depth exists, fall
                    // back to a video mode that fits only the resolution
                    match correct_rate {
                        Some(mode) => Some(mode),
                        None => {
                            warn!(
                                "Refresh rate specified in settings is incompatible with the \
                                 monitor. Choosing highest refresh rate possible instead."
                            );

                            self.select_video_mode_rec(
                                resolution,
                                None,
                                None,
                                Some(correct_res),
                                None,
                                Some(correct_rate),
                            )
                        },
                    }
                },
                // No bit depth and no refresh rate is given
                // get the video mode with the specified resolution and the max bit depth and
                // refresh rate
                None => correct_res
                    .into_iter()
                    // Prefer bit depth over refresh rate
                    .sorted_by_key(|mode| mode.bit_depth())
                    .max_by_key(|mode| mode.refresh_rate()),
            },
        }
    }

    pub fn select_video_mode(
        &self,
        resolution: [u16; 2],
        bit_depth: Option<u16>,
        refresh_rate: Option<u16>,
    ) -> VideoMode {
        // (resolution, bit depth, refresh rate) represents a video mode
        // spec: as specified
        // max: maximum value available

        // order of fallbacks as follows:
        // (spec, spec, spec)
        // (spec, spec, max), (spec, max, spec)
        // (spec, max, max)
        // (max, max, max)
        self.select_video_mode_rec(resolution, bit_depth, refresh_rate, None, None, None)
            // if there is no video mode with the specified resolution, fall back to the video mode with max resolution, bit depth and refresh rate
            .unwrap_or_else(|| {
                warn!(
                    "Resolution specified in settings is incompatible with the monitor. Choosing \
                     highest resolution possible instead."
                );

                self
                    .window
                    .window()
                    .current_monitor().unwrap()
                    .video_modes()
                    // Prefer bit depth over refresh rate
                    .sorted_by_key(|mode| mode.refresh_rate())
                    .sorted_by_key(|mode| mode.bit_depth())
                    .max_by_key(|mode| mode.size().width)
                    .expect("No video modes available!!")
            })
    }

    pub fn set_fullscreen_mode(&mut self, fullscreen: FullScreenSettings) {
        let window = self.window.window();
        self.fullscreen = fullscreen;
        window.set_fullscreen(fullscreen.enabled.then(|| match fullscreen.mode {
            FullscreenMode::Exclusive => {
                winit::window::Fullscreen::Exclusive(self.select_video_mode(
                    fullscreen.resolution,
                    fullscreen.bit_depth,
                    fullscreen.refresh_rate,
                ))
            },
            FullscreenMode::Borderless => {
                winit::window::Fullscreen::Borderless(window.current_monitor())
            },
        }));
    }

    pub fn needs_refresh_resize(&mut self) { self.needs_refresh_resize = true; }

    #[allow(clippy::or_fun_call)] // TODO: Pending review in #587
    pub fn logical_size(&self) -> Vec2<f64> {
        let (w, h) = self
            .window
            .window()
            .inner_size()
            .to_logical::<f64>(self.window.window().scale_factor())
            .into();
        Vec2::new(w, h)
    }

    pub fn set_size(&mut self, new_size: Vec2<u16>) {
        self.window
            .window()
            .set_inner_size(glutin::dpi::LogicalSize::new(
                new_size.x as f64,
                new_size.y as f64,
            ));
    }

    pub fn send_event(&mut self, event: Event) { self.events.push(event) }

    pub fn take_screenshot(&mut self, settings: &Settings) {
        match self.renderer.create_screenshot() {
            Ok(img) => {
                let mut path = settings.screenshots_path.clone();
                let sender = self.message_sender.clone();

                let builder = std::thread::Builder::new().name("screenshot".into());
                builder
                    .spawn(move || {
                        use std::time::SystemTime;
                        // Check if folder exists and create it if it does not
                        if !path.exists() {
                            if let Err(e) = std::fs::create_dir_all(&path) {
                                warn!(?e, "Couldn't create folder for screenshot");
                                let _result = sender
                                    .send(String::from("Couldn't create folder for screenshot"));
                            }
                        }
                        path.push(format!(
                            "screenshot_{}.png",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or(0)
                        ));
                        if let Err(e) = img.save(&path) {
                            warn!(?e, "Couldn't save screenshot");
                            let _result = sender.send(String::from("Couldn't save screenshot"));
                        } else {
                            let _result = sender
                                .send(format!("Screenshot saved to {}", path.to_string_lossy()));
                        }
                    })
                    .unwrap();
            },
            Err(e) => error!(?e, "Couldn't create screenshot due to renderer error"),
        }
    }

    fn is_pressed(
        map: &mut HashMap<GameInput, winit::event::ElementState>,
        input: GameInput,
    ) -> bool {
        *(map
            .entry(input)
            .or_insert(winit::event::ElementState::Released))
            == winit::event::ElementState::Pressed
    }

    fn set_pressed(
        map: &mut HashMap<GameInput, winit::event::ElementState>,
        input: GameInput,
        state: winit::event::ElementState,
    ) {
        map.insert(input, state);
    }

    // Function used to handle Mouse and Key events. It first checks if we're in
    // remapping mode for a specific GameInput. If we are, we modify the binding
    // of that GameInput with the KeyMouse passed. Else, we return an iterator of
    // the GameInputs for that KeyMouse.
    fn map_input<'a>(
        key_mouse: KeyMouse,
        controls: &'a mut ControlSettings,
        remapping: &mut Option<GameInput>,
    ) -> Option<impl Iterator<Item = &'a GameInput>> {
        match *remapping {
            // TODO: save settings
            Some(game_input) => {
                controls.modify_binding(game_input, key_mouse);
                *remapping = None;
                None
            },
            None => controls
                .get_associated_game_inputs(&key_mouse)
                .map(|game_inputs| game_inputs.iter()),
        }
    }

    pub fn set_keybinding_mode(&mut self, game_input: GameInput) {
        self.remapping_keybindings = Some(game_input);
    }

    pub fn window(&self) -> &winit::window::Window { self.window.window() }

    pub fn modifiers(&self) -> winit::event::ModifiersState { self.modifiers }

    pub fn scale_factor(&self) -> f64 { self.scale_factor }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum FullscreenMode {
    Exclusive,
    #[serde(other)]
    Borderless,
}

impl Default for FullscreenMode {
    fn default() -> Self { FullscreenMode::Borderless }
}
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FullScreenSettings {
    pub enabled: bool,
    pub mode: FullscreenMode,
    pub resolution: [u16; 2],
    pub bit_depth: Option<u16>,
    pub refresh_rate: Option<u16>,
}

impl Default for FullScreenSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: FullscreenMode::Borderless,
            resolution: [1920, 1080],
            bit_depth: None,
            refresh_rate: None,
        }
    }
}
