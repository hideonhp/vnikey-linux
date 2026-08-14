use vnikey_core::engine::{Action, Engine, InputMethod};

pub struct TestCase<'a> {
    pub input: &'a str,
    pub expected: &'a str,
}

pub fn simulate_typing_str(engine: &mut Engine, keystrokes_str: &str) -> String {
    let keys: Vec<&str> = keystrokes_str.split_whitespace().collect();
    simulate_typing(engine, &keys)
}

pub fn simulate_typing(engine: &mut Engine, keystrokes: &[&str]) -> String {
    let mut committed_text = String::new();
    let mut preedit_text = String::new();

    for &key in keystrokes {
        if key == "[[ToggleTelex]]" {
            if let Some(action) = engine.set_input_method(InputMethod::Telex) {
                match action {
                    Action::Commit(buf) | Action::CommitAndPassThrough(buf) => {
                        let buf_str: String = buf.as_slice().iter().collect();
                        committed_text.push_str(&buf_str);
                        preedit_text.clear();
                    }
                    _ => {}
                }
            }
            continue;
        }

        if key == "[[ToggleVni]]" {
            if let Some(action) = engine.set_input_method(InputMethod::Vni) {
                match action {
                    Action::Commit(buf) | Action::CommitAndPassThrough(buf) => {
                        let buf_str: String = buf.as_slice().iter().collect();
                        committed_text.push_str(&buf_str);
                        preedit_text.clear();
                    }
                    _ => {}
                }
            }
            continue;
        }

        let c = match key {
            "BackSpace" => '\x08',
            "Space" => ' ',
            "Enter" => '\n',
            "Tab" => '\t',
            "Escape" => '\x1b',
            k if k.len() == 1 => k.chars().next().unwrap(),
            _ => panic!("Unknown key: {}", key),
        };

        let action = engine.process_key(c);
        match action {
            Action::Preedit(buf) => {
                preedit_text = buf.as_slice().iter().collect();
            }
            Action::Commit(buf) => {
                let buf_str: String = buf.as_slice().iter().collect();
                committed_text.push_str(&buf_str);
                preedit_text.clear();
            }
            Action::CommitAndPassThrough(buf) => {
                let buf_str: String = buf.as_slice().iter().collect();
                committed_text.push_str(&buf_str);
                preedit_text.clear();

                // Pass-through
                if c != '\x08' {
                    committed_text.push(c);
                } else if !committed_text.is_empty() {
                    committed_text.pop();
                }
            }
            Action::PassThrough => {
                if c != '\x08' {
                    committed_text.push(c);
                } else if !committed_text.is_empty() {
                    committed_text.pop();
                }
            }
            Action::SurroundingRecompose {
                preedit,
                delete_count,
                ..
            } => {
                // Remove delete_count chars from end of committed_text
                committed_text.push_str(&preedit_text);
                preedit_text.clear();

                for _ in 0..delete_count {
                    committed_text.pop();
                }
                preedit_text = preedit.as_slice().iter().collect();
            }
        }
    }

    format!("{}{}", committed_text, preedit_text)
}
