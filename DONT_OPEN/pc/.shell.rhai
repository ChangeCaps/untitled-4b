fn read_command(command_history) {
    let output = "";

    let cursor_position = std::term::get_cursor_position();

    let event = std::term::read_key();

    let current_command = command_history.len();

    command_history.push("");

    while event.code() != "Enter" {
        switch event.code() {
            "Char" => {
                output += event.char();

                cursor_position.column += 1;

                print(event.char());
            },
            "Backspace" => {
                if output.len() > 0 {
                    output.truncate(output.len() - 1);

                    cursor_position.column -= 1;

                    std::term::set_cursor_position(cursor_position);

                    print(" ");

                    std::term::set_cursor_position(cursor_position);
                }
            },
            "Up" => {
                if current_command > 0 {
                    current_command -= 1;

                    cursor_position.column -= output.len();
                    
                    std::term::set_cursor_position(cursor_position);
                    
                    for i in range(0, output.len()) {
                        print(" ");
                    }

                    output = command_history[current_command];

                    std::term::set_cursor_position(cursor_position);

                    print(output);

                    cursor_position.column += output.len();
                }
            },
            "Down" => {
                if current_command < command_history.len() - 1 {
                    current_command += 1;

                    cursor_position.column -= output.len();

                    std::term::set_cursor_position(cursor_position);

                    for i in range(0, output.len()) {
                        print(" ");
                    }

                    output = command_history[current_command];

                    std::term::set_cursor_position(cursor_position);
                    
                    print(output);

                    cursor_position.column += output.len();
                }
            },
            _ => (),
        }

        event = std::term::read_key();
    }

    output
}

let command_history = [""];
let dir_path = [];

fn path(dir_path) {
    let path = "";

    for dir in dir_path {
        path += dir + "/";
    }

    path
}

loop {
    // get path
    let path = path(dir_path);

    print("/" + path + "> ");

    // get command
    let command = read_command(command_history);

    command.trim();

    // if there is no command, continue so we can read another
    if command.len() == 0 {
        print("\n");
        continue;
    }

    command_history.push(command);

    let args = command.split(" ");

    // handle commands
    switch args[0] {
        "exit" => {
            print(" (exiting shell)");
            break
        },
        "cd" => {
            switch args[1] {
                "." => {},
                ".." => dir_path.pop(),
                _ => {
                    if std::fs::exists(args[1]) {
                        if std::fs::is_dir(args[1]) {
                            dir_path.push(path + args[1]);
                        } else {
                            print("Not a directory");
                        }
                    } else {
                        print("Path doesn't exist");
                    }
                },
            }

            print("\n");

            continue;
        },
        _ => (),
    }

    // get program path
    let program = "programs/" + args[0] + ".rhai";

    // construct env
    let env = #{
        cd: path,
        args: args,
    };

    // check if program exists
    if std::fs::exists(program.to_string()) {
        // run program with env
        let handle = std::sys::run(program, env);
        print("\n");

        // keep running a loop aslong as the program is running
        while handle.running() {
            // sleep a bit so we don't take up all of the cpu
            std::sys::sleep(0.1);


            let keys = std::sys::keys_pressed();

            // termination of running program
            if keys.has("T") && keys.has("LControl") {
                print("\n (terminated program)");
                break;
            }
        }
    
        // if the program has stopped running, it must be safe to terminate
        handle.terminate();

        print("\n");
    } else {
        
        print(" (unrecogniced command: " + command + ")\n");
    }
    
}