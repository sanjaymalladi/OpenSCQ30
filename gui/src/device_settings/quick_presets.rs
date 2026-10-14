use cosmic::{
    Element, Task,
    app::ContextDrawer,
    iced::{Length, alignment},
    widget,
};
use openscq30_i18n::Translate;
use openscq30_lib::{quick_presets::QuickPresetsHandler, settings, storage::QuickPreset};

use crate::{
    app::DebugOpenSCQ30Device,
    components::{self, SPACE_M},
    equalizer_line::EqualizerLine,
    fl, handle_soft_error,
    utils::coalesce_result,
};

#[derive(Debug, Clone)]
pub enum Message {
    Warning(String),
    RefreshQuickPresets,
    ShowCreateQuickPresetDialog,
    ActivateQuickPreset(usize),
    EditQuickPreset(usize),
    SetQuickPresets(Vec<QuickPreset>),
    CreateQuickPreset(Option<String>),
    SnapshotQuickPresetSettings(String),
    SetCreateQuickPresetName(String),
    EditQuickPresetToggleField(usize, bool),
    EditQuickPresetClose,
    EditQuickPresetModified,
    ShowDeleteQuickPresetDialog(usize),
    DeleteQuickPreset(String),
    CancelDialog,
}

pub enum Action {
    None,
    Task(Task<Message>),
    FocusTextInput(widget::Id),
    Warning(String),
}

#[derive(Debug, Clone)]
enum Dialog {
    CreateQuickPreset(String),
    DeleteQuickPreset(String),
}

#[derive(Debug)]
pub struct QuickPresetsModel {
    device: DebugOpenSCQ30Device,
    quick_presets_handler: QuickPresetsHandler,
    quick_presets: Option<Vec<QuickPreset>>,
    editing_quick_preset: Option<QuickPreset>,
    dialog: Option<Dialog>,
    /// Name of the preset last applied in this session, if it still exists.
    /// Used only to mark the current card; applying and editing stay separate.
    last_applied: Option<String>,
}

impl QuickPresetsModel {
    pub fn new(
        device: DebugOpenSCQ30Device,
        quick_presets_handler: QuickPresetsHandler,
    ) -> (Self, Task<Message>) {
        let model = Self {
            device,
            quick_presets_handler,
            quick_presets: None,
            editing_quick_preset: None,
            dialog: None,
            last_applied: None,
        };
        let task = model.refresh_quick_presets();
        (model, task)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let Some(quick_presets) = &self.quick_presets else {
            return widget::text(fl!("loading-item", item = fl!("quick-presets"))).into();
        };
        let mut cards = widget::column![].spacing(SPACE_M);
        for (i, preset) in quick_presets.iter().enumerate() {
            let current = self.last_applied.as_deref() == Some(preset.name.as_str());
            cards = cards.push(components::preset_card(
                preset.name.clone(),
                self.summary(preset),
                self.curve(preset),
                current.then(|| fl!("active")),
                (!current).then(|| (fl!("activate"), Message::ActivateQuickPreset(i))),
                Some((fl!("edit"), Message::EditQuickPreset(i))),
                Some((fl!("delete"), Message::ShowDeleteQuickPresetDialog(i))),
            ));
        }
        widget::column![
            widget::row![
                widget::container(components::page_header(
                    fl!("quick-presets"),
                    Some(fl!("quick-presets-description")),
                ))
                .width(Length::Fill),
                widget::button::standard(fl!("create-quick-preset"))
                    .on_press(Message::ShowCreateQuickPresetDialog),
            ]
            .align_y(alignment::Vertical::Center),
            cards,
        ]
        .spacing(SPACE_M)
        .into()
    }

    /// Concise summary: translated names of up to three enabled settings.
    fn summary(&self, preset: &QuickPreset) -> String {
        preset
            .fields
            .iter()
            .filter(|field| field.is_enabled)
            .take(3)
            .map(|field| field.setting_id.translate())
            .collect::<Vec<_>>()
            .join(" · ")
    }

