const ICON_EMPTY: ratatui::text::Span = ratatui::text::Span {
    style: ratatui::style::Style::new(),
    content: std::borrow::Cow::Borrowed(" "),
};
const ICON_SELECTED: ratatui::text::Span = ratatui::text::Span {
    style: ratatui::style::Style::new(),
    content: std::borrow::Cow::Borrowed(">"),
};
const ICON_ERROR: ratatui::text::Span = ratatui::text::Span {
    style: ratatui::style::Style::new().fg(ratatui::style::Color::Red),
    content: std::borrow::Cow::Borrowed("×"),
};
const ICON_QUESTION: ratatui::text::Span = ratatui::text::Span {
    style: ratatui::style::Style::new(),
    content: std::borrow::Cow::Borrowed("?"),
};

struct InputField {
    icon: ratatui::text::Span<'static>,
    value: String,
    cursor_index: usize,
    hidden: bool,
}

impl InputField {
    fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Backspace => {
                self.value.pop();
            }
            crossterm::event::KeyCode::Char(c) => {
                self.value.push(c);
                self.icon = ICON_SELECTED;
            }
            _ => {}
        }
    }

    fn required_size(&self) -> u16 {
        u16::try_from(self.value.chars().count()).unwrap_or(u16::MAX)
    }
}

impl ratatui::widgets::Widget for &InputField {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let content = if self.hidden {
            let hidden = std::iter::repeat_n('•', self.value.chars().count()).collect::<String>();
            std::borrow::Cow::Owned(hidden)
        } else {
            std::borrow::Cow::Borrowed(&self.value)
        };
        let icon = ratatui::text::Span::raw(self.icon.to_string());
        let spacer = ratatui::text::Span::raw(" ");
        let text = ratatui::text::Span::raw(content.as_str());

        let line = ratatui::text::Line::from(vec![icon, spacer, text]);
        line.render(area, buf);
    }
}

pub struct LoginInfo {
    username: InputField,
    password: InputField,
    selected_input: SelectedInput,
}

impl LoginInfo {
    pub fn with_username(username: &str) -> Self {
        LoginInfo {
            username: InputField {
                icon: ICON_EMPTY,
                value: username.to_string(),
                cursor_index: username.chars().count(),
                hidden: false,
            },
            password: InputField {
                icon: ICON_SELECTED,
                value: String::new(),
                cursor_index: 0,
                hidden: true,
            },
            selected_input: SelectedInput::Password,
        }
    }

    pub fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Tab => {
                match self.selected_input {
                    SelectedInput::Password => {
                        self.selected_input = SelectedInput::Username;
                        self.username.icon = ICON_SELECTED;
                        self.password.icon = ICON_EMPTY;
                    }
                    SelectedInput::Username => {
                        self.selected_input = SelectedInput::Password;
                        self.username.icon = ICON_EMPTY;
                        self.password.icon = ICON_SELECTED;
                    }
                };
            }
            crossterm::event::KeyCode::Enter => self.login(),
            _ => match self.selected_input {
                SelectedInput::Username => self.username.handle_key_event(event),
                SelectedInput::Password => self.password.handle_key_event(event),
            },
        }
    }

    fn login(&mut self) {
        macro_rules! exit_on_err {
            ($e:expr, $fmt:expr) => {
                match $e {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!($fmt, e);
                        return;
                    }
                }
            };
        }
        if self.password.value.is_empty() {
            self.password.icon = ICON_QUESTION;
            return;
        }

        let mut client = exit_on_err!(pam::Client::with_password("cerbere"), "Failed to create PAM client: {}");
        let conv = client.conversation_mut();
        conv.set_credentials(&self.username.value, &self.password.value);

        /* SAFTEY: Safe as we will set length to zero afterwards. */
        unsafe { self.password.value.as_mut_vec().fill(0) };
        self.password.value.clear();

        match client.authenticate() {
            Ok(_) => match client.open_session() {
                Ok(_) => loop {},
                Err(e) => {
                    let user = exit_on_err!(client.get_user(), "Failed to retrieve user: {}");
                    eprintln!("Failed to open session for user {user}: {e}",);
                    return;
                }
            },
            Err(_) => {
                let user = exit_on_err!(client.get_user(), "Failed to retrieve user: {}");
                eprintln!("User {user} failed auth!");
                self.password.icon = ICON_ERROR;
            }
        }
    }
}

impl ratatui::widgets::Widget for &LoginInfo {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let input_field_widths = self.username.required_size().max(self.password.required_size());
        let required_width = input_field_widths + 4;
        let required_height = 2;

        let parity_width_offset = (required_width + area.width) % 2;
        let login_width = area.width.min(required_width + 2 + parity_width_offset);
        let login_height = area.height.min(required_height + 2);

        let login_x = (area.width - login_width) / 2;
        let login_y = (area.height - login_height) / 2;

        let login_area = ratatui::layout::Rect {
            x: login_x,
            y: login_y,
            width: login_width,
            height: login_height,
        };

        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick);
        block.render(login_area, buf);

        let content_width = login_width.saturating_sub(2);
        let content_height = login_height.saturating_sub(2);

        let username_area = ratatui::layout::Rect {
            x: login_x + 1.min(content_width),
            y: login_y + 1.min(content_height),
            width: content_width.saturating_sub(2),
            height: content_height,
        };
        self.username.render(username_area, buf);

        let password_area = ratatui::layout::Rect {
            x: login_x + 1.min(content_width),
            y: login_y + 2.min(content_height),
            width: content_width.saturating_sub(2),
            height: content_height,
        };
        self.password.render(password_area, buf);
    }
}

enum SelectedInput {
    Username,
    Password,
}
