let files = std::fs::dir(ENV.cd);

print("    FILES:\n");
print("        ");

for file in files {
    // exempt hidden files
    if file.sub_string(0, 1) == "." {
        continue;
    }

    print(file);
    print("     ");
}
