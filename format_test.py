import re

with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

# First, move `MockIBusHandler` and its tests into `mod tests`
# We will just rewrite the `mod tests` block

# Remove old mod tests
mod_tests_start = content.find("#[cfg(test)]\nmod tests {")
if mod_tests_start != -1:
    content = content[:mod_tests_start]

# Also remove everything after that since we appended mock tests at the end
# Oh wait, we appended MockIBusHandler globally!
