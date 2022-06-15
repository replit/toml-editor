# Toml Editor

This is the toml editor which is used by the dotreplit service to update the .replit file while preserving formatting (e.g. line spacing and comments).  

To run this, just run `cargo run`. For a production build, run `cargo build --release`.  

More info about this [here](https://replit.com/@util/Design-docs#goval/dotreplit_editor.md)  

Once this is running, it reads json input from stdin and returns output through stdout. 
