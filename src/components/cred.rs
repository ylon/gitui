use anyhow::Result;
use crossterm::event::Event;
use tui::{backend::Backend, layout::Rect, Frame};

use asyncgit::sync::cred::BasicAuthCredential;

use crate::components::{InputType, TextInputComponent};
use crate::{
    components::{
        visibility_blocking, CommandBlocking, CommandInfo, Component,
        DrawableComponent,
    },
    keys::SharedKeyConfig,
    strings,
    ui::style::SharedTheme,
};

///
pub struct CredComponent {
    visible: bool,
    key_config: SharedKeyConfig,
    input_username: TextInputComponent,
    input_password: TextInputComponent,
    cred: BasicAuthCredential,
}

impl CredComponent {
    ///
    pub fn new(
        theme: SharedTheme,
        key_config: SharedKeyConfig,
    ) -> Self {
        Self {
            visible: false,
            input_username: TextInputComponent::new(
                theme.clone(),
                key_config.clone(),
                &strings::username_popup_title(&key_config),
                &strings::username_popup_msg(&key_config),
                false,
            )
            .with_input_type(InputType::Singleline),
            input_password: TextInputComponent::new(
                theme,
                key_config.clone(),
                &strings::password_popup_title(&key_config),
                &strings::password_popup_msg(&key_config),
                false,
            )
            .with_input_type(InputType::Password),
            key_config,
            cred: BasicAuthCredential::new(None, None),
        }
    }

    pub fn set_cred(&mut self, cred: BasicAuthCredential) {
        self.cred = cred;
    }

    pub const fn get_cred(&self) -> &BasicAuthCredential {
        &self.cred
    }
}

impl DrawableComponent for CredComponent {
    fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> Result<()> {
        if self.visible {
            self.input_username.draw(f, rect)?;
            self.input_password.draw(f, rect)?;
        }
        Ok(())
    }
}

impl Component for CredComponent {
    fn commands(
        &self,
        out: &mut Vec<CommandInfo>,
        _force_all: bool,
    ) -> CommandBlocking {
        if self.is_visible() {
            out.clear();
        }

        out.push(CommandInfo::new(
            strings::commands::validate_msg(&self.key_config),
            true,
            self.visible,
        ));
        out.push(CommandInfo::new(
            strings::commands::close_popup(&self.key_config),
            true,
            self.visible,
        ));

        visibility_blocking(self)
    }

    fn event(&mut self, ev: Event) -> Result<bool> {
        if self.visible {
            if let Event::Key(e) = ev {
                if e == self.key_config.exit_popup {
                    self.hide();
                }
                if self.input_username.event(ev)?
                    || self.input_password.event(ev)?
                {
                    return Ok(true);
                } else if e == self.key_config.enter {
                    if self.input_username.is_visible() {
                        self.cred = BasicAuthCredential::new(
                            Some(
                                self.input_username
                                    .get_text()
                                    .to_owned(),
                            ),
                            None,
                        );
                        self.input_username.hide();
                        self.input_password.show()?;
                    } else if self.input_password.is_visible() {
                        self.cred = BasicAuthCredential::new(
                            self.cred.username.clone(),
                            Some(
                                self.input_password
                                    .get_text()
                                    .to_owned(),
                            ),
                        );
                        self.input_password.hide();
                        self.input_password.clear();
                        return Ok(false);
                    } else {
                        self.hide();
                    }
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn hide(&mut self) {
        self.cred = BasicAuthCredential::new(None, None);
        self.visible = false;
    }

    fn show(&mut self) -> Result<()> {
        self.visible = true;
        if self.cred.username.is_none() {
            self.input_username.show()
        } else if self.cred.password.is_none() {
            self.input_password.show()
        } else {
            Ok(())
        }
    }
}
