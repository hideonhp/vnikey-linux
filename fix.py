with open('vnikey-core/src/engine.rs', 'r') as f:
    content = f.read()

old = """    fn handle_commit(&mut self, trigger_key: char) -> Action {
        if self.state == State::Idle {
            return Action::PassThrough;
        }

        self.last_committed_raw = self.raw_buffer;
        self.last_committed_text = self.buffer;

        if !self.buffer.is_full() {
            self.buffer.push(trigger_key);
        }

        let action = Action::Commit(self.buffer);

        if trigger_key == '\\n' || trigger_key == '\\r' {
            self.reset_context();
        } else {
            self.reset();
        }
        action
    }"""
new = """    fn handle_commit(&mut self, trigger_key: char) -> Action {
        if self.state == State::Idle {
            return Action::PassThrough;
        }

        self.last_committed_raw = self.raw_buffer;

        if !self.buffer.is_full() {
            self.buffer.push(trigger_key);
        }

        self.last_committed_text = self.buffer;

        let action = Action::Commit(self.buffer);

        self.reset();

        action
    }"""
if old in content:
    with open('vnikey-core/src/engine.rs', 'w') as f:
        f.write(content.replace(old, new))
    print("Patched!")
else:
    print("Not found!")
