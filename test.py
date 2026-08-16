with open("vnikey-core/src/telex.rs", "r") as f:
    text = f.read()

assert "TONE_MAP" not in text
