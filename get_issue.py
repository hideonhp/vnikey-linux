# The issue is probably `keyval_to_char` is returning `None` for these basic keys. Let's check `keyval_to_char`!
with open("vnikey-ibus/src/main.rs", "r") as f:
    lines = f.readlines()
    for i, l in enumerate(lines):
        if "fn keyval_to_char" in l:
            print("".join(lines[i:i+10]))
            break
