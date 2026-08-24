with open("vnikey-ibus/src/main.rs", "r") as f:
    lines = f.readlines()
    for i, l in enumerate(lines):
        if "fn process_key_event(" in l and "self," in lines[i+2]:
            print("".join(lines[i+45:i+100]))
            break
