let inhibit_usage = true;

if inhibit_usage {
    print("    USAGE INHIBITED");
    return;
}

if ENV.args.len() > 1 {
    let path = ENV.cd + ENV.args[0];

    if std::fs::exists(path) {
        let contents = std::fs::read_file(path);

        print(contents);
    } else {
        print("    File '" + path + "' not found");
    }
} else {
    print("    CAT\n");
    print("        USAGE: cat <path>");
}