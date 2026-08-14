with open('vnikey-core/src/engine.rs', 'r') as f:
    content = f.read()

old = """        let action = Action::Commit(self.buffer);

        if trigger_key == '\\n' || trigger_key == '\\r' {
            self.reset_context();
        } else {
            self.reset();
        }
        action"""

new = """        let action = Action::Commit(self.buffer);

        self.reset();
        action"""

content = content.replace(old, new)

old_2 = """        self.last_committed_text = self.buffer;

        if !self.buffer.is_full() {
            self.buffer.push(trigger_key);
        }"""
new_2 = """        if !self.buffer.is_full() {
            self.buffer.push(trigger_key);
        }
        self.last_committed_text = self.buffer;"""

content = content.replace(old_2, new_2)

with open('vnikey-core/src/engine.rs', 'w') as f:
    f.write(content)
