let files = std::fs::dir("programs");

print("    COMMANDS:\n");
print("      - cd\n");
print("      - exit\n");
print("\n");

print("    PROGRAMS:");

for file in files {
    // ensure file name is long eough
    if file.len() < 5 {
        continue;
    }

    // make sure file name end with '.rhai'
    if file.sub_string(file.len() - 5, file.len()) != ".rhai" {
        continue
    }

    let file = file.sub_string(0, file.len() - 5);

    // don't show if file is hidden
    if file.sub_string(0, 1) == "." {
        continue;
    }

    print("\n");
    print("      - " + file);
}