    /// Live mini curve, drawn only when the preset stores band values whose
    /// length matches the connected device's equalizer metadata. Anything
    /// else yields no preview rather than an invented one.
    fn curve(&self, preset: &QuickPreset) -> Option<Element<'_, Message>> {
        let bands = preset.fields.iter().find_map(|field| match &field.value {
            settings::Value::I16Vec(bands) if field.is_enabled => Some(bands.clone()),
            _ => None,
        })?;
        for category in self.device.0.categories() {
            for setting_id in self.device.0.settings_in_category(&category) {
                if let Some(settings::Setting::Equalizer { setting, .. }) =
                    self.device.0.setting(&setting_id)
                    && setting.band_hz.len() == bands.len()
                {
                    return Some(Element::from(
                        widget::canvas(EqualizerLine::new(setting.min, setting.max, bands))
                            .width(Length::Fill),
                    ));
                }
            }
        }
        None
    }

    pub fn dialog(&self) -> Option<Element<'_, Message>> {
        self.dialog.as_ref().map(|dialog| match dialog {
            Dialog::CreateQuickPreset(name) => widget::dialog()
                .title(fl!("create-quick-preset"))
                .control(
                    widget::text_input(fl!("name"), name)
                        .id(widget::Id::new("create-quick-preset-name"))
                        .on_input(Message::SetCreateQuickPresetName)
                        .on_submit(|name| Message::CreateQuickPreset(Some(name))),
                )
                .primary_action(
                    widget::button::suggested(fl!("create"))
                        .on_press(Message::CreateQuickPreset(None)),
                )
                .secondary_action(
                    widget::button::destructive(fl!("cancel")).on_press(Message::CancelDialog),
                )
                .into(),
            Dialog::DeleteQuickPreset(name) => widget::dialog()
                .title(fl!("delete-quick-preset"))
                .body(fl!("delete-confirm", name = name))
                .primary_action(
                    widget::button::destructive(fl!("delete"))
                        .on_press(Message::DeleteQuickPreset(name.to_owned())),
                )
                .secondary_action(
                    widget::button::text(fl!("cancel")).on_press(Message::CancelDialog),
                )
                .into(),
        })
    }

    pub fn context_drawer(&self) -> Option<ContextDrawer<'_, Message>> {
        self.editing_quick_preset
            .as_ref()
            .map(|editing_quick_preset| ContextDrawer {
                title: Some(editing_quick_preset.name.as_str().into()),
                actions: None,
                header: None,
                content: widget::column(editing_quick_preset.fields.iter().enumerate().map(
                    |(i, field)| {
                        widget::column![
                            widget::toggler(field.is_enabled)
                                .label(field.setting_id.translate())
                                .width(Length::Fill)
                                .on_toggle(move |enabled| {
                                    Message::EditQuickPresetToggleField(i, enabled)
                                }),
                            widget::text::body(settings::localize_value(
                                self.device.setting(&field.setting_id).as_ref(),
                                &field.value,
                            )),
                        ]
                        .padding(8)
                        .into()
                    },
                ))
                .push(
                    widget::button::standard(fl!("overwrite-with-current-settings")).on_press(
                        Message::SnapshotQuickPresetSettings(editing_quick_preset.name.to_owned()),
                    ),
                )
                .into(),
                footer: None,
                on_close: Message::EditQuickPresetClose,
            })
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Warning(text) => Action::Warning(text),
            Message::RefreshQuickPresets => Action::Task(self.refresh_quick_presets()),
            Message::ActivateQuickPreset(index) => {
                let Some(name) = self
                    .quick_presets
                    .as_ref()
                    .and_then(|presets| presets.get(index))
                    .map(|preset| preset.name.clone())
                else {
                    return Action::None;
                };
                self.last_applied = Some(name.clone());
                let device = self.device.0.clone();
                let quick_presets_handler = self.quick_presets_handler.clone();
                Action::Task(
                    Task::future(async move {
                        quick_presets_handler
                            .activate(device.as_ref(), name)
                            .await
                            .map_err(handle_soft_error!())?;
                        Ok(Message::RefreshQuickPresets)
                    })
                    .map(coalesce_result),
                )
            }
            Message::EditQuickPreset(index) => {
                self.editing_quick_preset = self
                    .quick_presets
                    .as_ref()
                    .and_then(|presets| presets.get(index))
                    .cloned();
                Action::None
            }
            Message::SetQuickPresets(quick_presets) => {
                if let Some(editing_quick_preset) = &self.editing_quick_preset {
                    self.editing_quick_preset = quick_presets
                        .iter()
                        .find(|preset| preset.name == editing_quick_preset.name)
                        .cloned();
                }
                self.quick_presets = Some(quick_presets);
                Action::None
            }
            Message::ShowCreateQuickPresetDialog => {
                self.dialog = Some(Dialog::CreateQuickPreset(String::new()));
                Action::FocusTextInput(widget::Id::new("create-quick-preset-name"))
            }
            Message::SetCreateQuickPresetName(name) => {
                self.dialog = Some(Dialog::CreateQuickPreset(name));
                Action::None
            }
            Message::CreateQuickPreset(override_name) => {
                let Some(Dialog::CreateQuickPreset(name)) = self.dialog.take() else {
                    return Action::None;
                };
                let name = override_name.unwrap_or(name);

                let device = self.device.clone();
                let quick_presets_handler = self.quick_presets_handler.clone();
                Action::Task(
                    Task::future(async move {
                        quick_presets_handler
                            .save(device.as_ref(), name)
                            .await
                            .map_err(handle_soft_error!())?;
                        Ok(Message::RefreshQuickPresets)
                    })
                    .map(coalesce_result),
                )
            }
            Message::ShowDeleteQuickPresetDialog(index) => {
                if let Some(quick_presets) = &self.quick_presets {
                    self.dialog = quick_presets
                        .get(index)
                        .map(|preset| Dialog::DeleteQuickPreset(preset.name.to_owned()));
                }
                Action::None
            }
            Message::DeleteQuickPreset(name) => {
                self.dialog = None;
                let quick_presets_handler = self.quick_presets_handler.clone();
                let device = self.device.clone();
                Action::Task(
                    Task::future(async move {
                        quick_presets_handler
                            .delete(device.as_ref(), name)
                            .await
                            .map_err(handle_soft_error!())?;
                        Ok(Message::RefreshQuickPresets)
                    })
                    .map(coalesce_result),
                )
            }
            Message::SnapshotQuickPresetSettings(name) => {
                let device = self.device.clone();
                let quick_presets_handler = self.quick_presets_handler.clone();
                Action::Task(
                    Task::future(async move {
                        quick_presets_handler
                            .save(device.as_ref(), name)
                            .await
                            .map_err(handle_soft_error!())?;
                        Ok(Message::RefreshQuickPresets)
                    })
                    .map(coalesce_result),
                )
            }
            Message::EditQuickPresetToggleField(field_index, is_enabled) => {
                if let Some(preset) = &mut self.editing_quick_preset {
                    let preset_name = preset.name.to_owned();

                    // eagerly update the dispalyed value. ideally we would revert back to
                    // what it was if the returned future fails.
                    let field = &mut preset.fields[field_index];
                    field.is_enabled = is_enabled;

                    let setting_id = field.setting_id;
                    let device = self.device.clone();
                    let quick_presets_handler = self.quick_presets_handler.clone();
                    return Action::Task(
                        Task::future(async move {
                            quick_presets_handler
                                .toggle_field(device.as_ref(), preset_name, setting_id, is_enabled)
                                .await
                                .map_err(handle_soft_error!())?;
                            Ok(Message::EditQuickPresetModified)
                        })
                        .map(coalesce_result),
                    );
                }
                Action::None
            }
            Message::EditQuickPresetClose => {
                self.editing_quick_preset = None;
                Action::None
            }
            Message::EditQuickPresetModified => {
                // TODO either modify in place instead of re-fetching all presets, or fetch only the modified preset
                let quick_presets_handler = self.quick_presets_handler.clone();
                let device = self.device.clone();
                Action::Task(
                    Task::future(async move {
                        quick_presets_handler
                            .quick_presets(device.as_ref())
                            .await
                            .map(Message::SetQuickPresets)
                            .map_err(handle_soft_error!())
                    })
                    .map(coalesce_result),
                )
            }
            Message::CancelDialog => {
                self.close_dialog();
                Action::None
            }
        }
    }

    fn refresh_quick_presets(&self) -> Task<Message> {
        let device = self.device.clone();
        let quick_presets_handler = self.quick_presets_handler.clone();
        Task::future(async move {
            quick_presets_handler
                .quick_presets(device.as_ref())
                .await
                .map(Message::SetQuickPresets)
                .map_err(handle_soft_error!())
        })
        .map(coalesce_result)
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }
}
