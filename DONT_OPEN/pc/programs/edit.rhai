if ENV.args.len() > 1 {
    let path = ENV.cd + ENV.args[1];

    if path.len() < 5 {
        print("    Can only edit '.rhai' files");
        return;
    }

    if path.sub_string(path.len() - 5, path.len()) != ".rhai" {
        print("    Can only edit '.rhai' files");
        return;
    }

    if std::fs::exists(path.to_string()) {
        std::sys::open(path);
    } else {
        print("    Creating new file: " + path.to_string());
        std::fs::write_file(path.to_string(), "");

        std::sys::open(path);
    }
} else {
    print("    EDIT\n");
    print("        USAGE: edit <path>");
}